use super::{accounting::Accounting, retention};
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

pub(crate) async fn start(pool: SqlitePool) -> Result<Maintenance, sqlx::Error> {
    start_with(pool, PERIOD, Arc::new(|| Utc::now().naive_utc())).await
}

async fn start_with(
    pool: SqlitePool,
    period: Duration,
    clock: Clock,
) -> Result<Maintenance, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    Accounting::validate(&mut connection).await?;
    drop(connection);
    run_round(&pool, clock()).await?;
    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = run_round(&pool, clock()).await {
                tracing::warn!(%error, "Notification retention round failed");
            }
        }
    });
    Ok(Maintenance { task })
}

async fn run_round(pool: &SqlitePool, now: NaiveDateTime) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    retention::cleanup(&mut transaction, now, ROUND_TIME_LIMIT).await?;
    transaction.commit().await
}

#[cfg(test)]
pub(super) async fn start_for_test(
    pool: SqlitePool,
    period: Duration,
    now: NaiveDateTime,
) -> Result<Maintenance, sqlx::Error> {
    start_with(pool, period, Arc::new(move || now)).await
}
