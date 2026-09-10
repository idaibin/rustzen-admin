use axum::extract::State;
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use rustzen_ipc::{ModuleQuery, Page, Pagination};
use rustzen_storage::SqlitePool;
use serde::Deserialize;
use sqlx::Row;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

use super::super::queries::query_window;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SummaryQuery {
    current: Option<i64>,
    page_size: Option<i64>,
    node_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}
pub async fn summaries(
    State(s): State<AppState>,
    ModuleQuery(q): ModuleQuery<SummaryQuery>,
) -> AppResult<Page<serde_json::Value>> {
    let page = Pagination::parse(q.current, q.page_size)
        .map_err(|_| AppError::unprocessable("invalid pagination"))?;
    let (start, end) = query_window(q.from, q.to)?;
    let start = start.date_naive().to_string();
    let end = end.date_naive().to_string();
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM node_daily_summaries
         WHERE (? IS NULL OR node_id=?) AND summary_date>=? AND summary_date<=?",
    )
    .bind(q.node_id.as_deref())
    .bind(q.node_id.as_deref())
    .bind(&start)
    .bind(&end)
    .fetch_one(&s.pool)
    .await?;
    let rows = sqlx::query(
        "SELECT node_id,summary_date,sample_count,coverage_percent,cpu_min,cpu_avg,cpu_max,
                memory_min,memory_avg,memory_max,disk_summary_json,offline_seconds,incident_count
         FROM node_daily_summaries WHERE (? IS NULL OR node_id=?) AND summary_date>=? AND summary_date<=?
         ORDER BY summary_date DESC,node_id LIMIT ? OFFSET ?",
    )
    .bind(q.node_id.as_deref())
    .bind(q.node_id.as_deref())
    .bind(&start)
    .bind(&end)
    .bind(page.page_size())
    .bind(page.offset())
    .fetch_all(&s.pool)
    .await?;
    let data = rows
        .into_iter()
        .map(|row| {
            let disk_summary = serde_json::from_str::<serde_json::Value>(
                &row.get::<String, _>("disk_summary_json"),
            )
            .unwrap_or_else(|_| serde_json::json!({}));
            serde_json::json!({
                "nodeId": row.get::<String, _>("node_id"),
                "date": row.get::<String, _>("summary_date"),
                "sampleCount": row.get::<i64, _>("sample_count"),
                "coverage": row.get::<f64, _>("coverage_percent"),
                "coveragePercent": row.get::<f64, _>("coverage_percent"),
                "cpu": {"min": row.get::<Option<f64>, _>("cpu_min"), "avg": row.get::<Option<f64>, _>("cpu_avg"), "max": row.get::<Option<f64>, _>("cpu_max")},
                "memory": {"min": row.get::<Option<f64>, _>("memory_min"), "avg": row.get::<Option<f64>, _>("memory_avg"), "max": row.get::<Option<f64>, _>("memory_max")},
                "diskSummary": disk_summary,
                "offlineSeconds": row.get::<i64, _>("offline_seconds"),
                "incidentCount": row.get::<i64, _>("incident_count"),
            })
        })
        .collect();
    Ok(ApiResponse::success(Page { data, total, success: true }))
}

