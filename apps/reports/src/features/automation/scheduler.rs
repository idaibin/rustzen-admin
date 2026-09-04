use std::{sync::Arc, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use tokio::{
    sync::{Semaphore, watch},
    task::{JoinHandle, JoinSet},
};

use crate::{app::AppState, common::error::AppError};

use super::{browser, repo, service};

mod calendar;

use calendar::{
    DueDecision, decide_due_occurrence, latest_due_occurrence, occurrence_is_effective,
    parse_schedule_timezone,
};

pub(crate) use calendar::next_due_at;

const SCHEDULE_POLL_INTERVAL: Duration = Duration::from_secs(15);

pub async fn initialize(state: &AppState) -> Result<(), AppError> {
    parse_schedule_timezone(crate::config::CONFIG.timezone())?;
    let recovered = repo::recover_runs(&state.pool, &Utc::now().to_rfc3339()).await?;
    if recovered > 0 {
        tracing::warn!(recovered, "Recovered interrupted report runs");
    }
    Ok(())
}

pub struct Workers {
    pub shutdown: watch::Sender<bool>,
    handles: Vec<JoinHandle<()>>,
}

impl Workers {
    pub async fn shutdown(self) {
        self.shutdown.send_replace(true);
        for handle in self.handles {
            if let Err(error) = handle.await {
                tracing::error!(%error, "Reports worker failed during shutdown");
            }
        }
    }
}

pub fn spawn(state: AppState) -> Workers {
    let state = Arc::new(state);
    let (shutdown, receiver) = watch::channel(false);
    let handles = vec![
        spawn_runs(Arc::clone(&state), receiver.clone()),
        spawn_schedules(Arc::clone(&state), receiver.clone()),
        spawn_cleanup(state, receiver),
    ];
    Workers { shutdown, handles }
}

fn spawn_schedules(state: Arc<AppState>, mut shutdown: watch::Receiver<bool>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if *shutdown.borrow() {
                break;
            }
            if let Err(error) = process_schedules_once(&state, Utc::now()).await {
                tracing::error!(%error, "Reports schedule polling failed");
            }
            tokio::select! {
                _ = shutdown.wait_for(|stopped| *stopped) => break,
                _ = tokio::time::sleep(SCHEDULE_POLL_INTERVAL) => {}
            }
        }
    })
}

/// Evaluate all enabled schedules once. The database uniqueness constraint is the
/// process-restart and multi-poll idempotency boundary; this function deliberately
/// does not keep an in-memory occurrence cache.
pub async fn process_schedules_once(state: &AppState, now: DateTime<Utc>) -> Result<(), AppError> {
    let timezone = parse_schedule_timezone(crate::config::CONFIG.timezone())?;
    for schedule in repo::enabled_schedules(&state.pool).await? {
        let Some(occurrence) = latest_due_occurrence(now, &schedule, &timezone)? else {
            continue;
        };
        if !occurrence_is_effective(&occurrence, &schedule.effective_at, &timezone) {
            continue;
        }
        let decision = decide_due_occurrence(now, &occurrence)?;
        match decision {
            DueDecision::Enqueue => {
                // A missing or malformed flow is rejected without creating a decision.
                // This keeps invalid data retryable and avoids inventing a run.
                if let Err(error) = service::flow(&state.pool, &schedule.flow_id).await {
                    tracing::warn!(schedule_id=%schedule.id, %error, "Skipped invalid scheduled report flow");
                    continue;
                }
                let Some(due_at) = occurrence.due_at.as_deref() else {
                    continue;
                };
                let outcome = repo::enqueue_schedule_occurrence(
                    &state.pool,
                    &schedule.id,
                    &occurrence.occurrence_key,
                    &occurrence.due_local,
                    due_at,
                    &now.to_rfc3339(),
                    schedule.revision,
                    &schedule.updated_at,
                    &schedule.effective_at,
                )
                .await?;
                if let repo::ScheduleEnqueueOutcome::Enqueued(run_id) = outcome {
                    tracing::info!(schedule_id=%schedule.id, run_id=%run_id, "Enqueued scheduled report run");
                }
            }
            DueDecision::SkipMissed | DueDecision::SkipDstGap => {
                let reason = match decision {
                    DueDecision::SkipMissed => "missed",
                    DueDecision::SkipDstGap => "dst_gap",
                    DueDecision::Enqueue => unreachable!(),
                };
                let inserted = repo::skip_schedule_occurrence(
                    &state.pool,
                    &schedule.id,
                    &occurrence.occurrence_key,
                    &occurrence.due_local,
                    occurrence.due_at.as_deref(),
                    &now.to_rfc3339(),
                    reason,
                    schedule.revision,
                    &schedule.updated_at,
                    &schedule.effective_at,
                )
                .await?;
                if matches!(inserted, repo::ScheduleSkipOutcome::Skipped) {
                    tracing::info!(schedule_id=%schedule.id, reason, "Skipped scheduled report occurrence");
                }
            }
        }
    }
    Ok(())
}

