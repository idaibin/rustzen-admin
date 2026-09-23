use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{path::PathBuf, time::Duration};
use uuid::Uuid;

use crate::protocol::{AgentReport, ByteUsage};

pub(super) struct TestDatabase {
    pub(super) path: PathBuf,
    pub(super) primary: SqlitePool,
    pub(super) peer: SqlitePool,
}

impl TestDatabase {
    pub(super) async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("rustzen-monitor-notify-{}.db", Uuid::new_v4()));
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        let primary = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options.clone())
            .await
            .unwrap();
        crate::infra::db::migrate(&primary).await.unwrap();
        let peer = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA journal_mode").fetch_one(&peer).await.unwrap(),
            "wal"
        );
        Self { path, primary, peer }
    }

    pub(super) async fn reopen(&self) -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&self.path)
                    .foreign_keys(true)
                    .journal_mode(SqliteJournalMode::Wal)
                    .busy_timeout(Duration::from_secs(5)),
            )
            .await
            .unwrap()
    }

    pub(super) async fn close(self) {
        self.primary.close().await;
        self.peer.close().await;
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
    }
}

pub(super) fn report(
    node: &str,
    boot_id: Uuid,
    sequence: u64,
    collected_at: DateTime<Utc>,
    cpu_percent: f64,
) -> AgentReport {
    AgentReport {
        node_id: node.into(),
        boot_id,
        sequence,
        hostname: node.into(),
        agent_version: "test".into(),
        collected_at,
        cpu_percent,
        memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
        disks: vec![],
    }
}
