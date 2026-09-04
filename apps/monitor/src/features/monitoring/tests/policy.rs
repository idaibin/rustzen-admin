use super::support::*;

#[tokio::test]
async fn disabling_cpu_resolves_only_cpu_incidents_and_resets_its_counters() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 1, 30, 0).unwrap();
    for sequence in 1..=3 {
        record(
            &pool,
            report(
                "settings-node",
                boot,
                sequence,
                start + ChronoDuration::seconds(sequence as i64),
                95.0,
                10,
                10,
            ),
        )
        .await
        .unwrap();
    }
    apply_settings(
        &pool,
        SettingInput {
            cpu: Threshold { enabled: false, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='cpuHigh' AND status='active'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='cpuHigh' AND resolution_reason='setting disabled'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM alert_counters WHERE kind='cpuHigh' AND abnormal_count=0 AND normal_count=0",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='diskHigh'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn node_policy_overrides_global_changes_and_reset_restores_dynamic_inheritance() {
    let pool = migrated_test_pool().await;
    let at = Utc.with_ymd_and_hms(2026, 9, 1, 1, 45, 0).unwrap();
    for node_id in ["inherited-node", "custom-node"] {
        record_at(&pool, report(node_id, Uuid::new_v4(), 1, at, 10.0, 10, 10), at).await.unwrap();
    }

    apply_node_settings(
        &pool,
        "custom-node",
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 99.0 },
            memory: Threshold { enabled: true, threshold_percent: 91.0 },
            disk: Threshold { enabled: true, threshold_percent: 92.0 },
            offline: Offline { enabled: true, after_seconds: 120 },
        },
    )
    .await
    .unwrap();
    for node_id in ["inherited-node", "custom-node"] {
        sqlx::query(
            "UPDATE alert_counters SET abnormal_count=2 WHERE node_id=? AND kind='cpuHigh'",
        )
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    apply_settings(
        &pool,
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 80.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
    )
    .await
    .unwrap();

    let inherited = node_settings_value(&pool, "inherited-node").await.unwrap();
    assert_eq!(inherited["source"], "global");
    assert_eq!(inherited["isCustom"], false);
    assert_eq!(inherited["cpu"]["thresholdPercent"], 80.0);
    let custom = node_settings_value(&pool, "custom-node").await.unwrap();
    assert_eq!(custom["source"], "custom");
    assert_eq!(custom["isCustom"], true);
    assert_eq!(custom["cpu"]["thresholdPercent"], 99.0);
    assert_eq!(custom["offline"]["afterSeconds"], 120);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT abnormal_count FROM alert_counters WHERE node_id='inherited-node' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT abnormal_count FROM alert_counters WHERE node_id='custom-node' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        2
    );

    reset_node_settings_for(&pool, "custom-node").await.unwrap();
    let reset = node_settings_value(&pool, "custom-node").await.unwrap();
    assert_eq!(reset["source"], "global");
    assert_eq!(reset["isCustom"], false);
    assert_eq!(reset["cpu"]["thresholdPercent"], 80.0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT abnormal_count FROM alert_counters WHERE node_id='custom-node' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM node_alert_settings WHERE node_id='custom-node'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn effective_node_policy_controls_resource_and_offline_alerts() {
    let pool = migrated_test_pool().await;
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 1, 50, 0).unwrap();
    let mut boots = BTreeMap::new();
    for node_id in ["global-resource", "custom-resource", "global-offline", "custom-offline"] {
        let boot = Uuid::new_v4();
        boots.insert(node_id, boot);
        record_at(&pool, report(node_id, boot, 1, start, 10.0, 10, 10), start).await.unwrap();
    }
    for node_id in ["custom-resource", "custom-offline"] {
        apply_node_settings(
            &pool,
            node_id,
            SettingInput {
                cpu: Threshold { enabled: true, threshold_percent: 99.0 },
                memory: Threshold { enabled: true, threshold_percent: 90.0 },
                disk: Threshold { enabled: true, threshold_percent: 90.0 },
                offline: Offline { enabled: true, after_seconds: 120 },
            },
        )
        .await
        .unwrap();
    }
    for sequence in 2..=4 {
        for node_id in ["global-resource", "custom-resource"] {
            let collected = start + ChronoDuration::seconds(sequence as i64);
            record_at(
                &pool,
                report(node_id, boots[node_id], sequence, collected, 95.0, 10, 10),
                collected,
            )
            .await
            .unwrap();
        }
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='global-resource' AND kind='cpuHigh' AND status='active'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='custom-resource' AND kind='cpuHigh'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );

    offline_scan_at(&pool, start + ChronoDuration::seconds(100)).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='global-offline' AND kind='nodeOffline' AND status='active'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='custom-offline' AND kind='nodeOffline'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn global_disable_resolves_only_inherited_alerts_until_custom_policy_is_reset() {
    let pool = migrated_test_pool().await;
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 1, 55, 0).unwrap();
    let inherited_boot = Uuid::new_v4();
    let custom_boot = Uuid::new_v4();
    for (node_id, boot) in [("inherited-disable", inherited_boot), ("custom-disable", custom_boot)]
    {
        record_at(&pool, report(node_id, boot, 1, start, 10.0, 10, 10), start).await.unwrap();
    }
    apply_node_settings(
        &pool,
        "custom-disable",
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
    )
    .await
    .unwrap();
    for sequence in 2..=4 {
        for (node_id, boot) in
            [("inherited-disable", inherited_boot), ("custom-disable", custom_boot)]
        {
            let collected = start + ChronoDuration::seconds(sequence as i64);
            record_at(&pool, report(node_id, boot, sequence, collected, 95.0, 10, 10), collected)
                .await
                .unwrap();
        }
    }

    apply_settings(
        &pool,
        SettingInput {
            cpu: Threshold { enabled: false, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='inherited-disable' AND kind='cpuHigh' AND status='resolved' AND resolution_reason='setting disabled'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='custom-disable' AND kind='cpuHigh' AND status='active'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );

    reset_node_settings_for(&pool, "custom-disable").await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='custom-disable' AND kind='cpuHigh' AND status='resolved' AND resolution_reason='setting disabled'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn disabled_offline_alert_does_not_create_incident() {
    let pool = migrated_test_pool().await;
    let boot = Uuid::new_v4();
    let reported = Utc.with_ymd_and_hms(2026, 9, 2, 4, 0, 0).unwrap();
    record_at(&pool, report("disabled-offline", boot, 1, reported, 10.0, 10, 10), reported)
        .await
        .unwrap();
    apply_settings(
        &pool,
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: false, after_seconds: 90 },
        },
    )
    .await
    .unwrap();
    offline_scan_at(&pool, reported + ChronoDuration::hours(1)).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE kind='nodeOffline'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn invalid_alert_settings_are_rejected_without_partial_update() {
    let pool = migrated_test_pool().await;
    let invalid_inputs = [
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 0.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 101.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: f64::NAN },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 90 },
        },
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 29 },
        },
        SettingInput {
            cpu: Threshold { enabled: true, threshold_percent: 90.0 },
            memory: Threshold { enabled: true, threshold_percent: 90.0 },
            disk: Threshold { enabled: true, threshold_percent: 90.0 },
            offline: Offline { enabled: true, after_seconds: 3601 },
        },
    ];
    for invalid in invalid_inputs {
        assert!(apply_settings(&pool, invalid).await.is_err());
    }
    let settings = settings_value(&pool).await.unwrap();
    assert_eq!(settings["cpu"]["enabled"], true);
    assert_eq!(settings["cpu"]["thresholdPercent"], 90.0);
    assert_eq!(settings["offline"]["afterSeconds"], 90);
}
