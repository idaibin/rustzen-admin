use super::support::{TestDatabase, report};
use crate::notifications::runtime::validate_accounting;
use crate::{features::monitoring::record_at, protocol::AgentReportStatus};
use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;

#[tokio::test]
async fn accepted_transitions_enqueue_once_and_survive_reopen() {
    let database = TestDatabase::new().await;
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap();
    for sequence in 1..=3 {
        let at = start + Duration::seconds(sequence as i64);
        assert_eq!(
            record_at(&database.primary, report("node-a", boot, sequence, at, 95.0), at)
                .await
                .unwrap(),
            AgentReportStatus::Accepted
        );
    }
    let duplicate_at = start + Duration::seconds(3);
    assert_eq!(
        record_at(&database.primary, report("node-a", boot, 3, duplicate_at, 95.0), duplicate_at)
            .await
            .unwrap(),
        AgentReportStatus::Duplicate
    );
    assert_eq!(
        record_at(
            &database.primary,
            report("node-a", boot, 2, start + Duration::seconds(2), 95.0),
            start + Duration::seconds(4),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        1
    );

    for sequence in 4..=6 {
        let at = start + Duration::seconds(sequence as i64);
        record_at(&database.primary, report("node-a", boot, sequence, at, 10.0), at).await.unwrap();
    }
    let rows = sqlx::query_as::<_, (String, i64, String)>(
        "SELECT topic,subject_revision,subject_id FROM notification_outbox
         ORDER BY subject_revision",
    )
    .fetch_all(&database.peer)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, "monitor.incident.opened");
    assert_eq!(rows[0].1, 1);
    assert_eq!(rows[1].0, "monitor.incident.resolved");
    assert_eq!(rows[1].1, 2);
    assert_eq!(rows[0].2, rows[1].2);

    let reopened = database.reopen().await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT pending_count FROM notification_delivery_status")
            .fetch_one(&reopened)
            .await
            .unwrap(),
        2
    );
    reopened.close().await;
    database.close().await;
}

#[tokio::test]
async fn full_outbox_records_gap_without_rolling_back_incident() {
    let database = TestDatabase::new().await;
    sqlx::query(
        "UPDATE notification_delivery_status SET pending_count=100000,pending_bytes=67108864",
    )
    .execute(&database.primary)
    .await
    .unwrap();
    let boot = Uuid::new_v4();
    let start = Utc.with_ymd_and_hms(2026, 9, 7, 1, 0, 0).unwrap();
    for sequence in 1..=3 {
        let at = start + Duration::seconds(sequence as i64);
        record_at(&database.primary, report("node-full", boot, sequence, at, 95.0), at)
            .await
            .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitor_incidents WHERE node_id='node-full' AND status='active'",
        )
        .fetch_one(&database.peer)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT omitted_count FROM notification_delivery_status")
            .fetch_one(&database.peer)
            .await
            .unwrap(),
        1
    );
    database.close().await;
}

#[tokio::test]
async fn startup_accounting_validation_fails_closed_after_tamper() {
    let database = TestDatabase::new().await;
    validate_accounting(&database.primary).await.unwrap();
    sqlx::query("UPDATE notification_delivery_status SET pending_count=1")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(validate_accounting(&database.peer).await.is_err());
    database.close().await;
}
