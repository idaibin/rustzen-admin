use axum::extract::{Path, State};
use chrono::Utc;
use rustzen_ipc::ModuleJson;
use rustzen_storage::SqlitePool;
use sqlx::{Sqlite, Transaction};

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

mod evaluation;
mod settings;
pub(super) use evaluation::{evaluate, resolve};
pub(crate) use settings::SettingInput;
use settings::settings_row;
#[cfg(test)]
pub(super) use settings::{Offline, Threshold};
pub(super) use settings::{
    Settings, effective_settings_row, node_settings_value, settings, settings_value,
};

pub async fn update_settings(
    State(s): State<AppState>,
    ModuleJson(i): ModuleJson<SettingInput>,
) -> AppResult<serde_json::Value> {
    apply_settings(&s.pool, i).await?;
    s.nodes_cache.invalidate().await;
    Ok(ApiResponse::success(settings_value(&s.pool).await?))
}

pub(super) async fn apply_settings(pool: &SqlitePool, i: SettingInput) -> Result<(), AppError> {
    validate_setting_input(&i)?;
    let now = Utc::now().to_rfc3339();
    // Settings changes and offline scans must share the same SQLite writer
    // boundary, so the snapshot used for incident transitions cannot race a
    // report or scan that is already updating the monitoring state.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let previous = settings_row(&mut tx).await?;
    sqlx::query("UPDATE alert_settings SET cpu_enabled=?,cpu_threshold_percent=?,memory_enabled=?,memory_threshold_percent=?,disk_enabled=?,disk_threshold_percent=?,offline_enabled=?,offline_after_seconds=?,updated_at=? WHERE id=1").bind(i.cpu.enabled).bind(i.cpu.threshold_percent).bind(i.memory.enabled).bind(i.memory.threshold_percent).bind(i.disk.enabled).bind(i.disk.threshold_percent).bind(i.offline.enabled).bind(i.offline.after_seconds).bind(&now).execute(&mut *tx).await?;
    reconcile_setting_changes(&mut tx, &previous, &settings_from_input(&i), None, &now).await?;
    tx.commit().await?;
    #[cfg(feature = "notifications")]
    crate::notifications::diagnostics::warn_after_commit(pool).await;
    Ok(())
}

pub async fn node_settings(
    State(s): State<AppState>,
    Path(node_id): Path<String>,
) -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(node_settings_value(&s.pool, &node_id).await?))
}

pub async fn update_node_settings(
    State(s): State<AppState>,
    Path(node_id): Path<String>,
    ModuleJson(input): ModuleJson<SettingInput>,
) -> AppResult<serde_json::Value> {
    apply_node_settings(&s.pool, &node_id, input).await?;
    s.nodes_cache.invalidate().await;
    Ok(ApiResponse::success(node_settings_value(&s.pool, &node_id).await?))
}

pub async fn reset_node_settings(
    State(s): State<AppState>,
    Path(node_id): Path<String>,
) -> AppResult<serde_json::Value> {
    reset_node_settings_for(&s.pool, &node_id).await?;
    s.nodes_cache.invalidate().await;
    Ok(ApiResponse::success(node_settings_value(&s.pool, &node_id).await?))
}

fn validate_setting_input(input: &SettingInput) -> Result<(), AppError> {
    if !(1.0..=100.0).contains(&input.cpu.threshold_percent)
        || !(1.0..=100.0).contains(&input.memory.threshold_percent)
        || !(1.0..=100.0).contains(&input.disk.threshold_percent)
        || !(30..=3600).contains(&input.offline.after_seconds)
    {
        return Err(AppError::unprocessable("invalid alert setting"));
    }
    Ok(())
}

fn settings_from_input(input: &SettingInput) -> Settings {
    Settings {
        cpu_enabled: input.cpu.enabled as i64,
        cpu_threshold_percent: input.cpu.threshold_percent,
        memory_enabled: input.memory.enabled as i64,
        memory_threshold_percent: input.memory.threshold_percent,
        disk_enabled: input.disk.enabled as i64,
        disk_threshold_percent: input.disk.threshold_percent,
        offline_enabled: input.offline.enabled as i64,
        offline_after_seconds: input.offline.after_seconds,
    }
}

pub(super) async fn apply_node_settings(
    pool: &SqlitePool,
    node_id: &str,
    input: SettingInput,
) -> Result<(), AppError> {
    validate_setting_input(&input)?;
    let now = Utc::now().to_rfc3339();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    ensure_node_exists(&mut tx, node_id).await?;
    let previous = effective_settings_row(&mut tx, node_id).await?;
    sqlx::query(
        "INSERT INTO node_alert_settings
         (node_id,cpu_enabled,cpu_threshold_percent,memory_enabled,memory_threshold_percent,
          disk_enabled,disk_threshold_percent,offline_enabled,offline_after_seconds,updated_at)
         VALUES(?,?,?,?,?,?,?,?,?,?)
         ON CONFLICT(node_id) DO UPDATE SET
          cpu_enabled=excluded.cpu_enabled,cpu_threshold_percent=excluded.cpu_threshold_percent,
          memory_enabled=excluded.memory_enabled,memory_threshold_percent=excluded.memory_threshold_percent,
          disk_enabled=excluded.disk_enabled,disk_threshold_percent=excluded.disk_threshold_percent,
          offline_enabled=excluded.offline_enabled,offline_after_seconds=excluded.offline_after_seconds,
          updated_at=excluded.updated_at",
    )
    .bind(node_id)
    .bind(input.cpu.enabled)
    .bind(input.cpu.threshold_percent)
    .bind(input.memory.enabled)
    .bind(input.memory.threshold_percent)
    .bind(input.disk.enabled)
    .bind(input.disk.threshold_percent)
    .bind(input.offline.enabled)
    .bind(input.offline.after_seconds)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    reconcile_setting_changes(
        &mut tx,
        &previous,
        &settings_from_input(&input),
        Some(node_id),
        &now,
    )
    .await?;
    tx.commit().await?;
    #[cfg(feature = "notifications")]
    crate::notifications::diagnostics::warn_after_commit(pool).await;
    Ok(())
}

