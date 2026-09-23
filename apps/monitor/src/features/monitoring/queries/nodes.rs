use std::{collections::BTreeMap, future::Future};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};
use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use serde::Serialize;
use sqlx::Row;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Overview {
    registered_nodes: i64,
    online_nodes: i64,
    offline_nodes: i64,
    active_incidents: i64,
    latest_resource: Option<serde_json::Value>,
}
pub async fn overview(State(s): State<AppState>) -> AppResult<Overview> {
    let now = Utc::now();
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monitor_nodes").fetch_one(&s.pool).await?;
    let online: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monitor_nodes n
         CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings s ON s.node_id=n.node_id
         WHERE g.id=1 AND (? - unixepoch(n.last_received_at)) <=
             COALESCE(s.offline_after_seconds,g.offline_after_seconds)",
    )
    .bind(now.timestamp())
    .fetch_one(&s.pool)
    .await?;
    let active = sqlx::query_scalar("SELECT COUNT(*) FROM monitor_incidents WHERE status='active'")
        .fetch_one(&s.pool)
        .await?;
    let latest_resource = if let Some(row) = sqlx::query(
        "SELECT node_id,cpu_percent,memory_used_bytes,memory_total_bytes,last_report_at,last_received_at
         FROM monitor_nodes ORDER BY last_report_at DESC LIMIT 1",
    )
    .fetch_optional(&s.pool)
    .await?
    {
        let node_id: String = row.get("node_id");
        let collected_at: String = row.get("last_report_at");
        let disks = sqlx::query(
            "SELECT mount_point,used_bytes,total_bytes FROM disk_samples
             WHERE node_id=? AND collected_at=? ORDER BY mount_point",
        )
        .bind(&node_id)
        .bind(&collected_at)
        .fetch_all(&s.pool)
        .await?
        .into_iter()
        .map(|disk| {
            let used: i64 = disk.get("used_bytes");
            let total: i64 = disk.get("total_bytes");
            serde_json::json!({
                "mountPoint": disk.get::<String, _>("mount_point"),
                "usedBytes": used,
                "totalBytes": total,
                "usagePercent": used as f64 * 100.0 / total as f64,
            })
        })
        .collect::<Vec<_>>();
        Some(serde_json::json!({
            "nodeId": row.get::<String, _>("node_id"),
            "collectedAt": row.get::<String, _>("last_report_at"),
            "lastReceivedAt": row.get::<String, _>("last_received_at"),
            "cpuPercent": row.get::<f64, _>("cpu_percent"),
            "memoryPercent": row.get::<i64, _>("memory_used_bytes") as f64 * 100.0 / row.get::<i64, _>("memory_total_bytes") as f64,
            "disks": disks,
        }))
    } else {
        None
    };
    Ok(ApiResponse::success(Overview {
        registered_nodes: total,
        online_nodes: online,
        offline_nodes: total - online,
        active_incidents: active,
        latest_resource,
    }))
}

fn disk_values(rows: Vec<sqlx::sqlite::SqliteRow>) -> Vec<serde_json::Value> {
    rows.into_iter()
        .map(|disk| {
            let used: i64 = disk.get("used_bytes");
            let total: i64 = disk.get("total_bytes");
            serde_json::json!({
                "mountPoint": disk.get::<String, _>("mount_point"),
                "collectedAt": disk.get::<String, _>("collected_at"),
                "usedBytes": used,
                "totalBytes": total,
                "usagePercent": used as f64 * 100.0 / total as f64,
            })
        })
        .collect()
}

pub(in super::super) const LATEST_NODE_DISKS_SQL: &str =
    "SELECT d.node_id,d.mount_point,d.used_bytes,d.total_bytes,d.collected_at
     FROM monitor_nodes n
     CROSS JOIN disk_samples d INDEXED BY idx_disk_samples_node_time_mount
     WHERE d.node_id=n.node_id AND d.collected_at=n.last_report_at
     ORDER BY d.node_id,d.mount_point";

