use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use rustzen_storage::SqlitePool;

mod liveness;
mod retention;
mod summaries;

pub(super) use liveness::offline_scan_at;
#[cfg(test)]
pub(super) use liveness::offline_scan_at_with_lock_hook;
#[cfg(test)]
pub(super) use retention::cleanup_at_with;
pub(super) use retention::{MaintenanceResult, cleanup_at};
pub(super) use summaries::{generate_daily_summaries_at, summaries};

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
