use super::support::*;
use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::sync::Notify;

#[tokio::test]
async fn nodes_read_latest_disks_for_all_nodes_with_one_bounded_query() {
    let pool = migrated_test_pool().await;
    let start = Utc.with_ymd_and_hms(2026, 9, 5, 8, 0, 0).unwrap();
    let zulu_boot = Uuid::new_v4();
    let alpha_boot = Uuid::new_v4();
    let bravo_boot = Uuid::new_v4();

    for (node, boot, sequence, root, data) in [
        ("zulu", zulu_boot, 1, 10, 20),
        ("alpha", alpha_boot, 1, 30, 40),
        ("bravo", bravo_boot, 1, 50, 60),
        ("zulu", zulu_boot, 2, 70, 80),
    ] {
        record(
            &pool,
            report(
                node,
                boot,
                sequence,
                start + ChronoDuration::seconds(sequence as i64),
                sequence as f64,
                root,
                data,
            ),
        )
        .await
        .unwrap();
    }
    sqlx::query("UPDATE monitor_nodes SET hostname='a-host' WHERE node_id='zulu'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM disk_samples WHERE node_id='bravo'").execute(&pool).await.unwrap();

    let values = node_values(&pool, start + ChronoDuration::minutes(1)).await.unwrap();

    assert_eq!(
        values.iter().map(|value| value["nodeId"].as_str().unwrap()).collect::<Vec<_>>(),
        ["zulu", "alpha", "bravo"]
    );
    assert_eq!(values[0]["disks"][0]["usedBytes"], 70);
    assert_eq!(values[0]["disks"][1]["usedBytes"], 80);
    assert_eq!(values[1]["disks"][0]["usedBytes"], 30);
    assert_eq!(values[2]["disks"], serde_json::json!([]));
    assert!(LATEST_NODE_DISKS_SQL.contains("FROM monitor_nodes"));
    assert!(LATEST_NODE_DISKS_SQL.contains("CROSS JOIN disk_samples"));
    assert!(!LATEST_NODE_DISKS_SQL.contains("?"));
}

