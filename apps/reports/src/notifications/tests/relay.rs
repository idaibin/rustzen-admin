use super::{close, database, run};
use crate::{features::automation::repo, notifications::relay::*};
use chrono::{TimeZone, Utc};
use std::{future::Future, pin::Pin};

struct ResultTransport(DeliveryResult);

impl Transport for ResultTransport {
    fn send<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>> {
        Box::pin(async move { self.0 })
    }
}

#[tokio::test]
async fn accepted_delivery_deletes_payload_and_reopen_keeps_accounting() {
    let (pool, path) = database().await;
    run(&pool, "accepted", "running", Some(1)).await;
    repo::finish_run(&pool, "accepted", "succeeded", None, "2026-09-07T00:00:00Z").await.unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 1, 0).unwrap();
    assert!(run_once_at(&pool, &ResultTransport(DeliveryResult::Stored), now).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    crate::notifications::runtime::validate_accounting(&pool).await.unwrap();
    close(pool, path).await;
}

#[tokio::test]
async fn retryable_and_ambiguous_results_preserve_original_event() {
    let (pool, path) = database().await;
    run(&pool, "retry", "running", Some(1)).await;
    repo::finish_run(&pool, "retry", "failed", Some("failed"), "2026-09-07T00:00:00Z")
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 1, 0).unwrap();
    assert!(run_once_at(&pool, &ResultTransport(DeliveryResult::Unavailable), now).await.unwrap());
    sqlx::query("UPDATE notification_outbox SET next_attempt_at=?")
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    assert!(run_once_at(&pool, &ResultTransport(DeliveryResult::Timeout), now).await.unwrap());
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM notification_outbox")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "reconciling"
    );
    close(pool, path).await;
}
