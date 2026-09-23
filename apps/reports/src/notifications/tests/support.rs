use chrono::{DateTime, Duration, Utc};
use rustzen_storage::SqlitePool;
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{path::PathBuf, time::Duration as StdDuration};
use uuid::Uuid;

pub(super) struct TestDatabase {
    pub primary: SqlitePool,
    pub peer: SqlitePool,
    path: PathBuf,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let path = std::env::temp_dir().join(format!("reports-relay-{}.db", Uuid::new_v4()));
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(StdDuration::from_secs(5));
        let primary = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options.clone())
            .await
            .unwrap();
        crate::infra::db::run_migrations(&primary).await.unwrap();
        let peer = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
        Self { primary, peer, path }
    }

    pub async fn close(self) {
        self.primary.close().await;
        self.peer.close().await;
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
    }
}

pub async fn insert(
    database: &TestDatabase,
    event_id: &str,
    subject: &str,
    revision: i64,
    at: DateTime<Utc>,
) {
    insert_with_expiry(database, event_id, subject, revision, at, at + Duration::days(1), 512)
        .await;
}

pub async fn insert_with_expiry(
    database: &TestDatabase,
    event_id: &str,
    subject: &str,
    revision: i64,
    at: DateTime<Utc>,
    expires: DateTime<Utc>,
    charge: i64,
) {
    let payload = format!(r#"{{"eventId":"{event_id}"}}"#);
    sqlx::query(
        "INSERT INTO notification_outbox
         (event_id,topic,subject_kind,subject_id,subject_revision,payload_json,payload_sha256,
          occurred_at,expires_at,state,next_attempt_at,charged_bytes)
         VALUES(?,'reports.run.completed','reports-run',?,?,?,?,?,?,'pending',?,?)",
    )
    .bind(event_id)
    .bind(subject)
    .bind(revision)
    .bind(&payload)
    .bind(hex::encode(Sha256::digest(payload.as_bytes())))
    .bind(at.to_rfc3339())
    .bind(expires.to_rfc3339())
    .bind(at.to_rfc3339())
    .bind(charge)
    .execute(&database.primary)
    .await
    .unwrap();
}
