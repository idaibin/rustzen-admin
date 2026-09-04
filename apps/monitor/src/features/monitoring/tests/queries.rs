use super::support::*;

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
