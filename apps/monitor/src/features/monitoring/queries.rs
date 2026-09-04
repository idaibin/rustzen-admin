use std::collections::BTreeMap;

use axum::extract::{Path, State};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rustzen_ipc::{ModuleQuery, Page, Pagination};
use rustzen_storage::SqlitePool;
use serde::{Deserialize, Serialize};
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

async fn node_value(
    pool: &SqlitePool,
    row: sqlx::sqlite::SqliteRow,
    now: DateTime<Utc>,
) -> Result<serde_json::Value, AppError> {
    let node_id: String = row.get("node_id");
    let last_report_at: String = row.get("last_report_at");
    let disks = sqlx::query(
        "SELECT mount_point,used_bytes,total_bytes,collected_at
         FROM disk_samples WHERE node_id=? AND collected_at=? ORDER BY mount_point",
    )
    .bind(&node_id)
    .bind(&last_report_at)
    .fetch_all(pool)
    .await?
    .into_iter()
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
    .collect::<Vec<_>>();
    let memory_used: i64 = row.get("memory_used_bytes");
    let memory_total: i64 = row.get("memory_total_bytes");
    Ok(serde_json::json!({
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
    }))
}

pub async fn nodes(State(s): State<AppState>) -> AppResult<Vec<serde_json::Value>> {
    let now = Utc::now();
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
    .fetch_all(&s.pool)
    .await?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        values.push(node_value(&s.pool, row, now).await?);
    }
    Ok(ApiResponse::success(values))
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
    Ok(ApiResponse::success(node_value(&s.pool, row, Utc::now()).await?))
}
#[derive(Deserialize)]
pub(crate) struct Range {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    bucket: Option<String>,
}
pub async fn metrics(
    State(s): State<AppState>,
    Path(id): Path<String>,
    ModuleQuery(q): ModuleQuery<Range>,
) -> AppResult<serde_json::Value> {
    let Range { from, to, bucket } = q;
    if !matches!(bucket.as_deref(), None | Some("raw") | Some("5m") | Some("1h")) {
        return Err(AppError::unprocessable("invalid metrics bucket"));
    }
    let end = to.unwrap_or_else(Utc::now);
    let start = from.unwrap_or_else(|| end - chrono::Duration::days(1));
    if start > end || end - start > chrono::Duration::days(30) {
        return Err(AppError::unprocessable("metrics window must not exceed 30 days"));
    }
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM monitor_nodes WHERE node_id=?")
        .bind(&id)
        .fetch_optional(&s.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("node"));
    }

    let rows = sqlx::query(
        "SELECT collected_at,cpu_percent,memory_used_bytes,memory_total_bytes
         FROM resource_samples
         WHERE node_id=? AND collected_at>=? AND collected_at<=?
         ORDER BY collected_at",
    )
    .bind(&id)
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(&s.pool)
    .await?;
    let width = match bucket.as_deref() {
        None | Some("raw") => None,
        Some("5m") => Some(300_i64),
        Some("1h") => Some(3_600_i64),
        Some(_) => unreachable!("bucket was validated above"),
    };
    let points = aggregate_resource_points(rows, width)?;

    let disk_rows = sqlx::query(
        "SELECT mount_point,collected_at,used_bytes,total_bytes
         FROM disk_samples
         WHERE node_id=? AND collected_at>=? AND collected_at<=?
         ORDER BY mount_point,collected_at",
    )
    .bind(&id)
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(&s.pool)
    .await?;
    let disks = aggregate_disk_points(disk_rows, width)?;
    Ok(ApiResponse::success(
        serde_json::json!({"bucket":bucket.unwrap_or_else(||"raw".into()),"points":points,"disks":disks}),
    ))
}

pub(super) fn parse_utc(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| AppError::database())
}

