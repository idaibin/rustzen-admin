use super::{relay, transport::HttpTransport};
use rustzen_storage::SqlitePool;
use std::{sync::Arc, time::Duration};

pub(crate) struct RelayRuntime {
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Drop for RelayRuntime {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

impl RelayRuntime {
    pub(crate) async fn shutdown(mut self) {
        for task in &self.tasks {
            task.abort();
        }
        for task in &mut self.tasks {
            let _ = task.await;
        }
    }
}

pub(crate) async fn start(
    pool: SqlitePool,
    url: String,
    key_id: String,
    secret: &[u8],
) -> Result<RelayRuntime, Box<dyn std::error::Error>> {
    validate_accounting(&pool).await?;
    let transport = Arc::new(HttpTransport::new(url, key_id, secret)?);
    let mut tasks = Vec::new();
    for _ in 0..4 {
        let pool = pool.clone();
        let transport = transport.clone();
        tasks.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(error) = relay::run_batch(&pool, transport.as_ref(), 25).await {
                    tracing::warn!(%error, "Monitor notification relay failed");
                }
            }
        }));
    }
    Ok(RelayRuntime { tasks })
}

pub(crate) async fn validate_accounting(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let stored = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT pending_count,pending_bytes,quarantine_count,quarantine_bytes
         FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(pool)
    .await?;
    let actual = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT
           COALESCE(SUM(CASE WHEN state!='quarantined' THEN 1 ELSE 0 END),0),
           COALESCE(SUM(CASE WHEN state!='quarantined' THEN charged_bytes ELSE 0 END),0),
           COALESCE(SUM(CASE WHEN state='quarantined' THEN 1 ELSE 0 END),0),
           COALESCE(SUM(CASE WHEN state='quarantined' THEN charged_bytes ELSE 0 END),0)
         FROM notification_outbox",
    )
    .fetch_one(pool)
    .await?;
    if stored != actual {
        return Err(sqlx::Error::Protocol(
            "notification outbox accounting differs from durable rows".into(),
        ));
    }
    Ok(())
}
