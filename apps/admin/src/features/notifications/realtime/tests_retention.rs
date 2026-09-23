use super::test_support::*;
use crate::features::notifications::{
    admission::AdmissionService, maintenance, service::NotificationService,
};
use chrono::Utc;
use futures::StreamExt;
use std::time::Duration;

async fn wait_for(stream: &mut axum::body::BodyDataStream, received: &mut String, needle: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    while !received.contains(needle) {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let bytes = tokio::time::timeout(remaining, stream.next())
            .await
            .unwrap_or_else(|_| panic!("timed out waiting for {needle}"))
            .unwrap()
            .unwrap();
        received.push_str(std::str::from_utf8(&bytes).unwrap());
    }
}

async fn admit_old(database: &Database, event_id: &str, user_id: i64) {
    let accepted_at = Utc::now().naive_utc() - chrono::Duration::days(31);
    AdmissionService::start(database.pool.clone(), database.path.clone(), policy())
        .await
        .unwrap()
        .admit(&event(accepted_at, event_id, vec![user_id]), accepted_at)
        .await
        .unwrap();
}

#[tokio::test]
async fn periodic_cleanup_publishes_committed_revision_and_releases_recipient() {
    let database = Database::new().await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let response = request(
        app(database.pool.clone(), realtime.clone(), codec.clone()),
        Some(&token),
        "/stream",
    )
    .await;
    let mut stream = response.into_body().into_data_stream();
    let mut received = String::new();
    wait_for(&mut stream, &mut received, r#""reason":"connected""#).await;
    wait_for(&mut stream, &mut received, r#""revision":0"#).await;
    let maintenance = maintenance::start_for_test(
        database.pool.clone(),
        Duration::from_millis(100),
        Utc::now().naive_utc(),
        Some(realtime.clone()),
    )
    .await
    .unwrap();
    admit_old(&database, "periodic-old", 1).await;
    tokio::time::timeout(Duration::from_secs(1), async {
        while realtime.pending(1) == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    wait_for(&mut stream, &mut received, r#""revision":2"#).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notification_recipients WHERE user_id=1",
        )
        .fetch_one(&database.pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(realtime.counts(1), (1, 1));
    assert_eq!(
        NotificationService::unread_count_at(&database.pool, 1, Utc::now().naive_utc(),)
            .await
            .unwrap()
            .revision,
        2
    );
    drop(stream);

    let response =
        request(app(database.pool.clone(), realtime.clone(), codec), Some(&token), "/stream").await;
    let mut reconnected = response.into_body().into_data_stream();
    let mut received = String::new();
    wait_for(&mut reconnected, &mut received, r#""reason":"connected""#).await;
    wait_for(&mut reconnected, &mut received, r#""revision":2"#).await;
    let now = Utc::now().naive_utc();
    AdmissionService::start_with_realtime(
        database.pool.clone(),
        database.path.clone(),
        policy(),
        realtime.clone(),
    )
    .await
    .unwrap()
    .admit(&event(now, "after-cleanup", vec![1]), now)
    .await
    .unwrap();
    wait_for(&mut reconnected, &mut received, r#""revision":3"#).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT revision FROM notification_user_state WHERE user_id=1",
        )
        .fetch_one(&database.pool)
        .await
        .unwrap(),
        3
    );
    drop(reconnected);
    maintenance.shutdown().await;
    database.close().await;
}

#[tokio::test]
async fn admission_cleanup_publishes_to_affected_user_outside_new_audience() {
    let database = Database::new().await;
    database.add_user(9_002, "cleanup-reader", true).await;
    admit_old(&database, "admission-old", 9_002).await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (claims, _) = token(&database.pool, &codec, 9_002, "cleanup-reader", now_epoch).await;
    let mut stream = realtime.subscribe(claims).await.unwrap();
    assert!(stream.next_event().await.is_some());
    assert!(stream.next_event().await.is_some());

    let service = AdmissionService::start_with_realtime(
        database.pool.clone(),
        database.path.clone(),
        policy(),
        realtime.clone(),
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    service.admit(&event(now, "fresh-other-user", vec![1]), now).await.unwrap();
    assert_eq!(realtime.pending(9_002), 1);
    assert!(stream.next_event().await.is_some());
    assert_eq!(realtime.pending(9_002), 0);
    database.close().await;
}

#[tokio::test]
async fn rollback_and_no_change_never_publish_cleanup_revision() {
    let database = Database::new().await;
    admit_old(&database, "rollback-old", 1).await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (claims, _) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let _stream = realtime.subscribe(claims).await.unwrap();
    assert_eq!(realtime.pending(1), 2);

    let now = Utc::now().naive_utc();
    let rolled_back =
        maintenance::run_round_for_test(&database.pool, now, &realtime, false).await.unwrap();
    assert_eq!(rolled_back.invalidations, [(1, 2)]);
    assert_eq!(realtime.pending(1), 2);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT revision FROM notification_user_state WHERE user_id=1",
        )
        .fetch_one(&database.pool)
        .await
        .unwrap(),
        1
    );

    let committed =
        maintenance::run_round_for_test(&database.pool, now, &realtime, true).await.unwrap();
    assert_eq!(committed.invalidations, [(1, 2)]);
    assert_eq!(realtime.pending(1), 3);
    let no_change =
        maintenance::run_round_for_test(&database.pool, now, &realtime, true).await.unwrap();
    assert!(no_change.invalidations.is_empty());
    assert_eq!(realtime.pending(1), 3);
    database.close().await;
}
