use super::support::TestDatabase;
use crate::notifications::relay::{
    Claim, DeliveryResult, Transport, backoff_seconds, claim, finish, run_once_between,
};
use chrono::{Duration, TimeZone, Utc};
use sha2::{Digest, Sha256};
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

async fn insert(
    database: &TestDatabase,
    id: &str,
    at: chrono::DateTime<Utc>,
    expires: chrono::DateTime<Utc>,
) {
    let payload = format!(r#"{{"eventId":"{id}"}}"#);
    sqlx::query(
        "INSERT INTO notification_outbox
         (event_id,topic,subject_kind,subject_id,subject_revision,payload_json,payload_sha256,
          occurred_at,expires_at,state,next_attempt_at,charged_bytes)
         VALUES(?,'monitor.incident.opened','monitor-incident',?,1,?,?,?,?,'pending',?,512)",
    )
    .bind(id)
    .bind(id)
    .bind(&payload)
    .bind(hex::encode(Sha256::digest(payload.as_bytes())))
    .bind(at.to_rfc3339())
    .bind(expires.to_rfc3339())
    .bind(at.to_rfc3339())
    .execute(&database.primary)
    .await
    .unwrap();
}

#[tokio::test]
async fn response_completion_and_retry_after_set_bounded_next_attempt() {
    let database = TestDatabase::new().await;
    let started = Utc.with_ymd_and_hms(2026, 9, 7, 8, 0, 0).unwrap();
    insert(&database, "timeout", started, started + Duration::days(1)).await;
    let completed = started + Duration::seconds(5);
    assert!(run_once_between(&database.primary, &Timeout, started, completed).await.unwrap());
    let next: String = sqlx::query_scalar(
        "SELECT next_attempt_at FROM notification_outbox WHERE event_id='timeout'",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(next, (completed + Duration::seconds(backoff_seconds("timeout", 1))).to_rfc3339());
    sqlx::query("DELETE FROM notification_outbox WHERE event_id='timeout'")
        .execute(&database.primary)
        .await
        .unwrap();

    let at = started + Duration::minutes(1);
    let expires = at + Duration::seconds(20);
    insert(&database, "limited", at, expires).await;
    let limited = claim(&database.primary, at).await.unwrap().unwrap();
    finish(
        &database.primary,
        &limited,
        DeliveryResult::RateLimited { retry_after_seconds: 30 },
        at,
    )
    .await
    .unwrap();
    let next: String = sqlx::query_scalar(
        "SELECT next_attempt_at FROM notification_outbox WHERE event_id='limited'",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(next, expires.to_rfc3339());
    database.close().await;
}
