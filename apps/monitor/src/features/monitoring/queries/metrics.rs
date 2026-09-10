use std::collections::BTreeMap;

use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use rustzen_ipc::ModuleQuery;
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

pub(in super::super) fn parse_utc(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| AppError::database())
}

pub(in super::super) fn aggregate_resource_points(
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

pub(in super::super) fn aggregate_disk_points(
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
