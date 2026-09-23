use super::support::TestDatabase;
use crate::notifications::relay::{
    Claim, DeliveryResult, Transport, backoff_seconds, claim, finish, run_batch_at, run_once_at,
};
use chrono::{Duration, TimeZone, Utc};
use sha2::{Digest, Sha256};
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

async fn insert(
    database: &TestDatabase,
    event_id: &str,
    subject: &str,
    revision: i64,
    at: chrono::DateTime<Utc>,
) {
    let payload = format!(r#"{{"eventId":"{event_id}"}}"#);
    let digest = hex::encode(Sha256::digest(payload.as_bytes()));
    sqlx::query(
        "INSERT INTO notification_outbox
         (event_id,topic,subject_kind,subject_id,subject_revision,payload_json,payload_sha256,
          occurred_at,expires_at,state,next_attempt_at,charged_bytes)
         VALUES(?,'monitor.incident.opened','monitor-incident',?,?,?, ?,?,?, 'pending',?,512)",
    )
    .bind(event_id)
    .bind(subject)
    .bind(revision)
    .bind(payload)
    .bind(digest)
    .bind(at.to_rfc3339())
    .bind((at + Duration::days(1)).to_rfc3339())
    .bind(at.to_rfc3339())
    .execute(&database.primary)
    .await
    .unwrap();
}

#[tokio::test]
async fn lease_token_fences_crashed_worker_and_preserves_subject_order() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 2, 0, 0).unwrap();
    insert(&database, "event-open", "incident-a", 1, now).await;
    insert(&database, "event-resolve", "incident-a", 2, now + Duration::seconds(1)).await;

    let crashed = claim(&database.primary, now).await.unwrap().unwrap();
    assert_eq!(crashed.event_id, "event-open");
    assert!(claim(&database.peer, now + Duration::seconds(1)).await.unwrap().is_none());
    let replacement = claim(&database.peer, now + Duration::seconds(31)).await.unwrap().unwrap();
    assert_eq!(replacement.event_id, "event-open");
    assert_ne!(crashed.lease_token, replacement.lease_token);
    assert_eq!(finish(&database.primary, &crashed, DeliveryResult::Stored, now).await.unwrap(), 0);
    assert_eq!(
        finish(&database.primary, &replacement, DeliveryResult::Stored, now).await.unwrap(),
        1
    );
    let resolved = claim(&database.peer, now + Duration::seconds(31)).await.unwrap().unwrap();
    assert_eq!(resolved.event_id, "event-resolve");
    assert_eq!(
        finish(&database.primary, &resolved, DeliveryResult::NoRecipients, now).await.unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT pending_count FROM notification_delivery_status")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        0
    );
    database.close().await;
}

#[tokio::test]
async fn delivery_results_are_terminal_quarantined_or_reconciled() {
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
    for (index, (id, result, expected_state)) in cases.into_iter().enumerate() {
        let at = base + Duration::minutes(index as i64);
        insert(&database, id, id, 1, at).await;
        let claimed = claim(&database.primary, at).await.unwrap().unwrap();
        assert_eq!(claimed.event_id, id);
        assert_eq!(finish(&database.primary, &claimed, result, at).await.unwrap(), 1);
        let observed = sqlx::query_scalar::<_, String>(
            "SELECT state FROM notification_outbox WHERE event_id=?",
        )
        .bind(id)
        .fetch_optional(&database.peer)
        .await
        .unwrap();
        assert_eq!(observed.as_deref(), expected_state);
        if matches!(expected_state, Some("pending" | "reconciling")) {
            let next: String = sqlx::query_scalar(
                "SELECT next_attempt_at FROM notification_outbox WHERE event_id=?",
            )
            .bind(id)
            .fetch_one(&database.peer)
            .await
            .unwrap();
            let delay = if matches!(
                result,
                DeliveryResult::RateLimited { .. } | DeliveryResult::Capacity { .. }
            ) {
                30
            } else {
                let digest = Sha256::digest(format!("{id}:1").as_bytes());
                1 + i64::from(digest[0] % 3)
            };
            assert_eq!(next, (at + Duration::seconds(delay)).to_rfc3339());
            sqlx::query("UPDATE notification_outbox SET next_attempt_at=? WHERE event_id=?")
                .bind((base + Duration::hours(12)).to_rfc3339())
                .bind(id)
                .execute(&database.primary)
                .await
                .unwrap();
        }
    }
    let status = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT pending_count,quarantine_count,expired_count,quarantined_count,unconfirmed_count
         FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(status, (5, 3, 1, 3, 0));

    let retry = claim(&database.primary, base + Duration::hours(13)).await.unwrap().unwrap();
    assert_eq!(retry.event_id, "limited");
    assert_eq!(
        finish(&database.primary, &retry, DeliveryResult::Duplicate, base).await.unwrap(),
        1
    );
    database.close().await;
}

