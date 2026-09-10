use axum::extract::State;
use rustzen_storage::SqlitePool;
use serde::Deserialize;
use sqlx::{Row, Sqlite, Transaction};

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub(in super::super) struct Settings {
    pub(in super::super) cpu_enabled: i64,
    pub(in super::super) cpu_threshold_percent: f64,
    pub(in super::super) memory_enabled: i64,
    pub(in super::super) memory_threshold_percent: f64,
    pub(in super::super) disk_enabled: i64,
    pub(in super::super) disk_threshold_percent: f64,
    pub(in super::super) offline_enabled: i64,
    pub(in super::super) offline_after_seconds: i64,
}
pub(super) async fn settings_row(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<Settings, sqlx::Error> {
    sqlx::query_as("SELECT cpu_enabled,cpu_threshold_percent,memory_enabled,memory_threshold_percent,disk_enabled,disk_threshold_percent,offline_enabled,offline_after_seconds FROM alert_settings WHERE id=1").fetch_one(&mut **tx).await
}

pub(in super::super) async fn effective_settings_row(
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
pub async fn settings(State(s): State<AppState>) -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(settings_value(&s.pool).await?))
}

pub(in super::super) async fn settings_value(
    pool: &SqlitePool,
) -> Result<serde_json::Value, AppError> {
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

pub(in super::super) async fn node_settings_value(
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
    pub(in super::super) cpu: Threshold,
    pub(in super::super) memory: Threshold,
    pub(in super::super) disk: Threshold,
    pub(in super::super) offline: Offline,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in super::super) struct Threshold {
    pub(in super::super) enabled: bool,
    pub(in super::super) threshold_percent: f64,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in super::super) struct Offline {
    pub(in super::super) enabled: bool,
    pub(in super::super) after_seconds: i64,
}
