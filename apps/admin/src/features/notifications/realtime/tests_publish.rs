use super::test_support::*;
use crate::features::notifications::{
    admission::AdmissionService, admission_types::AdmissionResult,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use futures::StreamExt;
use std::time::Duration;
use tower::ServiceExt;

async fn chunk(stream: &mut axum::body::BodyDataStream) -> Option<String> {
    tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .unwrap()
        .map(|item| String::from_utf8(item.unwrap().to_vec()).unwrap())
}

async fn wait_for(stream: &mut axum::body::BodyDataStream, needle: &str) -> String {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let value = tokio::time::timeout(remaining, stream.next()).await.unwrap().unwrap().unwrap();
        let value = String::from_utf8(value.to_vec()).unwrap();
        if value.contains(needle) {
            return value;
        }
    }
}

#[tokio::test]
async fn committed_admission_publishes_revision_and_read_commit_publishes_next_revision() {
    let database = Database::new().await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let router = app(database.pool.clone(), realtime.clone(), codec);
    let response = request(router.clone(), Some(&token), "/stream").await;
    let mut stream = response.into_body().into_data_stream();
    let _ = chunk(&mut stream).await;
    let _ = chunk(&mut stream).await;

    let service = AdmissionService::start_with_realtime(
        database.pool.clone(),
        database.path.clone(),
        policy(),
        realtime,
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    let notification_id =
        match service.admit(&event(now, "sse-publish", vec![1]), now).await.unwrap() {
            AdmissionResult::Stored { notification_id, recipients: 1 } => notification_id,
            result => panic!("unexpected admission result: {result:?}"),
        };
    wait_for(&mut stream, r#""revision":1"#).await;

    let read = router
        .oneshot(
            Request::put(format!("/{notification_id}/read"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    wait_for(&mut stream, r#""revision":2"#).await;
    let (_, changed) =
        crate::features::notifications::service::NotificationService::mark_read_for_http(
            &database.pool,
            1,
            &notification_id,
        )
        .await
        .unwrap();
    assert!(!changed, "duplicate read must not publish an unchanged revision");
    drop(stream);
    database.close().await;
}

#[tokio::test]
async fn saturated_stream_never_blocks_admission_and_closes_with_reconcile() {
    let database = Database::new().await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime(database.pool.clone(), 10, 4, 2, clock);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let router = app(database.pool.clone(), realtime.clone(), codec);
    let response = request(router, Some(&token), "/stream").await;

    let service = AdmissionService::start_with_realtime(
        database.pool.clone(),
        database.path.clone(),
        policy(),
        realtime.clone(),
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    tokio::time::timeout(
        Duration::from_secs(1),
        service.admit(&event(now, "sse-lag", vec![1]), now),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(realtime.counts(1), (0, 0));
    let mut stream = response.into_body().into_data_stream();
    wait_for(&mut stream, r#""reason":"lagged""#).await;
    assert!(chunk(&mut stream).await.is_none());
    database.close().await;
}