async fn latest_disk_values_by_node(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<BTreeMap<String, Vec<serde_json::Value>>, AppError> {
    let mut disks_by_node: BTreeMap<String, Vec<sqlx::sqlite::SqliteRow>> = BTreeMap::new();
    for row in sqlx::query(LATEST_NODE_DISKS_SQL).fetch_all(&mut **transaction).await? {
        let node_id: String = row.get("node_id");
        disks_by_node.entry(node_id).or_default().push(row);
    }
    Ok(disks_by_node.into_iter().map(|(node_id, disks)| (node_id, disk_values(disks))).collect())
}

fn node_value(
    row: sqlx::sqlite::SqliteRow,
    disks: Vec<serde_json::Value>,
    now: DateTime<Utc>,
) -> serde_json::Value {
    let node_id: String = row.get("node_id");
    let last_report_at: String = row.get("last_report_at");
    let memory_used: i64 = row.get("memory_used_bytes");
    let memory_total: i64 = row.get("memory_total_bytes");
    serde_json::json!({
        "nodeId": node_id,
        "hostname": row.get::<String, _>("hostname"),
        "agentVersion": row.get::<String, _>("agent_version"),
        "bootId": row.get::<String, _>("current_boot_id"),
        "sequence": row.get::<i64, _>("last_sequence"),
        "lastReportAt": last_report_at,
        "lastReceivedAt": row.get::<String, _>("last_received_at"),
        "status": if (now - DateTime::parse_from_rfc3339(&row.get::<String, _>("last_received_at")).map(|value| value.with_timezone(&Utc)).unwrap_or(now)).num_seconds() <= row.get::<i64, _>("effective_offline_after_seconds") { "online" } else { "offline" },
        "alertPolicySource": row.get::<String, _>("alert_policy_source"),
        "cpuPercent": row.get::<f64, _>("cpu_percent"),
        "memory": {"usedBytes": memory_used, "totalBytes": memory_total, "usagePercent": memory_used as f64 * 100.0 / memory_total as f64},
        "disks": disks,
        "createdAt": row.get::<String, _>("created_at"),
        "updatedAt": row.get::<String, _>("updated_at"),
    })
}

pub(in super::super) async fn node_values(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<Vec<serde_json::Value>, AppError> {
    node_values_after_rows(pool, now, std::future::ready(())).await
}

pub(in super::super) async fn node_values_after_rows<F>(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    after_rows: F,
) -> Result<Vec<serde_json::Value>, AppError>
where
    F: Future<Output = ()>,
{
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        "SELECT n.*,
                COALESCE(s.offline_after_seconds,g.offline_after_seconds)
                    AS effective_offline_after_seconds,
                CASE WHEN s.node_id IS NULL THEN 'global' ELSE 'custom' END
                    AS alert_policy_source
         FROM monitor_nodes n CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings s ON s.node_id=n.node_id
         WHERE g.id=1 ORDER BY n.hostname",
    )
    .fetch_all(&mut *transaction)
    .await?;
    after_rows.await;
    let mut disks_by_node = latest_disk_values_by_node(&mut transaction).await?;
    let values = rows
        .into_iter()
        .map(|row| {
            let node_id: String = row.get("node_id");
            node_value(row, disks_by_node.remove(&node_id).unwrap_or_default(), now)
        })
        .collect();
    transaction.commit().await?;
    Ok(values)
}

pub async fn nodes(State(s): State<AppState>) -> Result<Response, AppError> {
    let body = s
        .nodes_cache
        .get_or_load(|| async {
            let values = node_values(&s.pool, Utc::now()).await?;
            serde_json::to_vec(&ApiResponse::success(values).0)
                .map(axum::body::Bytes::from)
                .map_err(|error| {
                    tracing::error!(%error, "Monitor node response serialization failed");
                    AppError::database()
                })
        })
        .await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("static Monitor Nodes response is valid"))
}
pub async fn node(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<serde_json::Value> {
    let row = sqlx::query(
        "SELECT n.*,
                COALESCE(s.offline_after_seconds,g.offline_after_seconds)
                    AS effective_offline_after_seconds,
                CASE WHEN s.node_id IS NULL THEN 'global' ELSE 'custom' END
                    AS alert_policy_source
         FROM monitor_nodes n CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings s ON s.node_id=n.node_id
         WHERE g.id=1 AND n.node_id=?",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    let row = row.ok_or_else(|| AppError::not_found("node"))?;
    let node_id: String = row.get("node_id");
    let last_report_at: String = row.get("last_report_at");
    let disks = disk_values(
        sqlx::query(
            "SELECT mount_point,used_bytes,total_bytes,collected_at
             FROM disk_samples WHERE node_id=? AND collected_at=? ORDER BY mount_point",
        )
        .bind(node_id)
        .bind(last_report_at)
        .fetch_all(&s.pool)
        .await?,
    );
    Ok(ApiResponse::success(node_value(row, disks, Utc::now())))
}
