use super::support::*;

#[tokio::test]
async fn fencing_accepts_duplicate_and_stale_without_side_effects() {
    let pool = migrated_test_pool().await;
    let first_boot = Uuid::new_v4();
    let first_time = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    let first = report("fence-node", first_boot, 1, first_time, 10.0, 10, 10);
    assert_eq!(record(&pool, first.clone()).await.unwrap(), AgentReportStatus::Accepted);
    assert_eq!(record(&pool, first.clone()).await.unwrap(), AgentReportStatus::Duplicate);
    let mut stale = first.clone();
    stale.hostname = "mutated".to_string();
    assert_eq!(record(&pool, stale).await.unwrap(), AgentReportStatus::Duplicate);
    let mut older = first.clone();
    older.sequence = 0;
    assert!(record(&pool, older).await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource_samples")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM disk_samples")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );

    let second_boot = Uuid::new_v4();
    let takeover = report(
        "fence-node",
        second_boot,
        1,
        first_time + ChronoDuration::seconds(30),
        20.0,
        20,
        20,
    );
    assert_eq!(record(&pool, takeover).await.unwrap(), AgentReportStatus::Accepted);
    let retired =
        report("fence-node", first_boot, 2, first_time + ChronoDuration::seconds(60), 30.0, 30, 30);
    assert_eq!(record(&pool, retired).await.unwrap(), AgentReportStatus::Stale);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource_samples")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_boots WHERE retired_at IS NOT NULL"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn historical_duplicate_stale_and_retired_reports_bypass_clock_skew() {
    let pool = migrated_test_pool().await;
    let received = Utc::now();
    let historical = received - ChronoDuration::minutes(10);
    let first_boot = Uuid::new_v4();
    let first = report("historical-node", first_boot, 1, historical, 10.0, 10, 10);
    assert_eq!(
        record_at(&pool, first.clone(), historical).await.unwrap(),
        AgentReportStatus::Accepted
    );
    let second = report(
        "historical-node",
        first_boot,
        2,
        historical + ChronoDuration::seconds(30),
        10.0,
        10,
        10,
    );
    assert_eq!(
        record_at(&pool, second.clone(), second.collected_at).await.unwrap(),
        AgentReportStatus::Accepted
    );
    assert_eq!(record_at(&pool, second, received).await.unwrap(), AgentReportStatus::Duplicate);
    assert_eq!(record_at(&pool, first, received).await.unwrap(), AgentReportStatus::Stale);

    let second_boot = Uuid::new_v4();
    let takeover = report(
        "historical-node",
        second_boot,
        1,
        historical + ChronoDuration::seconds(60),
        10.0,
        10,
        10,
    );
    assert_eq!(
        record_at(&pool, takeover, historical + ChronoDuration::seconds(60)).await.unwrap(),
        AgentReportStatus::Accepted
    );
    let retired = report(
        "historical-node",
        first_boot,
        3,
        historical + ChronoDuration::seconds(90),
        10.0,
        10,
        10,
    );
    assert_eq!(record_at(&pool, retired, received).await.unwrap(), AgentReportStatus::Stale);

    let before_samples: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM resource_samples WHERE node_id='historical-node'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let outside_skew = report(
        "historical-node",
        second_boot,
        2,
        historical + ChronoDuration::seconds(90),
        10.0,
        10,
        10,
    );
    assert!(record_at(&pool, outside_skew, received).await.is_err());
    let after_samples: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM resource_samples WHERE node_id='historical-node'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(after_samples, before_samples);
}

