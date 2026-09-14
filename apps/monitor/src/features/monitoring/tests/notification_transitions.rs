use super::support::*;

#[tokio::test]
async fn policy_and_background_transitions_enqueue_from_real_entrypoints() {
    let pool = migrated_test_pool().await;
    // Anchor to the real clock minus one hour: apply_settings reconciles with
    // Utc::now(), so absolute fixture dates would expire the seven-day outbox
    // window once real time moves a week past them.
    let start = Utc::now() - ChronoDuration::hours(1);
    let boot = Uuid::new_v4();
    for sequence in 1..=3 {
        let at = start + ChronoDuration::seconds(sequence as i64);
        record_at(&pool, report("policy-node", boot, sequence, at, 95.0, 10, 10), at)
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

    let reported = start + ChronoDuration::minutes(10);
    record_at(&pool, report("offline-node", Uuid::new_v4(), 1, reported, 10.0, 10, 10), reported)
        .await
        .unwrap();
    offline_scan_at(&pool, reported + ChronoDuration::seconds(91)).await.unwrap();
    let recovered = reported + ChronoDuration::seconds(122);
    record_at(&pool, report("offline-node", Uuid::new_v4(), 1, recovered, 10.0, 10, 10), recovered)
        .await
        .unwrap();

    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT i.kind,
                SUM(CASE WHEN o.subject_revision=1 THEN 1 ELSE 0 END),
                SUM(CASE WHEN o.subject_revision=2 THEN 1 ELSE 0 END)
         FROM monitor_incidents i JOIN notification_outbox o ON o.subject_id=i.id
         WHERE i.node_id IN ('policy-node','offline-node')
         GROUP BY i.kind ORDER BY i.kind",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows, [("cpuHigh".into(), 1, 1), ("nodeOffline".into(), 2, 1)]);
}
