use super::{accounting::Accounting, realtime::RealtimeHub, retention};
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;
use std::{sync::Arc, time::Duration};

const PERIOD: Duration = Duration::from_secs(60 * 60);
const ROUND_TIME_LIMIT: Duration = Duration::from_millis(50);
type Clock = Arc<dyn Fn() -> NaiveDateTime + Send + Sync>;

pub(crate) struct Maintenance {
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Maintenance {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Maintenance {
    pub(crate) async fn shutdown(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

pub(crate) async fn start(
    pool: SqlitePool,
    realtime: RealtimeHub,
) -> Result<Maintenance, sqlx::Error> {
    start_with(pool, PERIOD, Arc::new(|| Utc::now().naive_utc()), Some(realtime)).await
}

async fn start_with(
    pool: SqlitePool,
    period: Duration,
    clock: Clock,
    realtime: Option<RealtimeHub>,
) -> Result<Maintenance, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    Accounting::validate(&mut connection).await?;
    drop(connection);
    run_round(&pool, clock(), realtime.as_ref()).await?;
    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = run_round(&pool, clock(), realtime.as_ref()).await {
                tracing::warn!(%error, "Notification retention round failed");
            }
        }
    });
    Ok(Maintenance { task })
}

async fn run_round(
    pool: &SqlitePool,
    now: NaiveDateTime,
    realtime: Option<&RealtimeHub>,
) -> Result<(), sqlx::Error> {
    run_round_with_finish(pool, now, realtime, true).await.map(|_| ())
}

async fn run_round_with_finish(
    pool: &SqlitePool,
    now: NaiveDateTime,
    realtime: Option<&RealtimeHub>,
    commit: bool,
) -> Result<retention::CleanupResult, sqlx::Error> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    let result = retention::cleanup(&mut transaction, now, ROUND_TIME_LIMIT).await?;
    if commit {
        transaction.commit().await?;
    } else {
        transaction.rollback().await?;
    }
    if commit && let Some(realtime) = realtime {
        for &(user_id, revision) in &result.invalidations {
            realtime.publish(user_id, revision);
        }
    }
    Ok(result)
}

#[cfg(test)]
pub(super) async fn start_for_test(
    pool: SqlitePool,
    period: Duration,
    now: NaiveDateTime,
    realtime: Option<RealtimeHub>,
) -> Result<Maintenance, sqlx::Error> {
    start_with(pool, period, Arc::new(move || now), realtime).await
}

#[cfg(test)]
pub(super) async fn run_round_for_test(
    pool: &SqlitePool,
    now: NaiveDateTime,
    realtime: &RealtimeHub,
    commit: bool,
) -> Result<retention::CleanupResult, sqlx::Error> {
    run_round_with_finish(pool, now, Some(realtime), commit).await
}