#[tokio::test]
async fn three_high_then_three_normal_transitions_one_incident() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 1, 0, 0).unwrap();
    for sequence in 1..=3 {
        assert_eq!(
            record(
                &pool,
                report(
                    "alert-node",
                    boot,
                    sequence,
                    start + ChronoDuration::seconds(sequence as i64),
                    95.0,
                    10,
                    10
                )
            )
            .await
            .unwrap(),
            AgentReportStatus::Accepted
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE status='active' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    for sequence in 4..=6 {
        assert_eq!(
            record(
                &pool,
                report(
                    "alert-node",
                    boot,
                    sequence,
                    start + ChronoDuration::seconds(sequence as i64),
                    10.0,
                    10,
                    10
                )
            )
            .await
            .unwrap(),
            AgentReportStatus::Accepted
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE status='resolved' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_incidents WHERE kind='cpuHigh'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn disk_alert_targets_are_isolated_per_mount() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 2, 0, 0).unwrap();
    for sequence in 1..=3 {
        record_at(
            &pool,
            report(
                "disk-target-node",
                boot,
                sequence,
                start + ChronoDuration::seconds(sequence as i64),
                10.0,
                95,
                10,
            ),
            start + ChronoDuration::seconds(sequence as i64),
        )
        .await
        .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='diskHigh' AND target='/' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='diskHigh' AND target='/data' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    for sequence in 4..=6 {
        record_at(
            &pool,
            report(
                "disk-target-node",
                boot,
                sequence,
                start + ChronoDuration::seconds(sequence as i64),
                10.0,
                10,
                95,
            ),
            start + ChronoDuration::seconds(sequence as i64),
        )
        .await
        .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='diskHigh' AND target='/' AND status='resolved'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='diskHigh' AND target='/data' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn failed_report_transaction_does_not_leave_partial_node_state() {
    let pool = migrated_test_pool().await;
    sqlx::query("DROP TABLE disk_samples").execute(&pool).await.unwrap();
    let result = record_at(
        &pool,
        report("rollback-node", Uuid::new_v4(), 1, Utc::now(), 10.0, 10, 10),
        Utc::now(),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_nodes")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn older_collected_at_is_stale_even_with_an_increasing_sequence() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let received = Utc.with_ymd_and_hms(2026, 9, 2, 4, 0, 30).unwrap();
    let first_time = received;
    let second_time = received - ChronoDuration::seconds(1);
    assert_eq!(
        record_at(&pool, report("clock-node", boot, 1, first_time, 10.0, 10, 10), received)
            .await
            .unwrap(),
        AgentReportStatus::Accepted
    );
    assert_eq!(
        record_at(
            &pool,
            report("clock-node", boot, 2, second_time, 20.0, 20, 20),
            received + ChronoDuration::seconds(30),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    let last_received: String =
        sqlx::query_scalar("SELECT last_received_at FROM monitor_nodes WHERE node_id='clock-node'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(last_received, received.to_rfc3339());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM resource_samples WHERE node_id='clock-node'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn report_clock_skew_has_an_inclusive_five_minute_boundary() {
    use crate::protocol::MAX_REPORT_CLOCK_SKEW_SECONDS;

    let pool = migrated_test_pool().await;
    let received = Utc.with_ymd_and_hms(2026, 9, 2, 4, 30, 0).unwrap();
    let boot = Uuid::new_v4();
    for (sequence, offset) in
        [(1, -MAX_REPORT_CLOCK_SKEW_SECONDS), (2, MAX_REPORT_CLOCK_SKEW_SECONDS)]
    {
        let collected = received + ChronoDuration::seconds(offset);
        assert_eq!(
            record_at(
                &pool,
                report("skew-node", boot, sequence, collected, 10.0, 10, 10),
                received,
            )
            .await
            .unwrap(),
            AgentReportStatus::Accepted
        );
    }
    for (node, sequence, offset) in [
        ("future-node", 1, MAX_REPORT_CLOCK_SKEW_SECONDS + 1),
        ("past-node", 1, -MAX_REPORT_CLOCK_SKEW_SECONDS - 1),
    ] {
        let result = record_at(
            &pool,
            report(
                node,
                Uuid::new_v4(),
                sequence,
                received + ChronoDuration::seconds(offset),
                10.0,
                10,
                10,
            ),
            received,
        )
        .await;
        assert!(result.is_err());
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_nodes")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource_samples")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
}

#[tokio::test]
async fn threshold_boundary_applies_to_cpu_memory_and_each_disk() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 2, 5, 0, 0).unwrap();
    for sequence in 1..=3 {
        let mut input = report(
            "boundary-node",
            boot,
            sequence,
            start + ChronoDuration::seconds(sequence as i64),
            90.0,
            90,
            90,
        );
        input.memory = ByteUsage { used_bytes: 90, total_bytes: 100 };
        record(&pool, input).await.unwrap();
    }
    for (kind, target) in
        [("cpuHigh", "cpu"), ("memoryHigh", "memory"), ("diskHigh", "/"), ("diskHigh", "/data")]
    {
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_incidents WHERE kind=? AND target=? AND status='active'")
                .bind(kind)
                .bind(target)
                .fetch_one(&pool)
                .await
                .unwrap(),
            1,
            "missing boundary incident for {kind}/{target}"
        );
    }
}