pub(super) fn aggregate_resource_points(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    width: Option<i64>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let mut buckets: BTreeMap<i64, (f64, f64, i64)> = BTreeMap::new();
    for row in rows {
        let collected_at: String = row.get("collected_at");
        let timestamp = parse_utc(&collected_at)?;
        let cpu: f64 = row.get("cpu_percent");
        let used: i64 = row.get("memory_used_bytes");
        let total: i64 = row.get("memory_total_bytes");
        let memory = used as f64 * 100.0 / total as f64;
        let key = width.map_or(timestamp.timestamp(), |seconds| {
            timestamp.timestamp().div_euclid(seconds) * seconds
        });
        let entry = buckets.entry(key).or_insert((0.0, 0.0, 0));
        entry.0 += cpu;
        entry.1 += memory;
        entry.2 += 1;
    }
    buckets
        .into_iter()
        .map(|(timestamp, (cpu, memory, count))| {
            let collected_at =
                DateTime::from_timestamp(timestamp, 0).ok_or_else(AppError::database)?.to_rfc3339();
            Ok(serde_json::json!({
                "collectedAt": collected_at,
                "cpuPercent": cpu / count as f64,
                "memoryPercent": memory / count as f64,
            }))
        })
        .collect()
}

pub(super) fn aggregate_disk_points(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    width: Option<i64>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let mut mounts: BTreeMap<String, BTreeMap<i64, (f64, i64)>> = BTreeMap::new();
    for row in rows {
        let mount: String = row.get("mount_point");
        let collected_at: String = row.get("collected_at");
        let timestamp = parse_utc(&collected_at)?;
        let used: i64 = row.get("used_bytes");
        let total: i64 = row.get("total_bytes");
        let percent = used as f64 * 100.0 / total as f64;
        let key = width.map_or(timestamp.timestamp(), |seconds| {
            timestamp.timestamp().div_euclid(seconds) * seconds
        });
        let entry = mounts.entry(mount).or_default().entry(key).or_insert((0.0, 0));
        entry.0 += percent;
        entry.1 += 1;
    }
    mounts
        .into_iter()
        .map(|(mount_point, points)| {
            let points = points
                .into_iter()
                .map(|(timestamp, (percent, count))| {
                    let collected_at = DateTime::from_timestamp(timestamp, 0)
                        .ok_or_else(AppError::database)?
                        .to_rfc3339();
                    Ok(serde_json::json!({
                        "collectedAt": collected_at,
                        "percent": percent / count as f64,
                    }))
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            Ok(serde_json::json!({"mountPoint": mount_point, "points": points}))
        })
        .collect()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IncidentQuery {
    current: Option<i64>,
    page_size: Option<i64>,
    status: Option<String>,
    kind: Option<String>,
    node_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

pub(super) fn query_window(
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
    let end = to.unwrap_or_else(Utc::now);
    let start = from.unwrap_or_else(|| end - ChronoDuration::days(30));
    if start > end || end - start > ChronoDuration::days(30) {
        return Err(AppError::unprocessable("query window must not exceed 30 days"));
    }
    Ok((start, end))
}

fn validate_incident_filter(status: Option<&str>, kind: Option<&str>) -> Result<(), AppError> {
    if let Some(status) = status
        && !matches!(status, "active" | "resolved")
    {
        return Err(AppError::unprocessable("invalid incident status"));
    }
    if let Some(kind) = kind
        && !matches!(kind, "cpuHigh" | "memoryHigh" | "diskHigh" | "nodeOffline")
    {
        return Err(AppError::unprocessable("invalid incident kind"));
    }
    Ok(())
}

pub async fn incidents(
    State(s): State<AppState>,
    ModuleQuery(q): ModuleQuery<IncidentQuery>,
) -> AppResult<Page<serde_json::Value>> {
    let page = Pagination::parse(q.current, q.page_size)
        .map_err(|_| AppError::unprocessable("invalid pagination"))?;
    validate_incident_filter(q.status.as_deref(), q.kind.as_deref())?;
    let (start, end) = query_window(q.from, q.to)?;
    let start = start.to_rfc3339();
    let end = end.to_rfc3339();
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monitor_incidents
         WHERE (? IS NULL OR status=?) AND (? IS NULL OR kind=?) AND (? IS NULL OR node_id=?)
           AND (status='active' OR (status='resolved' AND resolved_at>=? AND resolved_at<=?))",
    )
    .bind(q.status.as_deref())
    .bind(q.status.as_deref())
    .bind(q.kind.as_deref())
    .bind(q.kind.as_deref())
    .bind(q.node_id.as_deref())
    .bind(q.node_id.as_deref())
    .bind(&start)
    .bind(&end)
    .fetch_one(&s.pool)
    .await?;
    let rows = sqlx::query(
        "SELECT id,node_id,kind,target,status,title,threshold_percent,observed_percent,opened_at,
                last_observed_at,resolved_at,resolution_reason,details
         FROM monitor_incidents
         WHERE (? IS NULL OR status=?) AND (? IS NULL OR kind=?) AND (? IS NULL OR node_id=?)
           AND (status='active' OR (status='resolved' AND resolved_at>=? AND resolved_at<=?))
         ORDER BY last_observed_at DESC LIMIT ? OFFSET ?",
    )
    .bind(q.status.as_deref())
    .bind(q.status.as_deref())
    .bind(q.kind.as_deref())
    .bind(q.kind.as_deref())
    .bind(q.node_id.as_deref())
    .bind(q.node_id.as_deref())
    .bind(&start)
    .bind(&end)
    .bind(page.page_size())
    .bind(page.offset())
    .fetch_all(&s.pool)
    .await?;
    let data = rows.iter().map(incident_value).collect::<Result<Vec<_>, _>>()?;
    Ok(ApiResponse::success(Page { data, total, success: true }))
}

fn incident_value(row: &sqlx::sqlite::SqliteRow) -> Result<serde_json::Value, AppError> {
    let details = serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("details"))
        .unwrap_or_else(|_| serde_json::json!({}));
    Ok(serde_json::json!({
        "id": row.get::<String, _>("id"),
        "nodeId": row.get::<String, _>("node_id"),
        "kind": row.get::<String, _>("kind"),
        "target": row.get::<String, _>("target"),
        "status": row.get::<String, _>("status"),
        "title": row.get::<String, _>("title"),
        "thresholdPercent": row.get::<Option<f64>, _>("threshold_percent"),
        "observedPercent": row.get::<Option<f64>, _>("observed_percent"),
        "openedAt": row.get::<String, _>("opened_at"),
        "lastObservedAt": row.get::<String, _>("last_observed_at"),
        "resolvedAt": row.get::<Option<String>, _>("resolved_at"),
        "resolutionReason": row.get::<Option<String>, _>("resolution_reason"),
        "details": details,
    }))
}

pub async fn incident(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<serde_json::Value> {
    let row = sqlx::query(
        "SELECT i.id,i.node_id,i.kind,i.target,i.status,i.title,i.threshold_percent,i.observed_percent,
                i.opened_at,i.last_observed_at,i.resolved_at,i.resolution_reason,i.details,
                n.hostname,n.agent_version,n.last_report_at,n.last_received_at
         FROM monitor_incidents i JOIN monitor_nodes n ON n.node_id=i.node_id WHERE i.id=?",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or_else(|| AppError::not_found("incident"))?;
    let mut value = incident_value(&row)?;
    value["node"] = serde_json::json!({
        "nodeId": row.get::<String, _>("node_id"),
        "hostname": row.get::<String, _>("hostname"),
        "agentVersion": row.get::<String, _>("agent_version"),
        "lastReportAt": row.get::<String, _>("last_report_at"),
        "lastReceivedAt": row.get::<String, _>("last_received_at"),
    });
    Ok(ApiResponse::success(value))
}