fn spawn_runs(state: Arc<AppState>, mut shutdown: watch::Receiver<bool>) -> JoinHandle<()> {
    let semaphore = Arc::new(Semaphore::new(state.max_concurrency));
    tokio::spawn(async move {
        let mut runs = JoinSet::new();
        loop {
            tokio::select! {
                biased;
                _ = shutdown.wait_for(|stopped| *stopped) => break,
                result = runs.join_next(), if !runs.is_empty() => {
                    if let Some(Err(error)) = result {
                        tracing::error!(%error, "Report execution task failed");
                    }
                    continue;
                }
                _ = tokio::time::sleep(Duration::from_millis(250)) => {}
            }
            let Ok(Some(id)) = repo::next_queued(&state.pool).await else { continue };
            if *shutdown.borrow() {
                break;
            }
            let Ok(permit) = Arc::clone(&semaphore).try_acquire_owned() else { continue };
            if !repo::claim_run(&state.pool, &id, &Utc::now().to_rfc3339()).await.unwrap_or(false) {
                continue;
            }
            let state = Arc::clone(&state);
            let shutdown = shutdown.clone();
            runs.spawn(async move {
                let _permit = permit;
                if let Err(error) = execute_run(&state, &id, shutdown).await {
                    tracing::error!(run_id=%id,%error,"Report run failed");
                    let _ = finish_run_or_cancellation(
                        &state.pool,
                        &id,
                        "failed",
                        Some(&error.to_string()),
                        &Utc::now().to_rfc3339(),
                    )
                    .await;
                }
            });
        }
        while let Some(result) = runs.join_next().await {
            if let Err(error) = result {
                tracing::error!(%error, "Report execution task failed during shutdown");
            }
        }
    })
}

async fn execute_run(
    state: &AppState,
    id: &str,
    shutdown: watch::Receiver<bool>,
) -> Result<(), AppError> {
    if *shutdown.borrow() {
        return Err(AppError::Interrupted);
    }
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
    // The browser owns its deadline so timeout still closes the process and
    // removes the run profile before this run becomes terminal.
    let result = browser::execute(
        state,
        &run,
        &flow,
        &system,
        Duration::from_secs(settings.max_run_timeout_seconds as u64),
        shutdown,
    )
    .await;
    let cancellation_requested = repo::run_cancel_requested(&state.pool, id).await?;
    if cancellation_requested || matches!(&result, Err(AppError::Cancelled)) {
        repo::finish_cancelled(&state.pool, id, &Utc::now().to_rfc3339()).await?;
        return Ok(());
    }
    let (status, error) = match result {
        Ok(()) => ("succeeded", None),
        Err(error) => ("failed", Some(error.to_string())),
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

fn spawn_cleanup(state: Arc<AppState>, mut shutdown: watch::Receiver<bool>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.wait_for(|stopped| *stopped) => break,
                _ = tokio::time::sleep(Duration::from_secs(3600)) => {}
            }
            if let Err(error) = cleanup_once(&state).await {
                tracing::error!(%error,"Reports retention cleanup failed");
            }
        }
    })
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