#[tokio::test]
async fn nodes_keep_node_state_and_disks_in_one_read_snapshot() {
    let path =
        std::env::temp_dir().join(format!("rustzen-monitor-nodes-snapshot-{}.db", Uuid::new_v4()));
    let options = SqliteConnectOptions::new().filename(&path).create_if_missing(true);
    let reader =
        SqlitePoolOptions::new().max_connections(1).connect_with(options.clone()).await.unwrap();
    migrate(&reader).await.unwrap();
    sqlx::query("PRAGMA journal_mode=WAL").execute(&reader).await.unwrap();
    let writer = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let start = Utc.with_ymd_and_hms(2026, 9, 5, 9, 0, 0).unwrap();
    let boot = Uuid::new_v4();
    record(&reader, report("snapshot-node", boot, 1, start, 10.0, 10, 20)).await.unwrap();

    let rows_read = Arc::new(Notify::new());
    let continue_read = Arc::new(Notify::new());
    let task = tokio::spawn({
        let reader = reader.clone();
        let rows_read = Arc::clone(&rows_read);
        let continue_read = Arc::clone(&continue_read);
        async move {
            node_values_after_rows(&reader, start, async move {
                rows_read.notify_one();
                continue_read.notified().await;
            })
            .await
        }
    });
    rows_read.notified().await;
    let next = start + ChronoDuration::seconds(30);
    record(&writer, report("snapshot-node", boot, 2, next, 20.0, 70, 80)).await.unwrap();
    continue_read.notify_one();
    let values = task.await.unwrap().unwrap();

    assert_eq!(values[0]["lastReportAt"], start.to_rfc3339());
    assert_eq!(values[0]["disks"][0]["collectedAt"], start.to_rfc3339());
    assert_eq!(values[0]["disks"][0]["usedBytes"], 10);

    reader.close().await;
    writer.close().await;
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

#[tokio::test]
async fn nodes_batch_disk_query_uses_latest_sample_index() {
    let pool = migrated_test_pool().await;
    let start = Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap();
    record(&pool, report("indexed-node", Uuid::new_v4(), 1, start, 10.0, 10, 20)).await.unwrap();
    for second in 1..=200 {
        sqlx::query(
            "INSERT INTO disk_samples(node_id,mount_point,used_bytes,total_bytes,collected_at)
             VALUES('indexed-node','/history',1,100,?)",
        )
        .bind((start - ChronoDuration::seconds(second)).to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    }
    // The production query is a compile-time constant; EXPLAIN requires a dynamic prefix.
    let plan =
        sqlx::query(sqlx::AssertSqlSafe(format!("EXPLAIN QUERY PLAN {LATEST_NODE_DISKS_SQL}")))
            .fetch_all(&pool)
            .await
            .unwrap();
    let details = plan.iter().map(|row| row.get::<String, _>("detail")).collect::<Vec<_>>();

    assert!(details.iter().any(|detail| detail.contains("idx_disk_samples_node_time_mount")));
    assert!(details.iter().any(|detail| detail.contains("node_id=? AND collected_at=?")));
}

#[tokio::test]
async fn metrics_bucket_and_daily_summary_keep_mounts_independent_and_idempotent() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 2, 0, 0).unwrap();
    for sequence in 1..=2 {
        record(
            &pool,
            report(
                "metrics-node",
                boot,
                sequence,
                start + ChronoDuration::minutes(sequence as i64),
                10.0 * sequence as f64,
                10 * sequence,
                20 * sequence,
            ),
        )
        .await
        .unwrap();
    }
    let rows = sqlx::query("SELECT collected_at,cpu_percent,memory_used_bytes,memory_total_bytes FROM resource_samples ORDER BY collected_at").fetch_all(&pool).await.unwrap();
    let points = aggregate_resource_points(rows, Some(300)).unwrap();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0]["cpuPercent"], 15.0);
    let disk_rows = sqlx::query("SELECT mount_point,collected_at,used_bytes,total_bytes FROM disk_samples ORDER BY mount_point,collected_at").fetch_all(&pool).await.unwrap();
    let disks = aggregate_disk_points(disk_rows, Some(300)).unwrap();
    assert_eq!(disks.len(), 2);
    assert_eq!(disks[0]["mountPoint"], "/");
    assert_eq!(disks[1]["mountPoint"], "/data");

    generate_daily_summaries_at(&pool, start.date_naive()).await.unwrap();
    generate_daily_summaries_at(&pool, start.date_naive()).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM node_daily_summaries")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    let summary: String = sqlx::query_scalar(
        "SELECT disk_summary_json FROM node_daily_summaries WHERE node_id='metrics-node'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let summary: serde_json::Value = serde_json::from_str(&summary).unwrap();
    assert_eq!(summary["/data"]["max"], 40.0);
    assert_eq!(summary["/"]["max"], 20.0);
}