pub(crate) async fn generate_daily_summaries_at(
    pool: &SqlitePool,
    day: NaiveDate,
) -> Result<(), sqlx::Error> {
    let day_text = day.to_string();
    let day_start = day.and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    let day_end = day_start + ChronoDuration::days(1);
    let start = day_start.to_rfc3339();
    let end = day_end.to_rfc3339();
    let nodes = sqlx::query(
        "SELECT node_id FROM monitor_nodes WHERE created_at<?
         UNION SELECT node_id FROM resource_samples WHERE collected_at>=? AND collected_at<?
         UNION SELECT node_id FROM monitor_incidents
           WHERE opened_at<? AND (resolved_at IS NULL OR resolved_at>?)",
    )
    .bind(&end)
    .bind(&start)
    .bind(&end)
    .bind(&end)
    .bind(&start)
    .fetch_all(pool)
    .await?;
    for node in nodes {
        let node_id: String = node.get("node_id");
        let row = sqlx::query(
            "SELECT COUNT(*) AS sample_count,
                    MIN(cpu_percent) AS cpu_min,AVG(cpu_percent) AS cpu_avg,MAX(cpu_percent) AS cpu_max,
                    MIN(memory_used_bytes*100.0/memory_total_bytes) AS memory_min,
                    AVG(memory_used_bytes*100.0/memory_total_bytes) AS memory_avg,
                    MAX(memory_used_bytes*100.0/memory_total_bytes) AS memory_max
             FROM resource_samples WHERE node_id=? AND collected_at>=? AND collected_at<?",
        )
        .bind(&node_id)
        .bind(&start)
        .bind(&end)
        .fetch_one(pool)
        .await?;
        let disk_rows = sqlx::query(
            "SELECT mount_point,MIN(used_bytes*100.0/total_bytes) AS min_percent,
                    AVG(used_bytes*100.0/total_bytes) AS avg_percent,
                    MAX(used_bytes*100.0/total_bytes) AS max_percent
             FROM disk_samples WHERE node_id=? AND collected_at>=? AND collected_at<?
             GROUP BY mount_point ORDER BY mount_point",
        )
        .bind(&node_id)
        .bind(&start)
        .bind(&end)
        .fetch_all(pool)
        .await?;
        let mut disks = serde_json::Map::new();
        for disk in disk_rows {
            let mount: String = disk.get("mount_point");
            disks.insert(
                mount,
                serde_json::json!({
                    "min": disk.get::<f64, _>("min_percent"),
                    "avg": disk.get::<f64, _>("avg_percent"),
                    "max": disk.get::<f64, _>("max_percent"),
                }),
            );
        }
        let offline_seconds = offline_seconds_for_day(pool, &node_id, day_start, day_end).await?;
        let incident_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM monitor_incidents
             WHERE node_id=? AND opened_at<? AND (resolved_at IS NULL OR resolved_at>?)",
        )
        .bind(&node_id)
        .bind(&end)
        .bind(&start)
        .fetch_one(pool)
        .await?;
        sqlx::query(
            "INSERT INTO node_daily_summaries
             (node_id,summary_date,sample_count,cpu_min,cpu_avg,cpu_max,memory_min,memory_avg,memory_max,
              disk_summary_json,coverage_percent,offline_seconds,incident_count)
             VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)
             ON CONFLICT(node_id,summary_date) DO UPDATE SET
              sample_count=excluded.sample_count,cpu_min=excluded.cpu_min,cpu_avg=excluded.cpu_avg,cpu_max=excluded.cpu_max,
              memory_min=excluded.memory_min,memory_avg=excluded.memory_avg,memory_max=excluded.memory_max,
              disk_summary_json=excluded.disk_summary_json,coverage_percent=excluded.coverage_percent,
              offline_seconds=excluded.offline_seconds,
              incident_count=excluded.incident_count",
        )
        .bind(&node_id)
        .bind(&day_text)
        .bind(row.get::<i64, _>("sample_count"))
        .bind(row.get::<Option<f64>, _>("cpu_min"))
        .bind(row.get::<Option<f64>, _>("cpu_avg"))
        .bind(row.get::<Option<f64>, _>("cpu_max"))
        .bind(row.get::<Option<f64>, _>("memory_min"))
        .bind(row.get::<Option<f64>, _>("memory_avg"))
        .bind(row.get::<Option<f64>, _>("memory_max"))
        .bind(serde_json::Value::Object(disks).to_string())
        .bind((row.get::<i64, _>("sample_count") as f64 / 2_880.0 * 100.0).min(100.0))
        .bind(offline_seconds)
        .bind(incident_count)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn offline_seconds_for_day(
    pool: &SqlitePool,
    node_id: &str,
    day_start: DateTime<Utc>,
    day_end: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT opened_at,resolved_at FROM monitor_incidents
         WHERE node_id=? AND kind='nodeOffline' AND opened_at<?
           AND (resolved_at IS NULL OR resolved_at>?)",
    )
    .bind(node_id)
    .bind(day_end.to_rfc3339())
    .bind(day_start.to_rfc3339())
    .fetch_all(pool)
    .await?;
    let mut total = 0;
    for row in rows {
        let opened = DateTime::parse_from_rfc3339(&row.get::<String, _>("opened_at"))
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(day_start);
        let resolved = row
            .get::<Option<String>, _>("resolved_at")
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(day_end);
        let from = opened.max(day_start);
        let to = resolved.min(day_end);
        total += (to - from).num_seconds().max(0);
    }
    Ok(total)
}