pub(super) async fn reset_node_settings_for(
    pool: &SqlitePool,
    node_id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    ensure_node_exists(&mut tx, node_id).await?;
    let previous = effective_settings_row(&mut tx, node_id).await?;
    let deleted = sqlx::query("DELETE FROM node_alert_settings WHERE node_id=?")
        .bind(node_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if deleted > 0 {
        let inherited = settings_row(&mut tx).await?;
        reconcile_setting_changes(&mut tx, &previous, &inherited, Some(node_id), &now).await?;
    }
    tx.commit().await?;
    #[cfg(feature = "notifications")]
    crate::notifications::diagnostics::warn_after_commit(pool).await;
    Ok(())
}

async fn ensure_node_exists(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
) -> Result<(), AppError> {
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM monitor_nodes WHERE node_id=?")
        .bind(node_id)
        .fetch_optional(&mut **tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("node"));
    }
    Ok(())
}

async fn reconcile_setting_changes(
    tx: &mut Transaction<'_, Sqlite>,
    previous: &Settings,
    next: &Settings,
    node_id: Option<&str>,
    now: &str,
) -> Result<(), sqlx::Error> {
    for (kind, enabled, changed) in [
        (
            "cpuHigh",
            next.cpu_enabled != 0,
            previous.cpu_enabled != next.cpu_enabled
                || previous.cpu_threshold_percent != next.cpu_threshold_percent,
        ),
        (
            "memoryHigh",
            next.memory_enabled != 0,
            previous.memory_enabled != next.memory_enabled
                || previous.memory_threshold_percent != next.memory_threshold_percent,
        ),
        (
            "diskHigh",
            next.disk_enabled != 0,
            previous.disk_enabled != next.disk_enabled
                || previous.disk_threshold_percent != next.disk_threshold_percent,
        ),
        (
            "nodeOffline",
            next.offline_enabled != 0,
            previous.offline_enabled != next.offline_enabled
                || previous.offline_after_seconds != next.offline_after_seconds,
        ),
    ] {
        if !changed {
            continue;
        }
        if let Some(node_id) = node_id {
            sqlx::query("UPDATE alert_counters SET abnormal_count=0,normal_count=0 WHERE node_id=? AND kind=?")
                .bind(node_id)
                .bind(kind)
                .execute(&mut **tx)
                .await?;
            if !enabled {
                #[cfg(feature = "notifications")]
                {
                    let ids = sqlx::query_scalar::<_, String>(
                        "SELECT id FROM monitor_incidents WHERE node_id=? AND kind=? AND status='active' ORDER BY id",
                    ).bind(node_id).bind(kind).fetch_all(&mut **tx).await?;
                    for id in ids {
                        crate::notifications::outbox::resolve_id(tx, &id, now, "setting disabled")
                            .await?;
                    }
                }
                #[cfg(not(feature = "notifications"))]
                sqlx::query("UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason='setting disabled' WHERE node_id=? AND kind=? AND status='active'")
                    .bind(now)
                    .bind(node_id)
                    .bind(kind)
                    .execute(&mut **tx)
                    .await?;
            }
        } else {
            sqlx::query("UPDATE alert_counters SET abnormal_count=0,normal_count=0 WHERE kind=? AND NOT EXISTS (SELECT 1 FROM node_alert_settings s WHERE s.node_id=alert_counters.node_id)")
                .bind(kind)
                .execute(&mut **tx)
                .await?;
            if !enabled {
                #[cfg(feature = "notifications")]
                {
                    let ids = sqlx::query_scalar::<_, String>(
                        "SELECT id FROM monitor_incidents WHERE kind=? AND status='active'
                         AND NOT EXISTS (SELECT 1 FROM node_alert_settings s WHERE s.node_id=monitor_incidents.node_id)
                         ORDER BY id",
                    ).bind(kind).fetch_all(&mut **tx).await?;
                    for id in ids {
                        crate::notifications::outbox::resolve_id(tx, &id, now, "setting disabled")
                            .await?;
                    }
                }
                #[cfg(not(feature = "notifications"))]
                sqlx::query("UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason='setting disabled' WHERE kind=? AND status='active' AND NOT EXISTS (SELECT 1 FROM node_alert_settings s WHERE s.node_id=monitor_incidents.node_id)")
                    .bind(now)
                    .bind(kind)
                    .execute(&mut **tx)
                    .await?;
            }
        }
    }
    Ok(())
}