#[tokio::test]
async fn metric_buckets_align_by_epoch_and_retain_independent_mount_series() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let first = Utc.with_ymd_and_hms(2026, 9, 2, 8, 4, 0).unwrap();
    for (sequence, at, cpu, root, data) in [
        (1, first, 10.0, 10, 20),
        (2, first + ChronoDuration::seconds(1), 20.0, 20, 30),
        (3, first + ChronoDuration::minutes(5), 40.0, 40, 50),
    ] {
        record_at(&pool, report("bucket-node", boot, sequence, at, cpu, root, data), at)
            .await
            .unwrap();
    }
    let rows = sqlx::query(
        "SELECT collected_at,cpu_percent,memory_used_bytes,memory_total_bytes
         FROM resource_samples WHERE node_id='bucket-node' ORDER BY collected_at",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let points = aggregate_resource_points(rows, Some(300)).unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0]["collectedAt"], "2026-09-02T08:00:00+00:00");
    assert_eq!(points[0]["cpuPercent"], 15.0);
    assert_eq!(points[1]["collectedAt"], "2026-09-02T08:05:00+00:00");
    assert_eq!(points[1]["cpuPercent"], 40.0);

    let disk_rows = sqlx::query(
        "SELECT mount_point,collected_at,used_bytes,total_bytes
         FROM disk_samples WHERE node_id='bucket-node' ORDER BY mount_point,collected_at",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let disks = aggregate_disk_points(disk_rows, Some(300)).unwrap();
    assert_eq!(disks.len(), 2);
    assert_eq!(disks[0]["mountPoint"], "/");
    assert_eq!(disks[0]["points"][0]["percent"], 15.0);
    assert_eq!(disks[1]["mountPoint"], "/data");
    assert_eq!(disks[1]["points"][1]["percent"], 50.0);
    assert!(query_window(Some(first), Some(first - ChronoDuration::seconds(1))).is_err());
    assert!(query_window(Some(first - ChronoDuration::days(31)), Some(first)).is_err());
}

#[tokio::test]
async fn daily_summary_includes_registered_node_with_zero_samples() {
    let pool = migrated_test_pool().await;
    let collected = Utc.with_ymd_and_hms(2026, 9, 1, 23, 59, 0).unwrap();
    let received = Utc.with_ymd_and_hms(2026, 9, 2, 0, 1, 0).unwrap();
    record_at(
        &pool,
        report("offline-summary-node", Uuid::new_v4(), 1, collected, 10.0, 10, 10),
        received,
    )
    .await
    .unwrap();
    generate_daily_summaries_at(&pool, received.date_naive()).await.unwrap();
    let row = sqlx::query(
        "SELECT sample_count,coverage_percent,cpu_min,memory_avg,disk_summary_json
         FROM node_daily_summaries WHERE node_id='offline-summary-node' AND summary_date='2026-09-02'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.get::<i64, _>("sample_count"), 0);
    assert_eq!(row.get::<f64, _>("coverage_percent"), 0.0);
    assert!(row.get::<Option<f64>, _>("cpu_min").is_none());
    assert!(row.get::<Option<f64>, _>("memory_avg").is_none());
    assert_eq!(row.get::<String, _>("disk_summary_json"), "{}");
}

#[tokio::test]
async fn daily_summary_candidate_respects_registration_boundary() {
    let pool = migrated_test_pool().await;
    let day = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    let before_day = day - ChronoDuration::hours(1);
    record_at(
        &pool,
        report("before-day-node", Uuid::new_v4(), 1, before_day, 10.0, 10, 10),
        before_day,
    )
    .await
    .unwrap();
    let after_day = day + ChronoDuration::days(1);
    record_at(
        &pool,
        report("after-day-node", Uuid::new_v4(), 1, after_day, 10.0, 10, 10),
        after_day,
    )
    .await
    .unwrap();

    generate_daily_summaries_at(&pool, day.date_naive()).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM node_daily_summaries WHERE summary_date='2026-09-01' AND node_id='before-day-node'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM node_daily_summaries WHERE summary_date='2026-09-01' AND node_id='after-day-node'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn daily_summary_keeps_cross_day_offline_participant() {
    let pool = migrated_test_pool().await;
    let day = Utc.with_ymd_and_hms(2026, 9, 1, 23, 0, 0).unwrap();
    let boot = Uuid::new_v4();
    record_at(&pool, report("cross-day-node", boot, 1, day, 10.0, 10, 10), day).await.unwrap();
    let next_day = day + ChronoDuration::hours(2);
    offline_scan_at(&pool, next_day).await.unwrap();
    generate_daily_summaries_at(&pool, day.date_naive()).await.unwrap();
    generate_daily_summaries_at(&pool, next_day.date_naive()).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM node_daily_summaries WHERE node_id='cross-day-node'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        2
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT incident_count FROM node_daily_summaries WHERE node_id='cross-day-node' AND summary_date='2026-09-02'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT offline_seconds FROM node_daily_summaries WHERE node_id='cross-day-node' AND summary_date='2026-09-02'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        23 * 60 * 60
    );
}
