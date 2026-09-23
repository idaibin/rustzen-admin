use super::support::*;

#[tokio::test]
async fn active_incident_remains_queryable_after_thirty_days() {
    let pool = migrated_test_pool().await;
    let opened = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    sqlx::query("INSERT INTO monitor_nodes(node_id,hostname,agent_version,current_boot_id,last_sequence,last_report_at,last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at) VALUES('old-node','old-node','test','boot',1,?,?,?,?,?,?,?)")
        .bind(opened.to_rfc3339()).bind(opened.to_rfc3339()).bind(10.0_f64).bind(50_i64).bind(100_i64)
        .bind(opened.to_rfc3339()).bind(opened.to_rfc3339()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,threshold_percent,observed_percent,opened_at,last_observed_at) VALUES('old-incident','old-node','cpuHigh','cpu','active','old',90,95,?,?)")
        .bind(opened.to_rfc3339()).bind(opened.to_rfc3339()).execute(&pool).await.unwrap();
    let from = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap().to_rfc3339();
    let to = Utc.with_ymd_and_hms(2026, 9, 2, 0, 0, 0).unwrap().to_rfc3339();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM monitor_incidents WHERE status='active' AND (status='active' OR (status='resolved' AND resolved_at>=? AND resolved_at<=?))")
        .bind(from).bind(to).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn offline_scan_is_clock_injected_and_next_accepted_report_recovers() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let reported = Utc.with_ymd_and_hms(2026, 9, 1, 3, 0, 0).unwrap();
    record_at(&pool, report("offline-node", boot, 1, reported, 10.0, 10, 10), reported)
        .await
        .unwrap();
    let scan_at = reported + ChronoDuration::seconds(90);
    offline_scan_at(&pool, scan_at).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='nodeOffline' AND status='active'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    let offline_at = scan_at + ChronoDuration::seconds(1);
    offline_scan_at(&pool, offline_at).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='nodeOffline' AND status='active'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    let first_observed: String = sqlx::query_scalar("SELECT last_observed_at FROM monitor_incidents WHERE kind='nodeOffline' AND status='active'").fetch_one(&pool).await.unwrap();
    offline_scan_at(&pool, offline_at + ChronoDuration::seconds(30)).await.unwrap();
    let second_observed: String = sqlx::query_scalar("SELECT last_observed_at FROM monitor_incidents WHERE kind='nodeOffline' AND status='active'").fetch_one(&pool).await.unwrap();
    assert!(second_observed > first_observed);
    let recovered_at = offline_at + ChronoDuration::seconds(31);
    record_at(&pool, report("offline-node", boot, 2, recovered_at, 10.0, 10, 10), recovered_at)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='nodeOffline' AND status='resolved'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn cleanup_keeps_cutoff_and_removes_older_raw_data() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 0, 0, 0).unwrap();
    record_at(
        &pool,
        report(
            "retention-node",
            boot,
            1,
            now - ChronoDuration::days(30) - ChronoDuration::seconds(1),
            10.0,
            10,
            10,
        ),
        now - ChronoDuration::days(30) - ChronoDuration::seconds(1),
    )
    .await
    .unwrap();
    record_at(
        &pool,
        report("retention-node", boot, 2, now - ChronoDuration::days(30), 10.0, 10, 10),
        now - ChronoDuration::days(30),
    )
    .await
    .unwrap();
    let cleanup = cleanup_at(&pool, now).await.unwrap();
    assert_eq!(cleanup.deleted_rows, 3);
    assert!(matches!(cleanup.maintenance, MaintenanceResult::Succeeded(_)));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource_samples")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_nodes")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    let second_cleanup = cleanup_at(&pool, now).await.unwrap();
    assert_eq!(second_cleanup.deleted_rows, 0);
    assert!(matches!(second_cleanup.maintenance, MaintenanceResult::NotNeeded));
}