#[tokio::test]
async fn quarantine_retention_is_bounded_per_relay_round() {
    let database = TestDatabase::new().await;
    let old = Utc.with_ymd_and_hms(2026, 9, 5, 0, 0, 0).unwrap();
    for index in 0..101 {
        insert(&database, &format!("old-{index}"), &format!("subject-{index}"), 1, old).await;
    }
    sqlx::query("UPDATE notification_outbox SET state='quarantined'")
        .execute(&database.primary)
        .await
        .unwrap();
    let now = old + Duration::days(2);
    assert!(!run_once_at(&database.primary, &Stored, now).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT quarantine_count FROM notification_delivery_status")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        1
    );
    assert!(!run_once_at(&database.primary, &Stored, now).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT quarantine_count FROM notification_delivery_status")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        0
    );
    database.close().await;
}

#[tokio::test]
async fn terminal_cleanup_respects_live_lease_and_reconcile_deadline() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 6, 0, 0).unwrap();
    insert(&database, "leased", "leased", 1, now).await;
    sqlx::query("UPDATE notification_outbox SET expires_at=? WHERE event_id='leased'")
        .bind((now + Duration::seconds(10)).to_rfc3339())
        .execute(&database.primary)
        .await
        .unwrap();
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
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        0
    );

    let recovered_at = now + Duration::minutes(1);
    insert(&database, "recovered", "recovered", 1, recovered_at).await;
    let claimed = claim(&database.primary, recovered_at).await.unwrap().unwrap();
    finish(&database.primary, &claimed, DeliveryResult::Timeout, recovered_at).await.unwrap();
    let after_business_expiry = recovered_at + Duration::days(1) + Duration::seconds(1);
    let retry = claim(&database.peer, after_business_expiry).await.unwrap().unwrap();
    assert_eq!(retry.event_id, "recovered");
    finish(&database.primary, &retry, DeliveryResult::Duplicate, after_business_expiry)
        .await
        .unwrap();

    let unconfirmed_at = now + Duration::minutes(2);
    insert(&database, "unconfirmed", "unconfirmed", 1, unconfirmed_at).await;
    let claimed = claim(&database.primary, unconfirmed_at).await.unwrap().unwrap();
    finish(&database.primary, &claimed, DeliveryResult::Timeout, unconfirmed_at).await.unwrap();
    assert!(
        !run_once_at(
            &database.peer,
            &Stored,
            unconfirmed_at + Duration::days(1) + Duration::seconds(61),
        )
        .await
        .unwrap()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        0
    );
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
async fn each_of_four_workers_has_a_bounded_twenty_five_item_batch() {
    let database = TestDatabase::new().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 7, 0, 0).unwrap();
    for index in 0..30 {
        insert(&database, &format!("batch-{index}"), &format!("subject-{index}"), 1, now).await;
    }
    assert_eq!(run_batch_at(&database.primary, &Stored, 100, now).await.unwrap(), 25);
    assert_eq!(run_batch_at(&database.peer, &Stored, 100, now).await.unwrap(), 5);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT pending_count FROM notification_delivery_status")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        0
    );
    database.close().await;
}

#[test]
fn retry_backoff_is_deterministic_jittered_and_bounded() {
    assert_eq!(backoff_seconds("stable", 1), backoff_seconds("stable", 1));
    assert!((1..=3).contains(&backoff_seconds("stable", 1)));
    assert_eq!(backoff_seconds("stable", 100), 60);
    let observed: std::collections::BTreeSet<_> =
        (0..100).map(|id| backoff_seconds(&format!("event-{id}"), 1)).collect();
    assert!(observed.len() > 1, "event identity must add deterministic jitter");
    assert!(observed.iter().all(|delay| (1..=60).contains(delay)));
}
