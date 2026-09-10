use axum::extract::{Path, State};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rustzen_ipc::{ModuleQuery, Page, Pagination};
use serde::Deserialize;
use sqlx::Row;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};
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

pub(in super::super) fn query_window(
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
