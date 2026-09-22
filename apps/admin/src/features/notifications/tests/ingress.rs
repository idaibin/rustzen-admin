use super::support::{TestDatabase, grant, policy, revoke_all};
use crate::features::notifications::ingress::{IngestError, IngestOutcome, IngressState};
use chrono::{TimeZone, Utc};
use rustzen_ipc::{
    EVENT_CREATED_HEADER, EVENT_EXPIRES_HEADER, EVENT_KEY_ID_HEADER, EVENT_NONCE_HEADER,
    EVENT_PRODUCER_HEADER, EVENT_SIGNATURE_HEADER, EVENT_VERSION_HEADER, NotificationAudience,
    NotificationContent, NotificationEvent, NotificationHeaders, NotificationSigner,
    NotificationSubject,
};
use tower::ServiceExt;

pub(super) const CURRENT: &[u8] = b"0123456789abcdef0123456789abcdef";
const PREVIOUS: &[u8] = b"previous-key-0123456789abcdef-1234";

fn nonce(label: &str) -> String {
    format!("{label}-{}", std::process::id())
}

pub(super) fn body(event_id: &str, summary: &str) -> Vec<u8> {
    body_at(event_id, summary, Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap())
}

pub(super) fn body_at(
    event_id: &str,
    summary: &str,
    occurred_at: chrono::DateTime<Utc>,
) -> Vec<u8> {
    serde_json::to_vec(&NotificationEvent {
        schema_version: 1,
        event_id: event_id.into(),
        producer: "monitor".into(),
        topic: "monitor.incident.opened".into(),
        occurred_at: occurred_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        expires_at: (occurred_at + chrono::Duration::days(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        subject: NotificationSubject {
            kind: "monitor-incident".into(),
            id: format!("incident-{event_id}"),
            revision: 1,
        },
        audience: NotificationAudience {
            policy: "monitor-incident-readers".into(),
            initiator_user_id: None,
        },
        content: NotificationContent { title: "CPU high".into(), summary: summary.into() },
    })
    .unwrap()
}

#[tokio::test]
async fn signed_ingress_recomputes_recipients_and_deduplicates() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let state = IngressState::new(
        database.primary.clone(),
        database.path.clone(),
        policy(),
        "current".into(),
        CURRENT.to_vec(),
        Some(("previous".into(), PREVIOUS.to_vec(), Utc::now().timestamp() + 60)),
    )
    .await
    .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap();
    let payload = body("event-1", "opened");
    let signer = NotificationSigner::new("current", "monitor", CURRENT).unwrap();
    let signed =
        signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-1")).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::Stored));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_recipients")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        2
    );
    assert!(!String::from_utf8_lossy(&payload).contains("recipient"));
    assert!(!String::from_utf8_lossy(&payload).contains("capability"));

    let duplicate =
        signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-2")).unwrap();
    assert_eq!(state.ingest(duplicate, &payload, now).await, Ok(IngestOutcome::Duplicate));
    let after_business_expiry = now + chrono::Duration::days(2);
    let duplicate = signer
        .sign(
            &payload,
            after_business_expiry.timestamp(),
            after_business_expiry.timestamp() + 60,
            nonce("nonce-expired-duplicate"),
        )
        .unwrap();
    assert_eq!(
        state.ingest(duplicate, &payload, after_business_expiry).await,
        Ok(IngestOutcome::Duplicate)
    );
    let expired_new = body("event-expired-new", "expired");
    let signed = signer
        .sign(
            &expired_new,
            after_business_expiry.timestamp(),
            after_business_expiry.timestamp() + 60,
            nonce("nonce-expired-new"),
        )
        .unwrap();
    assert_eq!(
        state.ingest(signed, &expired_new, after_business_expiry).await,
        Err(IngestError::Expired)
    );
    let changed = body("event-1", "tampered");
    let conflict =
        signer.sign(&changed, now.timestamp(), now.timestamp() + 60, nonce("nonce-3")).unwrap();
    assert_eq!(state.ingest(conflict, &changed, now).await, Err(IngestError::Conflict));

    revoke_all(&database.primary).await;
    sqlx::query("UPDATE users SET status=2 WHERE id=1").execute(&database.primary).await.unwrap();
    let payload = body("event-2", "no recipients");
    let previous = NotificationSigner::new("previous", "monitor", PREVIOUS).unwrap();
    let signed =
        previous.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-4")).unwrap();
    assert_eq!(state.ingest(signed, &payload, now).await, Ok(IngestOutcome::NoRecipients));
    database.close().await;
}

