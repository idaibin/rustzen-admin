use super::support::{TestDatabase, insert, insert_with_expiry};
use crate::notifications::relay::{
    Claim, DeliveryResult, Transport, backoff_seconds, claim, finish, run_batch_at, run_once_at,
};
use chrono::{Duration, TimeZone, Utc};
use std::{future::Future, pin::Pin};

struct Stored;

impl Transport for Stored {
    fn send<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>> {
        Box::pin(async { DeliveryResult::Stored })
    }
}

#[tokio::test]
async fn stale_lease_is_fenced_and_earlier_subject_revision_blocks_later_delivery() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 2, 0, 0).unwrap();
    insert(&database, "completed", "run-a", 1, now).await;
    insert(&database, "resolved", "run-a", 2, now + Duration::seconds(1)).await;

    let crashed = claim(&database.primary, now).await.unwrap().unwrap();
    assert_eq!(crashed.event_id, "completed");
    assert!(claim(&database.peer, now + Duration::seconds(1)).await.unwrap().is_none());
    let replacement = claim(&database.peer, now + Duration::seconds(31)).await.unwrap().unwrap();
    assert_eq!(replacement.event_id, "completed");
    assert_ne!(crashed.lease_token, replacement.lease_token);
    assert_eq!(finish(&database.primary, &crashed, DeliveryResult::Stored, now).await.unwrap(), 0);
    assert_eq!(finish(&database.peer, &replacement, DeliveryResult::Stored, now).await.unwrap(), 1);
    let later = claim(&database.primary, now + Duration::seconds(31)).await.unwrap().unwrap();
    assert_eq!(later.event_id, "resolved");
    database.close().await;
}

#[tokio::test]
async fn response_matrix_updates_reports_rows_and_durable_status_exactly() {
    let database = TestDatabase::new().await;
    let base = Utc.with_ymd_and_hms(2026, 9, 7, 3, 0, 0).unwrap();
    let cases = [
        ("stored", DeliveryResult::Stored, None),
        ("duplicate", DeliveryResult::Duplicate, None),
        ("none", DeliveryResult::NoRecipients, None),
        ("expired", DeliveryResult::Expired, None),
        ("conflict", DeliveryResult::Conflict, Some("quarantined")),
        ("invalid", DeliveryResult::Invalid, Some("quarantined")),
        ("unauthorized", DeliveryResult::Unauthorized, Some("quarantined")),
        ("limited", DeliveryResult::RateLimited { retry_after_seconds: 30 }, Some("pending")),
        ("capacity", DeliveryResult::Capacity { retry_after_seconds: 30 }, Some("pending")),
        ("unavailable", DeliveryResult::Unavailable, Some("pending")),
        ("timeout", DeliveryResult::Timeout, Some("reconciling")),
        ("ambiguous", DeliveryResult::Ambiguous, Some("reconciling")),
    ];
    for (index, (id, result, expected)) in cases.into_iter().enumerate() {
        let at = base + Duration::minutes(index as i64);
        insert(&database, id, id, 1, at).await;
        let claimed = claim(&database.primary, at).await.unwrap().unwrap();
        assert_eq!(finish(&database.peer, &claimed, result, at).await.unwrap(), 1);
        let state = sqlx::query_scalar::<_, String>(
            "SELECT state FROM notification_outbox WHERE event_id=?",
        )
        .bind(id)
        .fetch_optional(&database.primary)
        .await
        .unwrap();
        assert_eq!(state.as_deref(), expected);
        if expected.is_some() {
            sqlx::query("UPDATE notification_outbox SET next_attempt_at=? WHERE event_id=?")
                .bind((base + Duration::hours(12)).to_rfc3339())
                .bind(id)
                .execute(&database.primary)
                .await
                .unwrap();
        }
    }
    let status = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT pending_count,quarantine_count,expired_count,quarantined_count,
                unconfirmed_count FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(status, (5, 3, 1, 3, 0));
    database.close().await;
}

#[tokio::test]
async fn expiry_respects_live_lease_and_reconcile_horizon() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 6, 0, 0).unwrap();
    insert_with_expiry(&database, "leased", "leased", 1, now, now + Duration::seconds(10), 512)
        .await;
    let _lease = claim(&database.primary, now).await.unwrap().unwrap();
    assert!(!run_once_at(&database.peer, &Stored, now + Duration::seconds(11)).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        1
    );
    assert!(!run_once_at(&database.peer, &Stored, now + Duration::seconds(31)).await.unwrap());

    let at = now + Duration::minutes(1);
    insert_with_expiry(&database, "ambiguous", "ambiguous", 1, at, at + Duration::seconds(10), 512)
        .await;
    let ambiguous = claim(&database.primary, at).await.unwrap().unwrap();
    finish(&database.primary, &ambiguous, DeliveryResult::Timeout, at).await.unwrap();
    assert!(claim(&database.peer, at + Duration::seconds(11)).await.unwrap().is_some());
    sqlx::query("UPDATE notification_outbox SET lease_token=NULL,lease_until=NULL")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(!run_once_at(&database.peer, &Stored, at + Duration::seconds(71)).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT unconfirmed_count FROM notification_delivery_status WHERE id=1",
        )
        .fetch_one(&database.peer)
        .await
        .unwrap(),
        1
    );
    database.close().await;
}

#[tokio::test]
async fn each_worker_round_is_bounded_to_twenty_five_of_the_hundred_item_batch() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 7, 0, 0).unwrap();
    for index in 0..30 {
        insert(&database, &format!("batch-{index}"), &format!("run-{index}"), 1, now).await;
    }
    assert_eq!(run_batch_at(&database.primary, &Stored, 100, now).await.unwrap(), 25);
    assert_eq!(run_batch_at(&database.peer, &Stored, 100, now).await.unwrap(), 5);
    database.close().await;
}

#[test]
fn retry_backoff_is_deterministic_jittered_and_bounded() {
    assert_eq!(backoff_seconds("stable", 1), backoff_seconds("stable", 1));
    assert!((1..=3).contains(&backoff_seconds("stable", 1)));
    assert_eq!(backoff_seconds("stable", 100), 60);
    let values = (0..100)
        .map(|id| backoff_seconds(&format!("event-{id}"), 1))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(values.len() > 1);
    assert!(values.iter().all(|delay| (1..=60).contains(delay)));
}
