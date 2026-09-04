pub(super) use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(super) use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
pub(super) use rustzen_storage::{SqliteMaintenanceReport, SqlitePool};
pub(super) use sqlx::Row;
pub(super) use tokio::sync::Notify;
pub(super) use uuid::Uuid;

pub(super) use super::super::{
    acceptance::{LockHook, record_at_with_lock_hook},
    background::{
        MaintenanceResult, cleanup_at, cleanup_at_with, generate_daily_summaries_at,
        offline_scan_at, offline_scan_at_with_lock_hook,
    },
    policy::{
        Offline, SettingInput, Threshold, apply_node_settings, apply_settings, node_settings_value,
        reset_node_settings_for, settings_value,
    },
    queries::{aggregate_disk_points, aggregate_resource_points, query_window},
    record_at,
};
pub(super) use crate::{
    common::error::AppError,
    infra::db::migrated_test_pool,
    protocol::{AgentReport, AgentReportStatus, ByteUsage, DiskUsage},
};

pub(super) fn report(
    node: &str,
    boot_id: Uuid,
    sequence: u64,
    collected_at: DateTime<Utc>,
    cpu: f64,
    root: u64,
    data: u64,
) -> AgentReport {
    AgentReport {
        node_id: node.to_string(),
        boot_id,
        sequence,
        hostname: node.to_string(),
        agent_version: "test".to_string(),
        collected_at,
        cpu_percent: cpu,
        memory: ByteUsage { used_bytes: 50, total_bytes: 100 },
        disks: vec![
            DiskUsage {
                mount_point: "/".to_string(),
                usage: ByteUsage { used_bytes: root, total_bytes: 100 },
            },
            DiskUsage {
                mount_point: "/data".to_string(),
                usage: ByteUsage { used_bytes: data, total_bytes: 100 },
            },
        ],
    }
}

pub(super) async fn record(
    pool: &SqlitePool,
    report: AgentReport,
) -> Result<AgentReportStatus, AppError> {
    let received_at = report.collected_at;
    record_at(pool, report, received_at).await
}
