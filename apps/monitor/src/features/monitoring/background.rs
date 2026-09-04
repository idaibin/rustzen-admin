use std::{pin::Pin, time::Duration};

use axum::extract::State;
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use rustzen_ipc::{ModuleQuery, Page, Pagination};
use rustzen_storage::{
    CoreError, SqliteMaintenancePlan, SqliteMaintenanceReport, SqlitePool, run_sqlite_maintenance,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    app::AppState,
    common::{
        api::{ApiResponse, AppResult},
        error::AppError,
    },
};

use super::{acceptance::LockHook, queries::query_window};

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

pub(crate) fn spawn_background(pool: SqlitePool) {
    let offline_pool = pool.clone();
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(30));
        loop {
            timer.tick().await;
            let now = Utc::now();
            if let Err(error) = offline_scan_at(&offline_pool, now).await {
                tracing::error!(%error, "monitor offline scan failed");
            }
        }
    });
    let summary_pool = pool.clone();
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(3_600));
        loop {
            timer.tick().await;
            let now = Utc::now();
            if let Err(error) = generate_daily_summaries_at(
                &summary_pool,
                (now - ChronoDuration::days(1)).date_naive(),
            )
            .await
            {
                tracing::error!(%error, "monitor daily summary generation failed");
            }
        }
    });
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(3_600));
        loop {
            timer.tick().await;
            match cleanup_at(&pool, Utc::now()).await {
                Ok(report) => {
                    tracing::debug!(
                        deleted_rows = report.deleted_rows,
                        "monitor retention rows deleted"
                    );
                    match &report.maintenance {
                        MaintenanceResult::Succeeded(maintenance) => {
                            tracing::debug!(
                                before_freelist = maintenance.before.freelist_count,
                                after_freelist = maintenance.after.freelist_count,
                                "monitor SQLite maintenance completed"
                            );
                        }
                        MaintenanceResult::Failed(error) => {
                            tracing::error!(%error, "monitor SQLite maintenance pending");
                        }
                        MaintenanceResult::NotNeeded => {}
                    }
                }
                Err(error) => {
                    tracing::error!(%error, "monitor retention cleanup failed");
                }
            }
        }
    });
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

pub(crate) async fn offline_scan_at(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    offline_scan_at_inner(pool, now, None).await
}

#[cfg(test)]
pub(super) async fn offline_scan_at_with_lock_hook(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    lock_hook: LockHook,
) -> Result<(), sqlx::Error> {
    offline_scan_at_inner(pool, now, Some(lock_hook)).await
}

async fn offline_scan_at_inner(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    lock_hook: Option<LockHook>,
) -> Result<(), sqlx::Error> {
    // Liveness must be decided against a snapshot that cannot be changed by a
    // concurrent report between the read and the incident insert.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if let Some(lock_hook) = lock_hook {
        lock_hook.acquired.notify_one();
        lock_hook.release.notified().await;
    }
    for node in sqlx::query(
        "SELECT n.node_id,n.last_received_at,
                COALESCE(s.offline_enabled,g.offline_enabled) AS offline_enabled,
                COALESCE(s.offline_after_seconds,g.offline_after_seconds)
                    AS offline_after_seconds
         FROM monitor_nodes n CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings s ON s.node_id=n.node_id
         WHERE g.id=1",
    )
    .fetch_all(&mut *tx)
    .await?
    {
        if node.get::<i64, _>("offline_enabled") == 0 {
            continue;
        }
        let id: String = node.get(0);
        let last: DateTime<Utc> = DateTime::parse_from_rfc3339(&node.get::<String, _>(1))
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(now);
        if (now - last).num_seconds() > node.get::<i64, _>("offline_after_seconds") {
            let t = now.to_rfc3339();
            sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES(?,?, 'nodeOffline','node','active','node offline',?,?) ON CONFLICT(node_id,kind,target) WHERE status='active' DO UPDATE SET last_observed_at=excluded.last_observed_at").bind(Uuid::new_v4().to_string()).bind(id).bind(&t).bind(&t).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
pub(crate) struct CleanupReport {
    pub deleted_rows: u64,
    pub maintenance: MaintenanceResult,
}

#[derive(Debug)]
pub(crate) enum MaintenanceResult {
    NotNeeded,
    Succeeded(SqliteMaintenanceReport),
    Failed(String),
}

type MaintenanceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<SqliteMaintenanceReport, CoreError>> + Send + 'a>>;

pub(crate) async fn cleanup_at(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<CleanupReport, AppError> {
    cleanup_at_with(pool, now, |pool, plan| Box::pin(run_sqlite_maintenance(pool, plan))).await
}

pub(super) async fn cleanup_at_with<F>(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    maintenance: F,
) -> Result<CleanupReport, AppError>
where
    F: for<'a> Fn(&'a SqlitePool, SqliteMaintenancePlan) -> MaintenanceFuture<'a>,
{
    let cutoff = (now - ChronoDuration::days(30)).to_rfc3339();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let mut deleted_rows = 0;
    deleted_rows += sqlx::query("DELETE FROM resource_samples WHERE collected_at < ?")
        .bind(&cutoff)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    deleted_rows += sqlx::query("DELETE FROM disk_samples WHERE collected_at < ?")
        .bind(&cutoff)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    deleted_rows += sqlx::query("DELETE FROM node_daily_summaries WHERE summary_date < ?")
        .bind((now - ChronoDuration::days(30)).date_naive().to_string())
        .execute(&mut *tx)
        .await?
        .rows_affected();
    deleted_rows +=
        sqlx::query("DELETE FROM monitor_incidents WHERE status='resolved' AND resolved_at < ?")
            .bind(cutoff)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    tx.commit().await?;
    let freelist_count: i64 = sqlx::query_scalar("PRAGMA freelist_count").fetch_one(pool).await?;
    let maintenance = if deleted_rows > 0 || freelist_count > 0 {
        match maintenance(pool, SqliteMaintenancePlan::reclaim()).await {
            Ok(report) => MaintenanceResult::Succeeded(report),
            Err(error) => {
                tracing::error!(%error, "monitor SQLite maintenance failed; retry pending");
                MaintenanceResult::Failed(error.to_string())
            }
        }
    } else {
        MaintenanceResult::NotNeeded
    };
    Ok(CleanupReport { deleted_rows, maintenance })
}
