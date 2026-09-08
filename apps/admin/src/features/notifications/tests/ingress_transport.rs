use super::{
    ingress::{CURRENT, body_at},
    support::{TestDatabase, grant, policy},
};
use crate::features::notifications::ingress::{IngestOutcome, IngressState};
use chrono::Utc;
use rustzen_ipc::{
    EVENT_CREATED_HEADER, EVENT_EXPIRES_HEADER, EVENT_KEY_ID_HEADER, EVENT_NONCE_HEADER,
    EVENT_PRODUCER_HEADER, EVENT_SIGNATURE_HEADER, EVENT_VERSION_HEADER, NotificationSigner,
};

#[tokio::test]
async fn admin_commit_survives_lost_response_and_reconciles_after_business_expiry() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
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
    let committed = std::sync::Arc::new(tokio::sync::Notify::new());
    let release_response = std::sync::Arc::new(tokio::sync::Notify::new());
    let app = crate::features::notifications::ingress_http::router(state.clone()).layer({
        let committed = committed.clone();
        let release_response = release_response.clone();
        axum::middleware::from_fn(
            move |request: axum::extract::Request, next: axum::middleware::Next| {
                let committed = committed.clone();
                let release_response = release_response.clone();
                async move {
                    let response = next.run(request).await;
                    committed.notify_one();
                    release_response.notified().await;
                    response
                }
            },
        )
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let now = Utc::now();
    let payload = body_at("event-response-lost", "opened", now);
    let signer = NotificationSigner::new("current", "monitor", CURRENT).unwrap();
    let transport_now = Utc::now().timestamp();
    let signed =
        signer.sign(&payload, transport_now, transport_now + 60, "nonce-http-lost").unwrap();
    let request_payload = payload.clone();
    let mut response = tokio::spawn(async move {
        reqwest::Client::new()
            .post(format!("http://{address}{}", rustzen_ipc::NOTIFICATION_PATH))
            .header("content-type", rustzen_ipc::NOTIFICATION_CONTENT_TYPE)
            .header(EVENT_VERSION_HEADER, signed.version)
            .header(EVENT_KEY_ID_HEADER, signed.key_id)
            .header(EVENT_PRODUCER_HEADER, signed.producer)
            .header(EVENT_CREATED_HEADER, signed.created)
            .header(EVENT_EXPIRES_HEADER, signed.expires)
            .header(EVENT_NONCE_HEADER, signed.nonce)
            .header(EVENT_SIGNATURE_HEADER, signed.signature)
            .body(request_payload)
            .send()
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(2), committed.notified())
        .await
        .expect("Admin committed before withholding the response");
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notification_receipts WHERE event_id='event-response-lost'",
        )
        .fetch_one(&database.peer)
        .await
        .unwrap(),
        1
    );
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), &mut response).await.is_err(),
        "client response wait must time out after the Admin commit"
    );
    response.abort();
    release_response.notify_one();
    let after_expiry = now + chrono::Duration::days(2);
    let signed = signer
        .sign(
            &payload,
            after_expiry.timestamp(),
            after_expiry.timestamp() + 60,
            "nonce-http-reconcile",
        )
        .unwrap();
    assert_eq!(state.ingest(signed, &payload, after_expiry).await, Ok(IngestOutcome::Duplicate));
    server.abort();
    database.close().await;
}