#[tokio::test]
async fn ingress_rejects_tamper_replay_expiry_and_oversize() {
    let database = TestDatabase::new().await;
    let state = IngressState::new(
        database.primary.clone(),
        database.path.clone(),
        policy(),
        "current".into(),
        CURRENT.to_vec(),
        None,
    )
    .await
    .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap();
    let signer = NotificationSigner::new("current", "monitor", CURRENT).unwrap();
    let payload = body("event-auth", "opened");
    let signed =
        signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-auth")).unwrap();
    assert_eq!(state.ingest(signed.clone(), b"{}", now).await, Err(IngestError::Unauthorized));
    assert_eq!(state.ingest(signed.clone(), &payload, now).await, Ok(IngestOutcome::Stored));
    assert_eq!(state.ingest(signed, &payload, now).await, Err(IngestError::Unauthorized));

    let expired = signer
        .sign(&payload, now.timestamp() - 60, now.timestamp(), nonce("nonce-expired"))
        .unwrap();
    assert_eq!(state.ingest(expired, &payload, now).await, Err(IngestError::Expired));
    let oversized = vec![b'x'; 16 * 1024 + 1];
    let signed = signer
        .sign(&oversized, now.timestamp(), now.timestamp() + 60, nonce("nonce-large"))
        .unwrap();
    assert_eq!(state.ingest(signed, &oversized, now).await, Err(IngestError::BadRequest));

    let rotation = Utc::now();
    assert!(
        IngressState::new(
            database.primary.clone(),
            database.path.clone(),
            policy(),
            "current".into(),
            CURRENT.to_vec(),
            Some(("previous".into(), PREVIOUS.to_vec(), rotation.timestamp() + 121)),
        )
        .await
        .is_err()
    );
    let rotating = IngressState::new(
        database.primary.clone(),
        database.path.clone(),
        policy(),
        "current".into(),
        CURRENT.to_vec(),
        Some(("previous".into(), PREVIOUS.to_vec(), rotation.timestamp() + 1)),
    )
    .await
    .unwrap();
    let previous = NotificationSigner::new("previous", "monitor", PREVIOUS).unwrap();
    let observed = rotation + chrono::Duration::seconds(1);
    let signed = previous
        .sign(&payload, observed.timestamp(), observed.timestamp() + 60, nonce("nonce-old-key"))
        .unwrap();
    assert_eq!(rotating.ingest(signed, &payload, observed).await, Err(IngestError::Unauthorized));
    database.close().await;
}

fn request(
    payload: Vec<u8>,
    headers: &NotificationHeaders,
    content_type: bool,
) -> axum::http::Request<axum::body::Body> {
    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri(rustzen_ipc::NOTIFICATION_PATH)
        .header(EVENT_VERSION_HEADER, &headers.version)
        .header(EVENT_KEY_ID_HEADER, &headers.key_id)
        .header(EVENT_PRODUCER_HEADER, &headers.producer)
        .header(EVENT_CREATED_HEADER, headers.created)
        .header(EVENT_EXPIRES_HEADER, headers.expires)
        .header(EVENT_NONCE_HEADER, &headers.nonce)
        .header(EVENT_SIGNATURE_HEADER, &headers.signature);
    if content_type {
        request = request.header("content-type", rustzen_ipc::NOTIFICATION_CONTENT_TYPE);
    }
    request.body(axum::body::Body::from(payload)).unwrap()
}

#[tokio::test]
async fn dedicated_http_route_enforces_protocol_headers_and_content_type() {
    let database = TestDatabase::new().await;
    let state = IngressState::new(
        database.primary.clone(),
        database.path.clone(),
        policy(),
        "current".into(),
        CURRENT.to_vec(),
        None,
    )
    .await
    .unwrap();
    let app = crate::features::notifications::ingress_http::router(state);
    let now = Utc::now();
    let payload = body_at("event-http", "opened", now);
    let signer = NotificationSigner::new("current", "monitor", CURRENT).unwrap();
    let signed =
        signer.sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-http")).unwrap();
    let response = app.clone().oneshot(request(payload.clone(), &signed, false)).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let signed = signer
        .sign(&payload, now.timestamp(), now.timestamp() + 60, nonce("nonce-http-valid"))
        .unwrap();
    let response = app.oneshot(request(payload, &signed, true)).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    assert_eq!(response.headers()["content-type"], "application/json");
    database.close().await;
}
