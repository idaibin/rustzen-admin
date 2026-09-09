use axum::extract::{Path, State};
use chrono::Utc;
use rustzen_ipc::ModuleJson;
use rustzen_storage::SqlitePool;
use serde::Deserialize;
use sqlx::{Row, Sqlite, Transaction};
#[cfg(not(feature = "notifications"))]
use uuid::Uuid;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub(super) struct Settings {
    pub(super) cpu_enabled: i64,
    pub(super) cpu_threshold_percent: f64,
    pub(super) memory_enabled: i64,
    pub(super) memory_threshold_percent: f64,
    pub(super) disk_enabled: i64,
    pub(super) disk_threshold_percent: f64,
    pub(super) offline_enabled: i64,
    pub(super) offline_after_seconds: i64,
}
async fn settings_row(tx: &mut Transaction<'_, Sqlite>) -> Result<Settings, sqlx::Error> {
    sqlx::query_as("SELECT cpu_enabled,cpu_threshold_percent,memory_enabled,memory_threshold_percent,disk_enabled,disk_threshold_percent,offline_enabled,offline_after_seconds FROM alert_settings WHERE id=1").fetch_one(&mut **tx).await
}

pub(super) async fn effective_settings_row(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
) -> Result<Settings, sqlx::Error> {
    sqlx::query_as(
        "SELECT
            COALESCE(n.cpu_enabled,g.cpu_enabled) AS cpu_enabled,
            COALESCE(n.cpu_threshold_percent,g.cpu_threshold_percent) AS cpu_threshold_percent,
            COALESCE(n.memory_enabled,g.memory_enabled) AS memory_enabled,
            COALESCE(n.memory_threshold_percent,g.memory_threshold_percent) AS memory_threshold_percent,
            COALESCE(n.disk_enabled,g.disk_enabled) AS disk_enabled,
            COALESCE(n.disk_threshold_percent,g.disk_threshold_percent) AS disk_threshold_percent,
            COALESCE(n.offline_enabled,g.offline_enabled) AS offline_enabled,
            COALESCE(n.offline_after_seconds,g.offline_after_seconds) AS offline_after_seconds
         FROM alert_settings g LEFT JOIN node_alert_settings n ON n.node_id=? WHERE g.id=1",
    )
    .bind(node_id)
    .fetch_one(&mut **tx)
    .await
}
#[expect(
    clippy::too_many_arguments,
    reason = "evaluation needs the report target and configured threshold"
)]
pub(super) async fn evaluate(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    value: f64,
    enabled: i64,
    threshold: f64,
    now: &str,
) -> Result<(), sqlx::Error> {
    if enabled == 0 {
        return Ok(());
    }
    let high = value >= threshold;
    sqlx::query("INSERT INTO alert_counters(node_id,kind,target,abnormal_count,normal_count) VALUES(?,?,?,?,?) ON CONFLICT(node_id,kind,target) DO UPDATE SET abnormal_count=CASE WHEN ? THEN abnormal_count+1 ELSE 0 END,normal_count=CASE WHEN ? THEN 0 ELSE normal_count+1 END").bind(node).bind(kind).bind(target).bind(if high{1}else{0}).bind(if high{0}else{1}).bind(high).bind(high).execute(&mut **tx).await?;
    let (a,n):(i64,i64)=sqlx::query_as("SELECT abnormal_count,normal_count FROM alert_counters WHERE node_id=? AND kind=? AND target=?").bind(node).bind(kind).bind(target).fetch_one(&mut **tx).await?;
    if a >= 3 {
        #[cfg(feature = "notifications")]
        crate::notifications::outbox::open(
            tx,
            node,
            kind,
            target,
            &format!("{kind} threshold exceeded"),
            Some(threshold),
            Some(value),
            now,
        )
        .await?;
        #[cfg(not(feature = "notifications"))]
        sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,threshold_percent,observed_percent,opened_at,last_observed_at) VALUES(?,?,?,?, 'active', ?,?,?,?,?) ON CONFLICT(node_id,kind,target) WHERE status='active' DO UPDATE SET observed_percent=excluded.observed_percent,last_observed_at=excluded.last_observed_at").bind(Uuid::new_v4().to_string()).bind(node).bind(kind).bind(target).bind(format!("{kind} threshold exceeded")).bind(threshold).bind(value).bind(now).bind(now).execute(&mut **tx).await?;
    }
    if n >= 3 {
        resolve(tx, node, kind, target, now, "normal samples").await?;
    }
    Ok(())
}
pub(super) async fn resolve(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    now: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    #[cfg(feature = "notifications")]
    return crate::notifications::outbox::resolve_exact(tx, node, kind, target, now, reason).await;
    #[cfg(not(feature = "notifications"))]
    sqlx::query("UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason=?,last_observed_at=? WHERE node_id=? AND kind=? AND target=? AND status='active'").bind(now).bind(reason).bind(now).bind(node).bind(kind).bind(target).execute(&mut **tx).await?;
    #[cfg(not(feature = "notifications"))]
    Ok(())
}

