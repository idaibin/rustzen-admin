use super::support::{TestDatabase, grant, policy, revoke_all};
use crate::features::notifications::ingress::{
    IngestError, IngestOutcome, IngressState, ProducerKeys,
};
use chrono::{TimeZone, Utc};
use rustzen_ipc::{
    NotificationAudience, NotificationContent, NotificationEvent, NotificationSigner,
    NotificationSubject,
};

const MONITOR_KEY: &[u8] = b"monitor-key-0123456789abcdef-123456";
const REPORTS_KEY: &[u8] = b"reports-key-0123456789abcdef-123456";

fn nonce() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn report(event_id: &str, initiator: i64) -> Vec<u8> {
    serde_json::to_vec(&NotificationEvent {
        schema_version: 1,
        event_id: event_id.into(),
        producer: "reports".into(),
        topic: "reports.run.completed".into(),
        occurred_at: "2026-09-07T00:00:00Z".into(),
        expires_at: "2026-09-08T00:00:00Z".into(),
        subject: NotificationSubject {
            kind: "reports-run".into(),
            id: "run-1".into(),
            revision: 1,
        },
        audience: NotificationAudience {
            policy: "reports-run-initiator".into(),
            initiator_user_id: Some(initiator),
        },
        content: NotificationContent {
            title: "Report run completed".into(),
            summary: "Your report run completed.".into(),
        },
    })
    .unwrap()
}

async fn state(database: &TestDatabase) -> IngressState {
    IngressState::new_with_keys(
        database.primary.clone(),
        database.path.clone(),
        policy(),
        vec![
            ProducerKeys {
                producer: "monitor",
                current_id: "shared-id".into(),
                current_secret: MONITOR_KEY.to_vec(),
                previous: None,
            },
            ProducerKeys {
                producer: "reports",
                current_id: "shared-id".into(),
                current_secret: REPORTS_KEY.to_vec(),
                previous: None,
            },
        ],
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn reports_key_and_current_authority_are_both_producer_scoped() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "reports:run:view").await;
    let state = state(&database).await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap();
    let payload = report("report-1", 2);

    let wrong = NotificationSigner::new("shared-id", "reports", MONITOR_KEY)
        .unwrap()
        .sign(&payload, now.timestamp(), now.timestamp() + 60, nonce())
        .unwrap();
    assert_eq!(state.ingest(wrong, &payload, now).await, Err(IngestError::Unauthorized));

    let signer = NotificationSigner::new("shared-id", "reports", REPORTS_KEY).unwrap();
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::Stored));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT user_id FROM notification_recipients")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        2
    );
    let duplicate = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(duplicate, &payload, now).await, Ok(IngestOutcome::Duplicate));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        1
    );

    sqlx::query("UPDATE users SET status=2 WHERE id=2").execute(&database.primary).await.unwrap();
    let payload = report("report-2", 2);
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::NoRecipients));

    sqlx::query("UPDATE users SET status=1,deleted_at=CURRENT_TIMESTAMP WHERE id=2")
        .execute(&database.primary)
        .await
        .unwrap();
    let payload = report("report-3", 2);
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::NoRecipients));

    sqlx::query("UPDATE users SET deleted_at=NULL WHERE id=2")
        .execute(&database.primary)
        .await
        .unwrap();
    revoke_all(&database.primary).await;
    let payload = report("report-4", 2);
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::NoRecipients));

    grant(&database.primary, "reports:run:view").await;
    sqlx::query("UPDATE modules SET enabled=0 WHERE id='reports'")
        .execute(&database.primary)
        .await
        .unwrap();
    let payload = report("report-5", 2);
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::NoRecipients));
    database.close().await;
}

#[tokio::test]
async fn reports_rejects_cross_topic_and_missing_initiator() {
    let database = TestDatabase::new().await;
    let state = state(&database).await;
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap();
    let signer = NotificationSigner::new("shared-id", "reports", REPORTS_KEY).unwrap();
    let mut event: NotificationEvent = serde_json::from_slice(&report("cross", 2)).unwrap();
    event.topic = "monitor.incident.opened".into();
    let payload = serde_json::to_vec(&event).unwrap();
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Err(IngestError::Forbidden));

    event.topic = "reports.run.failed".into();
    event.audience.initiator_user_id = None;
    let payload = serde_json::to_vec(&event).unwrap();
    let signed = signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce()).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Err(IngestError::Forbidden));
    database.close().await;
}
