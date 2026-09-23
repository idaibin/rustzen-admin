use super::test_support::*;
use std::time::Duration;

async fn assert_closes_after(database: &Database, mutation: &'static str) {
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (claims, _) = token(&database.pool, &codec, 1, "owner", now).await;
    let mut stream = realtime.subscribe(claims).await.unwrap();
    assert!(stream.next_event().await.is_some());
    assert!(stream.next_event().await.is_some());
    sqlx::query(mutation).execute(&database.pool).await.unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let next = tokio::time::timeout(remaining, stream.next_event()).await.unwrap();
        if next.is_none() {
            break;
        }
    }
    assert_eq!(realtime.counts(1), (0, 0));
}

#[tokio::test]
async fn session_user_permission_and_module_changes_close_established_streams() {
    for mutation in [
        "UPDATE access_sessions SET revoked_at=CURRENT_TIMESTAMP",
        "UPDATE users SET auth_epoch=auth_epoch+1 WHERE id=1",
        "DELETE FROM role_menus WHERE role_id=1",
        "UPDATE modules SET enabled=0 WHERE id='monitor'",
    ] {
        let database = Database::new().await;
        assert_closes_after(&database, mutation).await;
        database.close().await;
    }
}

#[tokio::test]
async fn exact_expiry_and_authority_database_failure_close_without_a_frame() {
    let database = Database::new().await;
    let now = chrono::Utc::now().timestamp();
    let (clock_value, clock) = fixed_clock(now);
    let realtime = realtime(database.pool.clone(), 10, 4, 16, clock);
    let codec = codec();
    let (claims, _) = token(&database.pool, &codec, 1, "owner", now).await;
    let mut stream = realtime.subscribe(claims).await.unwrap();
    assert!(stream.next_event().await.is_some());
    assert!(stream.next_event().await.is_some());

    clock_value.store(now + 3_600, std::sync::atomic::Ordering::SeqCst);
    assert!(stream.next_event().await.is_none());
    assert_eq!(realtime.counts(1), (0, 0));

    clock_value.store(now, std::sync::atomic::Ordering::SeqCst);
    let (claims, _) = token(&database.pool, &codec, 1, "owner", now).await;
    let mut stream = realtime.subscribe(claims).await.unwrap();
    assert!(stream.next_event().await.is_some());
    assert!(stream.next_event().await.is_some());
    database.pool.close().await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    while tokio::time::timeout(
        deadline.saturating_duration_since(tokio::time::Instant::now()),
        stream.next_event(),
    )
    .await
    .unwrap()
    .is_some()
    {}
    database.close().await;
}

#[tokio::test]
async fn jwt_expiry_deadline_releases_an_unpolled_stream_without_waiting_for_other_events() {
    let database = Database::new().await;
    let now = chrono::Utc::now().timestamp();
    let (_, clock) = fixed_clock(now);
    let realtime = realtime_with_deadlines(
        database.pool.clone(),
        Duration::from_secs(2),
        Duration::from_secs(3),
        clock,
    );
    let codec = codec();
    let (mut claims, _) = token(&database.pool, &codec, 1, "owner", now).await;
    claims.exp = (now + 1) as usize;
    let _stream = realtime.subscribe(claims).await.unwrap();
    assert_eq!(realtime.counts(1), (1, 1));
    assert_eq!(realtime.supervisor_count(), 1);

    tokio::time::timeout(Duration::from_millis(1_200), async {
        while realtime.counts(1) != (0, 0) || realtime.supervisor_count() != 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("JWT deadline must release quota and supervisor near exact expiry");
    database.close().await;
}
