use super::support::{TestDatabase, insert_with_expiry};
use crate::notifications::relay::{
    Claim, DeliveryResult, Transport, backoff_seconds, claim, finish, run_once_between,
};
use chrono::{Duration, TimeZone, Utc};
use std::{future::Future, pin::Pin};

struct Timeout;

impl Transport for Timeout {
    fn send<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>> {
        Box::pin(async { DeliveryResult::Timeout })
    }
}

#[tokio::test]
async fn completion_clock_retry_after_and_ambiguity_drive_bounded_retry_times() {
    let database = TestDatabase::new().await;
    let started = Utc.with_ymd_and_hms(2026, 9, 7, 8, 0, 0).unwrap();
    insert_with_expiry(
        &database,
        "timeout",
        "timeout",
        1,
        started,
        started + Duration::days(1),
        512,
    )
    .await;
    let completed = started + Duration::seconds(5);
    assert!(run_once_between(&database.primary, &Timeout, started, completed).await.unwrap());
    let (state, next): (String, String) = sqlx::query_as(
        "SELECT state,next_attempt_at FROM notification_outbox WHERE event_id='timeout'",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(state, "reconciling");
    assert_eq!(next, (completed + Duration::seconds(backoff_seconds("timeout", 1))).to_rfc3339());
    sqlx::query("DELETE FROM notification_outbox WHERE event_id='timeout'")
        .execute(&database.primary)
        .await
        .unwrap();

    let at = started + Duration::minutes(1);
    let expires = at + Duration::seconds(20);
    insert_with_expiry(&database, "limited", "limited", 1, at, expires, 512).await;
    let limited = claim(&database.primary, at).await.unwrap().unwrap();
    assert_eq!(limited.event_id, "limited");
    finish(
        &database.primary,
        &limited,
        DeliveryResult::RateLimited { retry_after_seconds: 30 },
        at,
    )
    .await
    .unwrap();
    let (state, next): (String, String) = sqlx::query_as(
        "SELECT state,next_attempt_at FROM notification_outbox WHERE event_id='limited'",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(state, "pending");
    assert_eq!(next, expires.to_rfc3339());
    database.close().await;
}
