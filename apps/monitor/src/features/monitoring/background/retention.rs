use std::pin::Pin;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rustzen_storage::{
    CoreError, SqliteMaintenancePlan, SqliteMaintenanceReport, SqlitePool, run_sqlite_maintenance,
};

use crate::common::error::AppError;

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

pub(in super::super) async fn cleanup_at_with<F>(
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
