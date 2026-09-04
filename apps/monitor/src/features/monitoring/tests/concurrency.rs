use super::support::*;

#[tokio::test]
async fn sqlite_foreign_key_and_active_incident_uniqueness_are_enforced() {
    let pool = migrated_test_pool().await;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await.unwrap();
    let missing_node = sqlx::query("INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES('missing',1,1,2,'2026-09-02T00:00:00Z')")
        .execute(&pool)
        .await;
    assert!(missing_node.is_err());
    let boot = Uuid::new_v4();
    let at = Utc.with_ymd_and_hms(2026, 9, 2, 6, 0, 0).unwrap();
    record_at(&pool, report("unique-node", boot, 1, at, 10.0, 10, 10), at).await.unwrap();
    sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES('incident-a','unique-node','nodeOffline','node','active','offline',?,?)")
        .bind(at.to_rfc3339()).bind(at.to_rfc3339()).execute(&pool).await.unwrap();
    let duplicate = sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES('incident-b','unique-node','nodeOffline','node','active','offline',?,?)")
        .bind(at.to_rfc3339()).bind(at.to_rfc3339()).execute(&pool).await;
    assert!(duplicate.is_err());
}

#[tokio::test]
async fn concurrent_replay_has_one_accept_and_one_duplicate() {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    let filename = std::env::temp_dir().join(format!("rustzen-monitor-race-{}.db", Uuid::new_v4()));
    let options = SqliteConnectOptions::new()
        .filename(&filename)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(2));
    let pool = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
    crate::infra::db::migrate(&pool).await.unwrap();
    let boot = Uuid::new_v4();
    let at = Utc.with_ymd_and_hms(2026, 9, 2, 7, 0, 0).unwrap();
    let first = report("race-node", boot, 1, at, 10.0, 10, 10);
    let second = first.clone();
    let (left, right) = tokio::join!(record_at(&pool, first, at), record_at(&pool, second, at));
    let statuses = [left.unwrap(), right.unwrap()];
    assert!(statuses.contains(&AgentReportStatus::Accepted));
    assert!(statuses.contains(&AgentReportStatus::Duplicate));
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM resource_samples WHERE node_id='race-node'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    pool.close().await;
    let _ = std::fs::remove_file(filename);
}

#[tokio::test]
async fn concurrent_scan_and_report_use_the_latest_received_time() {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    let filename =
        std::env::temp_dir().join(format!("rustzen-monitor-offline-race-{}.db", Uuid::new_v4()));
    let options = SqliteConnectOptions::new()
        .filename(&filename)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(2));
    let pool = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
    crate::infra::db::migrate(&pool).await.unwrap();
    let boot = Uuid::new_v4();
    let base = Utc.with_ymd_and_hms(2026, 9, 2, 9, 0, 0).unwrap();
    record_at(&pool, report("offline-race-node", boot, 1, base, 10.0, 10, 10), base).await.unwrap();
    let interleaving = base + ChronoDuration::seconds(91);
    let scan_hook = LockHook {
        acquired: std::sync::Arc::new(Notify::new()),
        release: std::sync::Arc::new(Notify::new()),
    };
    let scan_acquired = scan_hook.acquired.notified();
    let scan_pool = pool.clone();
    let scan_hook_for_task = scan_hook.clone();
    let scan = tokio::spawn(async move {
        offline_scan_at_with_lock_hook(&scan_pool, interleaving, scan_hook_for_task).await
    });
    scan_acquired.await;
    let report_pool = pool.clone();
    let received = tokio::spawn(async move {
        record_at(
            &report_pool,
            report("offline-race-node", boot, 2, interleaving, 10.0, 10, 10),
            interleaving,
        )
        .await
    });
    scan_hook.release.notify_one();
    assert!(scan.await.unwrap().is_ok());
    assert_eq!(received.await.unwrap().unwrap(), AgentReportStatus::Accepted);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='offline-race-node' AND kind='nodeOffline' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    let second_interleaving = base + ChronoDuration::seconds(182);
    let report_hook = LockHook {
        acquired: std::sync::Arc::new(Notify::new()),
        release: std::sync::Arc::new(Notify::new()),
    };
    let report_acquired = report_hook.acquired.notified();
    let report_pool = pool.clone();
    let report_hook_for_task = report_hook.clone();
    let received = tokio::spawn(async move {
        record_at_with_lock_hook(
            &report_pool,
            report("offline-race-node", boot, 3, second_interleaving, 10.0, 10, 10),
            second_interleaving,
            report_hook_for_task,
        )
        .await
    });
    report_acquired.await;
    let scan_pool = pool.clone();
    let scan = tokio::spawn(async move { offline_scan_at(&scan_pool, second_interleaving).await });
    report_hook.release.notify_one();
    assert_eq!(received.await.unwrap().unwrap(), AgentReportStatus::Accepted);
    assert!(scan.await.unwrap().is_ok());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='offline-race-node' AND kind='nodeOffline' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    pool.close().await;
    let _ = std::fs::remove_file(filename);
}
