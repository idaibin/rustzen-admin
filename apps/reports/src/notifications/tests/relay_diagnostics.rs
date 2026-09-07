use super::support::{TestDatabase, insert, insert_with_expiry};
use crate::notifications::{
    diagnostics::{inspect_after_commit_at, test_limiter},
    relay::{Claim, DeliveryResult, Transport, claim, finish, run_once_at},
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
async fn reports_gap_diagnostic_observes_all_five_counters_and_rate_limits_commits() {
    let database = TestDatabase::new().await;
    let limiter = test_limiter();
    let base = Utc.with_ymd_and_hms(2026, 9, 7, 9, 0, 0).unwrap();

    let mut rolled_back = database.primary.begin().await.unwrap();
    sqlx::query("UPDATE notification_delivery_status SET omitted_count=1 WHERE id=1")
        .execute(&mut *rolled_back)
        .await
        .unwrap();
    rolled_back.rollback().await.unwrap();
    assert!(!inspect_after_commit_at(&database.peer, &limiter, base).await.unwrap());

    sqlx::query("UPDATE notification_delivery_status SET omitted_count=1 WHERE id=1")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, base).await.unwrap());

    let expired_at = base + Duration::minutes(1);
    insert(&database, "expired", "expired", 1, expired_at).await;
    let expired = claim(&database.primary, expired_at).await.unwrap().unwrap();
    finish(&database.primary, &expired, DeliveryResult::Expired, expired_at).await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, expired_at).await.unwrap());

    let unconfirmed_at = base + Duration::minutes(2);
    insert_with_expiry(
        &database,
        "unconfirmed",
        "unconfirmed",
        1,
        unconfirmed_at,
        unconfirmed_at + Duration::seconds(10),
        512,
    )
    .await;
    let ambiguous = claim(&database.primary, unconfirmed_at).await.unwrap().unwrap();
    finish(&database.primary, &ambiguous, DeliveryResult::Timeout, unconfirmed_at).await.unwrap();
    let reconcile_end = unconfirmed_at + Duration::seconds(71);
    assert!(!run_once_at(&database.peer, &Stored, reconcile_end).await.unwrap());
    assert!(inspect_after_commit_at(&database.peer, &limiter, reconcile_end).await.unwrap());

    let quarantined_at = base + Duration::minutes(5);
    insert(&database, "quarantined", "quarantined", 1, quarantined_at).await;
    let quarantined = claim(&database.primary, quarantined_at).await.unwrap().unwrap();
    finish(&database.primary, &quarantined, DeliveryResult::Conflict, quarantined_at)
        .await
        .unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, quarantined_at).await.unwrap());

    let evicted_at = base + Duration::minutes(6);
    insert_with_expiry(
        &database,
        "evicted-a",
        "evicted-a",
        1,
        evicted_at,
        evicted_at + Duration::days(1),
        2 * 1024 * 1024,
    )
    .await;
    let first = claim(&database.primary, evicted_at).await.unwrap().unwrap();
    finish(&database.primary, &first, DeliveryResult::Invalid, evicted_at).await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, evicted_at).await.unwrap());
    insert_with_expiry(
        &database,
        "evicted-b",
        "evicted-b",
        1,
        evicted_at + Duration::seconds(1),
        evicted_at + Duration::days(1),
        2 * 1024 * 1024,
    )
    .await;
    let second =
        claim(&database.primary, evicted_at + Duration::seconds(1)).await.unwrap().unwrap();
    finish(&database.primary, &second, DeliveryResult::Invalid, evicted_at + Duration::seconds(1))
        .await
        .unwrap();

    let status = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT omitted_count,expired_count,unconfirmed_count,quarantined_count,
                quarantine_evicted_count FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(status, (1, 1, 1, 3, 1));
    assert!(
        !inspect_after_commit_at(&database.peer, &limiter, evicted_at + Duration::seconds(59))
            .await
            .unwrap()
    );
    database.close().await;
}
