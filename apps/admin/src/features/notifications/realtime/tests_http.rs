use super::test_support::*;
use axum::{
    body::to_bytes,
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, RETRY_AFTER},
    },
};
use futures::StreamExt;
use tower::ServiceExt;

async fn chunk(stream: &mut axum::body::BodyDataStream) -> Option<String> {
    tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
        .await
        .unwrap()
        .map(|item| String::from_utf8(item.unwrap().to_vec()).unwrap())
}

#[tokio::test]
async fn initial_revision_ignores_last_event_id_and_never_contains_message_data() {
    let database = Database::new().await;
    sqlx::query(
        "INSERT INTO notification_user_state(user_id,revision,charged_bytes) VALUES(1,42,128)",
    )
    .execute(&database.pool)
    .await
    .unwrap();
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now).await;
    let app = app(database.pool.clone(), realtime.clone(), codec.clone());
    let response = app
        .oneshot(
            axum::http::Request::get("/stream")
                .header("authorization", format!("Bearer {token}"))
                .header("last-event-id", "1")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/event-stream");
    assert_eq!(response.headers()["x-accel-buffering"], "no");
    assert!(response.headers()[CACHE_CONTROL].to_str().unwrap().contains("no-store"));
    let mut stream = response.into_body().into_data_stream();
    let initial = chunk(&mut stream).await.unwrap();
    let revision = chunk(&mut stream).await.unwrap();
    assert!(initial.contains("event: reconcile.required"));
    assert!(initial.contains(r#""reason":"connected""#));
    assert!(revision.contains("event: inbox.changed"));
    assert!(revision.contains("id: 42"));
    assert!(revision.contains(r#""revision":42"#));
    assert!(!format!("{initial}{revision}").contains("title"));
    let heartbeat = chunk(&mut stream).await.unwrap();
    assert!(heartbeat.contains(": heartbeat"));
    drop(stream);
    assert_eq!(realtime.counts(1), (0, 0));
    database.close().await;
}

#[tokio::test]
async fn response_matrix_is_exact_and_all_failures_disable_caching() {
    let database = Database::new().await;
    database.add_user(8, "no-notification-grant", false).await;
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (_, owner) = token(&database.pool, &codec, 1, "owner", now).await;
    let (_, denied) = token(&database.pool, &codec, 8, "no-notification-grant", now).await;
    let app = app(database.pool.clone(), realtime.clone(), codec.clone());

    let unauthorized = request(app.clone(), None, "/stream").await;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(unauthorized.headers()[CACHE_CONTROL], "no-store");
    let invalid = request(app.clone(), Some("invalid.jwt"), "/stream").await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(invalid.headers()[CACHE_CONTROL], "no-store");
    let (mut expired_claims, _) = token(&database.pool, &codec, 1, "owner", now).await;
    expired_claims.exp = usize::try_from(now - 1).unwrap();
    let expired = codec.encode_claims(&expired_claims).unwrap();
    let expired = request(app.clone(), Some(&expired), "/stream").await;
    assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(expired.headers()[CACHE_CONTROL], "no-store");
    let forbidden = request(app.clone(), Some(&denied), "/stream").await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    assert_eq!(forbidden.headers()[CACHE_CONTROL], "no-store");
    let query = request(app.clone(), Some(&owner), "/stream?token=secret").await;
    assert_eq!(query.status(), StatusCode::BAD_REQUEST);
    assert_eq!(query.headers()[CACHE_CONTROL], "no-store");

    realtime.shutdown();
    let draining = request(app, Some(&owner), "/stream").await;
    assert_eq!(draining.status(), StatusCode::NO_CONTENT);
    assert_eq!(draining.headers()[CACHE_CONTROL], "no-store");
    database.close().await;
}

#[tokio::test]
async fn user_and_global_quota_errors_retry_and_body_drop_releases_capacity() {
    let database = Database::new().await;
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now).await;
    let hub = realtime(database.pool.clone(), 10, 4, 16, clock.clone());
    let router = app(database.pool.clone(), hub.clone(), codec.clone());
    let mut held = Vec::new();
    for _ in 0..4 {
        let response = request(router.clone(), Some(&token), "/stream").await;
        assert_eq!(response.status(), StatusCode::OK);
        held.push(response);
    }
    let limited = request(router.clone(), Some(&token), "/stream").await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(limited.headers()[RETRY_AFTER], "60");
    assert_eq!(limited.headers()[CACHE_CONTROL], "no-store");
    let body = to_bytes(limited.into_body(), 4096).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"], 42901);
    drop(held.pop());
    assert_eq!(request(router, Some(&token), "/stream").await.status(), StatusCode::OK);
    drop(held);

    let global = realtime(database.pool.clone(), 1, 4, 16, clock);
    let app = app(database.pool.clone(), global.clone(), codec);
    let first = request(app.clone(), Some(&token), "/stream").await;
    let limited = request(app, Some(&token), "/stream").await;
    assert_eq!(limited.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(limited.headers()[RETRY_AFTER], "60");
    let body = to_bytes(limited.into_body(), 4096).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"], 50301);
    drop(first);
    assert_eq!(global.counts(1), (0, 0));
    database.close().await;
}

#[tokio::test]
async fn initial_authority_store_failure_is_retryable_503() {
    let database = Database::new().await;
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now).await;
    let app = app(database.pool.clone(), realtime, codec);
    database.pool.close().await;

    let response = request(app, Some(&token), "/stream").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[RETRY_AFTER], "60");
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "code": 50302,
            "message": "Authorization authority unavailable",
            "data": null
        })
    );
    database.close().await;
}
