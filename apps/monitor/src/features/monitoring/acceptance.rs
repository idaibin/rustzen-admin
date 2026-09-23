use axum::extract::State;
use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use sqlx::Row;
use tokio::sync::Notify;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
    middleware::require_agent_token,
    protocol::{AgentReport, AgentReportStatus, AgentResponseData, MAX_AGENT_REPORT_BODY_BYTES},
};

use super::{
    policy::{effective_settings_row, evaluate, resolve},
    queries::parse_utc,
};

#[derive(Clone)]
pub(super) struct LockHook {
    pub(super) acquired: std::sync::Arc<Notify>,
    pub(super) release: std::sync::Arc<Notify>,
}

pub async fn submit(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<AgentResponseData> {
    require_agent_token(request.headers(), state.agent_token.as_ref())?;
    let body = axum::body::to_bytes(request.into_body(), MAX_AGENT_REPORT_BODY_BYTES)
        .await
        .map_err(|_| AppError::unprocessable("invalid agent report body"))?;
    let report = serde_json::from_slice::<AgentReport>(&body)
        .map_err(|error| AppError::unprocessable(format!("invalid agent report: {error}")))?;
    let received_at = Utc::now();
    report.validate().map_err(|e| AppError::unprocessable(e.to_string()))?;
    let status = record_at(&state.pool, report, received_at).await?;
    if status == AgentReportStatus::Accepted {
        state.nodes_cache.invalidate().await;
    }
    Ok(ApiResponse::success(AgentResponseData { status }))
}

pub(crate) async fn record_at(
    pool: &SqlitePool,
    report: AgentReport,
    received_at: DateTime<Utc>,
) -> Result<AgentReportStatus, AppError> {
    record_at_inner(pool, report, received_at, None).await
}

#[cfg(test)]
pub(super) async fn record_at_with_lock_hook(
    pool: &SqlitePool,
    report: AgentReport,
    received_at: DateTime<Utc>,
    lock_hook: LockHook,
) -> Result<AgentReportStatus, AppError> {
    record_at_inner(pool, report, received_at, Some(lock_hook)).await
}

async fn record_at_inner(
    pool: &SqlitePool,
    report: AgentReport,
    received_at: DateTime<Utc>,
    lock_hook: Option<LockHook>,
) -> Result<AgentReportStatus, AppError> {
    report.validate().map_err(|e| AppError::unprocessable(e.to_string()))?;
    if report.sequence == 0 {
        return Ok(AgentReportStatus::Stale);
    }
    if report.sequence > i64::MAX as u64 {
        return Err(AppError::unprocessable("sequence is outside the supported range"));
    }
    let now = received_at.to_rfc3339();
    let collected = report.collected_at.to_rfc3339();
    let boot = report.boot_id.to_string();
    // Fencing is decided from the current row, so serialize report decisions
    // before reading it. Without an immediate SQLite write lock, two first
    // reports can both observe an absent node and one loses with a 500.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if let Some(lock_hook) = lock_hook {
        lock_hook.acquired.notify_one();
        lock_hook.release.notified().await;
    }
    let row = sqlx::query(
        "SELECT current_boot_id,last_sequence,last_report_at FROM monitor_nodes WHERE node_id=?",
    )
    .bind(&report.node_id)
    .fetch_optional(&mut *tx)
    .await?;
    let mut takeover = false;
    if let Some(row) = row {
        let current: String = row.get(0);
        let last: i64 = row.get(1);
        let last_report_at: String = row.get(2);
        if current == boot {
            if report.sequence == last as u64 {
                return Ok(AgentReportStatus::Duplicate);
            }
            if report.sequence < last as u64 {
                return Ok(AgentReportStatus::Stale);
            }
            if report.collected_at <= parse_utc(&last_report_at)? {
                return Ok(AgentReportStatus::Stale);
            }
        } else {
            let retired: Option<i64> = sqlx::query_scalar("SELECT 1 FROM monitor_boots WHERE node_id=? AND boot_id=? AND retired_at IS NOT NULL").bind(&report.node_id).bind(&boot).fetch_optional(&mut *tx).await?;
            if retired.is_some() || report.sequence != 1 {
                return Ok(AgentReportStatus::Stale);
            }
            if report.collected_at <= parse_utc(&last_report_at)? {
                return Ok(AgentReportStatus::Stale);
            }
            takeover = true;
        }
    } else if report.sequence != 1 {
        return Ok(AgentReportStatus::Stale);
    }
    report.validate_at(received_at).map_err(|e| AppError::unprocessable(e.to_string()))?;
    if takeover {
        let current: String =
            sqlx::query_scalar("SELECT current_boot_id FROM monitor_nodes WHERE node_id=?")
                .bind(&report.node_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query("UPDATE monitor_boots SET retired_at=? WHERE node_id=? AND boot_id=?")
            .bind(&now)
            .bind(&report.node_id)
            .bind(current)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO monitor_nodes(node_id,hostname,agent_version,current_boot_id,last_sequence,last_report_at,last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(node_id) DO UPDATE SET hostname=excluded.hostname,agent_version=excluded.agent_version,current_boot_id=excluded.current_boot_id,last_sequence=excluded.last_sequence,last_report_at=excluded.last_report_at,last_received_at=excluded.last_received_at,cpu_percent=excluded.cpu_percent,memory_used_bytes=excluded.memory_used_bytes,memory_total_bytes=excluded.memory_total_bytes,updated_at=excluded.updated_at")
        .bind(&report.node_id).bind(&report.hostname).bind(&report.agent_version).bind(&boot).bind(report.sequence as i64).bind(&collected).bind(&now).bind(report.cpu_percent).bind(report.memory.used_bytes as i64).bind(report.memory.total_bytes as i64).bind(&now).bind(&now).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO monitor_boots(node_id,boot_id,last_sequence,retired_at) VALUES(?,?,?,NULL) ON CONFLICT(node_id,boot_id) DO UPDATE SET last_sequence=excluded.last_sequence").bind(&report.node_id).bind(&boot).bind(report.sequence as i64).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES(?,?,?,?,?)").bind(&report.node_id).bind(report.cpu_percent).bind(report.memory.used_bytes as i64).bind(report.memory.total_bytes as i64).bind(&collected).execute(&mut *tx).await?;
    for disk in &report.disks {
        sqlx::query("INSERT INTO disk_samples(node_id,mount_point,used_bytes,total_bytes,collected_at) VALUES(?,?,?,?,?)").bind(&report.node_id).bind(&disk.mount_point).bind(disk.usage.used_bytes as i64).bind(disk.usage.total_bytes as i64).bind(&collected).execute(&mut *tx).await?;
    }
    resolve(&mut tx, &report.node_id, "nodeOffline", "node", &now, "report received").await?;
    let s = effective_settings_row(&mut tx, &report.node_id).await?;
    evaluate(
        &mut tx,
        &report.node_id,
        "cpuHigh",
        "cpu",
        report.cpu_percent,
        s.cpu_enabled,
        s.cpu_threshold_percent,
        &now,
    )
    .await?;
    evaluate(
        &mut tx,
        &report.node_id,
        "memoryHigh",
        "memory",
        report.memory.used_bytes as f64 * 100.0 / report.memory.total_bytes as f64,
        s.memory_enabled,
        s.memory_threshold_percent,
        &now,
    )
    .await?;
    for d in &report.disks {
        evaluate(
            &mut tx,
            &report.node_id,
            "diskHigh",
            &d.mount_point,
            d.usage.used_bytes as f64 * 100.0 / d.usage.total_bytes as f64,
            s.disk_enabled,
            s.disk_threshold_percent,
            &now,
        )
        .await?;
    }
    tx.commit().await?;
    #[cfg(feature = "notifications")]
    crate::notifications::diagnostics::warn_after_commit(pool).await;
    Ok(AgentReportStatus::Accepted)
}