#[tokio::test]
async fn file_cleanup_reports_reclaim_maintenance_and_is_idempotent() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("rustzen-monitor-cleanup-{stamp}.db"));
    let pool = rustzen_storage::connect_sqlite_with_options(
        &rustzen_storage::database_url_from_path(&path),
        rustzen_storage::DatabaseConnectionOptions {
            max_connections: 1,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: None,
        },
    )
    .await
    .expect("open cleanup database");
    crate::infra::db::migrate(&pool).await.unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 0, 0, 0).unwrap();
    let old = now - ChronoDuration::days(30) - ChronoDuration::seconds(1);
    record_at(&pool, report("file-cleanup-node", Uuid::new_v4(), 1, old, 10.0, 10, 10), old)
        .await
        .unwrap();
    let cleanup = cleanup_at(&pool, now).await.unwrap();
    let maintenance = match cleanup.maintenance {
        MaintenanceResult::Succeeded(report) => report,
        other => panic!("expected maintenance report, got {other:?}"),
    };
    assert_eq!(cleanup.deleted_rows, 3);
    assert!(maintenance.optimized);
    assert!(maintenance.vacuumed);
    assert!(maintenance.after.page_count > 0);
    assert!(maintenance.after.freelist_count <= maintenance.before.freelist_count);
    let second_cleanup = cleanup_at(&pool, now).await.unwrap();
    assert_eq!(second_cleanup.deleted_rows, 0);
    assert!(matches!(second_cleanup.maintenance, MaintenanceResult::NotNeeded));
    pool.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_retries_maintenance_after_committed_deletion_failure() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("rustzen-monitor-retry-{stamp}.db"));
    let pool = rustzen_storage::connect_sqlite_with_options(
        &rustzen_storage::database_url_from_path(&path),
        rustzen_storage::DatabaseConnectionOptions {
            max_connections: 1,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: None,
        },
    )
    .await
    .expect("open cleanup retry database");
    crate::infra::db::migrate(&pool).await.unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 0, 0, 0).unwrap();
    let old = now - ChronoDuration::days(30) - ChronoDuration::seconds(1);
    record_at(&pool, report("cleanup-retry-node", Uuid::new_v4(), 1, old, 10.0, 10, 10), old)
        .await
        .unwrap();
    for _ in 0..1_000 {
        sqlx::query("INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES('cleanup-retry-node',10,1,2,?)")
            .bind(old.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
    }
    let failed = cleanup_at_with(&pool, now, |_pool, _plan| {
        Box::pin(async {
            Err::<SqliteMaintenanceReport, _>(rustzen_storage::CoreError::InvalidInput(
                "injected maintenance failure".to_string(),
            ))
        })
    })
    .await;
    let failed = failed.expect("deletion succeeds even when maintenance fails");
    assert!(matches!(failed.maintenance, MaintenanceResult::Failed(_)));
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM resource_samples WHERE node_id='cleanup-retry-node'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    assert!(
        sqlx::query_scalar::<_, i64>("PRAGMA freelist_count").fetch_one(&pool).await.unwrap() > 0
    );
    let retry = cleanup_at(&pool, now).await.unwrap();
    assert_eq!(retry.deleted_rows, 0);
    assert!(matches!(retry.maintenance, MaintenanceResult::Succeeded(_)));
    pool.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn retention_removes_old_resolved_rows_but_keeps_active_state_and_counters() {
    let pool = migrated_test_pool().await;
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 8, 0, 0).unwrap();
    let old = now - ChronoDuration::days(30) - ChronoDuration::seconds(1);
    let node = report("retention-state-node", Uuid::new_v4(), 1, old, 10.0, 10, 10);
    record_at(&pool, node, old).await.unwrap();
    sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at,resolved_at) VALUES('resolved-old','retention-state-node','cpuHigh','cpu','resolved','old',?,?,?)")
        .bind(old.to_rfc3339())
        .bind(old.to_rfc3339())
        .bind(old.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES('active-old','retention-state-node','memoryHigh','memory','active','active',?,?)")
        .bind(old.to_rfc3339())
        .bind(old.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE alert_counters SET abnormal_count=2,normal_count=0 WHERE node_id='retention-state-node' AND kind='cpuHigh' AND target='cpu'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO node_daily_summaries(node_id,summary_date,sample_count,disk_summary_json,coverage_percent) VALUES('retention-state-node','2026-08-01',0,'{}',0)")
        .execute(&pool)
        .await
        .unwrap();
    cleanup_at(&pool, now).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE id='resolved-old'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE id='active-old'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM alert_counters WHERE node_id='retention-state-node'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        4
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM node_daily_summaries WHERE summary_date='2026-08-01'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}