pub async fn settings(State(s): State<AppState>) -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(settings_value(&s.pool).await?))
}

pub(super) async fn settings_value(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
    let row = sqlx::query("SELECT cpu_enabled,cpu_threshold_percent,memory_enabled,memory_threshold_percent,disk_enabled,disk_threshold_percent,offline_enabled,offline_after_seconds,updated_at FROM alert_settings WHERE id=1")
        .fetch_one(pool)
        .await?;
    Ok(settings_json(&row, "global"))
}

fn settings_json(row: &sqlx::sqlite::SqliteRow, source: &str) -> serde_json::Value {
    serde_json::json!({
        "cpu": {"enabled": row.get::<i64, _>("cpu_enabled") != 0, "thresholdPercent": row.get::<f64, _>("cpu_threshold_percent")},
        "memory": {"enabled": row.get::<i64, _>("memory_enabled") != 0, "thresholdPercent": row.get::<f64, _>("memory_threshold_percent")},
        "disk": {"enabled": row.get::<i64, _>("disk_enabled") != 0, "thresholdPercent": row.get::<f64, _>("disk_threshold_percent")},
        "offline": {"enabled": row.get::<i64, _>("offline_enabled") != 0, "afterSeconds": row.get::<i64, _>("offline_after_seconds")},
        "updatedAt": row.get::<String, _>("updated_at"),
        "source": source,
        "isCustom": source == "custom",
    })
}

pub(super) async fn node_settings_value(
    pool: &SqlitePool,
    node_id: &str,
) -> Result<serde_json::Value, AppError> {
    let row = sqlx::query(
        "SELECT
            COALESCE(n.cpu_enabled,g.cpu_enabled) AS cpu_enabled,
            COALESCE(n.cpu_threshold_percent,g.cpu_threshold_percent) AS cpu_threshold_percent,
            COALESCE(n.memory_enabled,g.memory_enabled) AS memory_enabled,
            COALESCE(n.memory_threshold_percent,g.memory_threshold_percent) AS memory_threshold_percent,
            COALESCE(n.disk_enabled,g.disk_enabled) AS disk_enabled,
            COALESCE(n.disk_threshold_percent,g.disk_threshold_percent) AS disk_threshold_percent,
            COALESCE(n.offline_enabled,g.offline_enabled) AS offline_enabled,
            COALESCE(n.offline_after_seconds,g.offline_after_seconds) AS offline_after_seconds,
            COALESCE(n.updated_at,g.updated_at) AS updated_at,
            CASE WHEN n.node_id IS NULL THEN 'global' ELSE 'custom' END AS source
         FROM monitor_nodes m CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings n ON n.node_id=m.node_id
         WHERE g.id=1 AND m.node_id=?",
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("node"))?;
    Ok(settings_json(&row, &row.get::<String, _>("source")))
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingInput {
    pub(super) cpu: Threshold,
    pub(super) memory: Threshold,
    pub(super) disk: Threshold,
    pub(super) offline: Offline,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Threshold {
    pub(super) enabled: bool,
    pub(super) threshold_percent: f64,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Offline {
    pub(super) enabled: bool,
    pub(super) after_seconds: i64,
}
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
