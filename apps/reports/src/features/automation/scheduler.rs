use std::{sync::Arc, time::Duration};

use chrono::{TimeDelta, Utc};
use tokio::sync::Semaphore;

use crate::{app::AppState, common::error::AppError};

use super::{browser, repo, service};

pub async fn initialize(state: &AppState) -> Result<(), AppError> {
    let recovered = repo::recover_runs(&state.pool, &Utc::now().to_rfc3339()).await?;
    if recovered > 0 {
        tracing::warn!(recovered, "Recovered interrupted report runs");
    }
    Ok(())
}

pub fn spawn(state: AppState) {
    let state = Arc::new(state);
    spawn_runs(Arc::clone(&state));
    spawn_cleanup(state);
}

fn spawn_runs(state: Arc<AppState>) {
    let semaphore = Arc::new(Semaphore::new(state.max_concurrency));
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let Ok(Some(id)) = repo::next_queued(&state.pool).await else { continue };
            let Ok(permit) = Arc::clone(&semaphore).try_acquire_owned() else { continue };
            if !repo::claim_run(&state.pool, &id, &Utc::now().to_rfc3339()).await.unwrap_or(false) {
                continue;
            }
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                let _permit = permit;
                if let Err(error) = execute_run(&state, &id).await {
                    tracing::error!(run_id=%id,%error,"Report run failed");
                }
            });
        }
    });
}

async fn execute_run(state: &AppState, id: &str) -> Result<(), AppError> {
    let run = service::run(&state.pool, id).await?;
    let flow = service::flow(&state.pool, &run.flow_id).await?;
    let system = service::system(&state.pool, &flow.system_id).await?;
    if !system.enabled {
        finish_run_or_cancellation(
            &state.pool,
            id,
            "failed",
            Some("target system is disabled"),
            &Utc::now().to_rfc3339(),
        )
        .await?;
        return Ok(());
    }
    let settings = repo::settings(&state.pool).await?;
    let result = tokio::time::timeout(
        Duration::from_secs(settings.max_run_timeout_seconds as u64),
        browser::execute(state, &run, &flow, &system),
    )
    .await;
    let cancellation_requested = repo::run_cancel_requested(&state.pool, id).await?;
    if cancellation_requested || matches!(&result, Ok(Err(AppError::Cancelled))) {
        repo::finish_cancelled(&state.pool, id, &Utc::now().to_rfc3339()).await?;
        return Ok(());
    }
    let (status, error) = match result {
        Ok(Ok(())) => ("succeeded", None),
        Ok(Err(error)) => ("failed", Some(error.to_string())),
        Err(_) => ("failed", Some("run timed out".into())),
    };
    finish_run_or_cancellation(&state.pool, id, status, error.as_deref(), &Utc::now().to_rfc3339())
        .await?;
    Ok(())
}

async fn finish_run_or_cancellation(
    pool: &rustzen_storage::SqlitePool,
    id: &str,
    status: &str,
    error: Option<&str>,
    now: &str,
) -> Result<(), AppError> {
    if repo::finish_run(pool, id, status, error, now).await? {
        return Ok(());
    }
    if repo::run_cancel_requested(pool, id).await? {
        repo::finish_cancelled(pool, id, now).await?;
        return Ok(());
    }
    Err(AppError::Conflict("run is no longer running".into()))
}

fn spawn_cleanup(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            if let Err(error) = cleanup_once(&state).await {
                tracing::error!(%error,"Reports retention cleanup failed");
            }
        }
    });
}

pub async fn cleanup_once(state: &AppState) -> Result<(u64, u64), AppError> {
    let settings = repo::settings(&state.pool).await?;
    let now = Utc::now();
    let artifact_cutoff = (now - TimeDelta::days(settings.artifact_retention_days)).to_rfc3339();
    let run_cutoff = (now - TimeDelta::days(settings.run_retention_days)).to_rfc3339();
    let expired = repo::expired_artifacts(&state.pool, &artifact_cutoff).await?;
    for artifact in &expired {
        let _ = tokio::fs::remove_file(
            state.output_dir.join(&artifact.run_id).join(&artifact.file_name),
        )
        .await;
    }
    Ok(repo::cleanup_retention(&state.pool, &artifact_cutoff, &run_cutoff).await?)
}
