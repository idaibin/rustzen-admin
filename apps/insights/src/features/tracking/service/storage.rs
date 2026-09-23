use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use axum::http::StatusCode;
use chrono::{Duration as ChronoDuration, Utc};
use rustzen_storage::{SqliteMaintenancePlan, SqlitePool, run_sqlite_maintenance};
use sqlx::Row;
use sysinfo::Disks;

use crate::common::error::AppError;

use super::super::{repo, types::NewEvent};

pub(super) const STORAGE_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
pub(super) const FREE_DISK_RESERVE_BYTES: u64 = 512 * 1024 * 1024;
const STORAGE_PAGE_AMPLIFICATION: u64 = 4;
const STORAGE_EVENT_OVERHEAD_BYTES: u64 = 2048;
const STORAGE_WAL_RESERVE_BYTES: u64 = 64 * 1024;

pub(super) async fn ensure_storage_capacity(
    pool: &SqlitePool,
    events: &[NewEvent],
) -> Result<(), AppError> {
    let Some(database_path) = database_path(pool).await? else {
        // In-memory SQLite has no filesystem budget to measure; production paths
        // always resolve to a file and are checked below.
        return Ok(());
    };
    let projected_bytes = events
        .iter()
        .map(estimated_event_bytes)
        .map(|bytes| bytes.saturating_mul(STORAGE_PAGE_AMPLIFICATION))
        .map(|bytes| bytes.saturating_add(STORAGE_EVENT_OVERHEAD_BYTES))
        .sum::<u64>()
        .saturating_add(STORAGE_WAL_RESERVE_BYTES);
    tokio::task::spawn_blocking(move || {
        if filesystem_capacity_ok(&database_path, projected_bytes) {
            Ok(())
        } else {
            Err(AppError::input_rejection(
                StatusCode::INSUFFICIENT_STORAGE,
                "Insights storage protection rejected the batch",
            ))
        }
    })
    .await
    .map_err(AppError::internal)?
}

async fn database_path(pool: &SqlitePool) -> Result<Option<PathBuf>, AppError> {
    let rows =
        sqlx::query("PRAGMA database_list").fetch_all(pool).await.map_err(AppError::internal)?;
    let row = rows
        .into_iter()
        .find(|row| row.try_get::<String, _>("name").ok().is_some_and(|name| name == "main"));
    let Some(row) = row else { return Ok(None) };
    let file = row.try_get::<String, _>("file").map_err(AppError::internal)?;
    if file.is_empty() { Ok(None) } else { Ok(Some(PathBuf::from(file))) }
}

fn estimated_event_bytes(event: &NewEvent) -> u64 {
    (event.event_name.len()
        + event.visitor_id.len()
        + event.user_id.as_deref().map_or(0, str::len)
        + event.session_id.as_deref().map_or(0, str::len)
        + event.platform.as_deref().map_or(0, str::len)
        + event.page_path.as_deref().map_or(0, str::len)
        + event.referrer.as_deref().map_or(0, str::len)
        + event.api_path.as_deref().map_or(0, str::len)
        + event.api_method.as_deref().map_or(0, str::len)
        + event.properties.len()
        + event.occurred_at.len()
        + event.received_at.len()) as u64
        + 256
}

fn filesystem_capacity_ok(database_path: &Path, projected_bytes: u64) -> bool {
    let Ok(main_size) = fs::metadata(database_path).map(|metadata| metadata.len()) else {
        return false;
    };
    let sidecar_size = [
        PathBuf::from(format!("{}-wal", database_path.display())),
        PathBuf::from(format!("{}-shm", database_path.display())),
    ]
    .into_iter()
    .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
    .sum::<u64>();
    let current_bytes = main_size.saturating_add(sidecar_size);
    let Ok(candidate) = database_path.canonicalize() else { return false };
    let disks = Disks::new_with_refreshed_list();
    let Some(disk) = disks
        .list()
        .iter()
        .filter(|disk| candidate.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
    else {
        return false;
    };
    check_storage_capacity(current_bytes, disk.available_space(), projected_bytes).is_ok()
}

pub(super) fn check_storage_capacity(
    current_bytes: u64,
    available_bytes: u64,
    projected_bytes: u64,
) -> Result<(), AppError> {
    if current_bytes.saturating_add(projected_bytes) > STORAGE_BUDGET_BYTES
        || available_bytes < FREE_DISK_RESERVE_BYTES.saturating_add(projected_bytes)
    {
        return Err(AppError::input_rejection(
            StatusCode::INSUFFICIENT_STORAGE,
            "Insights storage protection rejected the batch",
        ));
    }
    Ok(())
}

pub fn spawn_retention(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            let result = async {
                let settings = crate::features::settings::service::get(&pool).await?;
                let cutoff =
                    (Utc::now() - ChronoDuration::days(settings.event_retention_days)).to_rfc3339();
                cleanup_before(&pool, &cutoff).await
            }
            .await;
            match result {
                Ok(0) => {}
                Ok(deleted) => {
                    tracing::info!(deleted, "Insights retention completed");
                    if let Err(error) =
                        run_sqlite_maintenance(&pool, SqliteMaintenancePlan::reclaim()).await
                    {
                        tracing::error!(%error, "Insights SQLite maintenance failed");
                    }
                }
                Err(error) => tracing::error!(%error, "Insights retention failed"),
            }
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });
}

async fn cleanup_before(pool: &SqlitePool, cutoff: &str) -> Result<u64, AppError> {
    let mut transaction = pool.begin().await.map_err(AppError::internal)?;
    let deleted = repo::delete_events_before(&mut transaction, cutoff).await?;
    transaction.commit().await.map_err(AppError::internal)?;
    Ok(deleted)
}
