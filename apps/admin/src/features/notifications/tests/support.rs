use crate::features::notifications::{
    admission_types::{AdmissionEvent, AdmissionPolicy},
    types::InboxListQuery,
};
use chrono::NaiveDateTime;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{path::PathBuf, time::Duration};
use uuid::Uuid;

pub(super) const SECRET: &[u8] = b"notification-test-secret";

pub(super) struct TestDatabase {
    pub(super) path: PathBuf,
    pub(super) primary: SqlitePool,
    pub(super) peer: SqlitePool,
}

impl TestDatabase {
    pub(super) async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("rustzen-notifications-{}.db", Uuid::new_v4()));
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
        crate::infra::db::run_migrations(&primary).await.unwrap();
        sqlx::query("UPDATE users SET status = 1 WHERE id = 1").execute(&primary).await.unwrap();
        sqlx::query(
            "INSERT OR IGNORE INTO users
             (id, username, email, password_hash, real_name, status, is_system)
             VALUES (2, 'notification-reader', 'notification-reader@example.com',
                     '!test-only!', 'Notification Reader', 1, 0)",
        )
        .execute(&primary)
        .await
        .unwrap();
        sqlx::query(
            "INSERT OR IGNORE INTO user_roles (user_id, role_id, created_at)
             VALUES (2, 2, CURRENT_TIMESTAMP)",
        )
        .execute(&primary)
        .await
        .unwrap();
        let peer = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
        crate::infra::db::run_migrations(&peer).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA journal_mode").fetch_one(&peer).await.unwrap(),
            "wal"
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>(
                "SELECT name FROM sqlite_master
                 WHERE type = 'table' AND name LIKE 'notification%' ORDER BY name",
            )
            .fetch_all(&peer)
            .await
            .unwrap(),
            [
                "notification_accounting",
                "notification_receipts",
                "notification_recipients",
                "notification_user_state",
                "notifications",
            ]
        );
        Self { path, primary, peer }
    }

    pub(super) async fn reopen(&self) -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(&self.path)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap()
    }

    pub(super) async fn close(self) {
        self.primary.close().await;
        self.peer.close().await;
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
    }
}

pub(super) fn policy() -> AdmissionPolicy {
    AdmissionPolicy {
        free_space_reserve_bytes: 0,
        wal_pressure_frames: u32::MAX,
        ..AdmissionPolicy::default()
    }
}

pub(super) fn event(
    event_id: &str,
    accepted_at: NaiveDateTime,
    candidates: Vec<i64>,
) -> AdmissionEvent {
    AdmissionEvent {
        producer: "monitor".into(),
        event_id: event_id.into(),
        payload_sha256: "a".repeat(64),
        expires_at: accepted_at + chrono::Duration::days(1),
        topic: "monitor.incident.opened".into(),
        subject_kind: "incident".into(),
        subject_id: event_id.into(),
        subject_revision: 1,
        occurred_at: accepted_at,
        title: format!("title {event_id}"),
        summary: "summary".into(),
        required_capability: "monitor:incident:view".into(),
        candidate_user_ids: candidates,
    }
}

pub(super) async fn create_users(pool: &SqlitePool, first: i64, last: i64) {
    sqlx::query(
        "WITH RECURSIVE ids(value) AS (
             SELECT ? UNION ALL SELECT value + 1 FROM ids WHERE value < ?
         ) INSERT OR IGNORE INTO users
           (id, username, email, password_hash, real_name, status, is_system)
         SELECT value, 'notification-user-' || value,
                'notification-user-' || value || '@example.com', '!test-only!',
                'Notification User', 1, 0 FROM ids",
    )
    .bind(first)
    .bind(last)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "WITH RECURSIVE ids(value) AS (
             SELECT ? UNION ALL SELECT value + 1 FROM ids WHERE value < ?
         ) INSERT OR IGNORE INTO user_roles (user_id, role_id, created_at)
         SELECT value, 2, CURRENT_TIMESTAMP FROM ids",
    )
    .bind(first)
    .bind(last)
    .execute(pool)
    .await
    .unwrap();
}

pub(super) async fn grant(pool: &SqlitePool, capability: &str) {
    sqlx::query(
        "INSERT OR IGNORE INTO menus
         (parent_id, name, code, menu_type, sort_order, status, is_system, is_manual, is_active)
         VALUES (0, 'notification test', ?, 3, 0, 1, 0, 1, 1)",
    )
    .bind(capability)
    .execute(pool)
    .await
    .unwrap();
    let menu_id =
        sqlx::query_scalar::<_, i64>("SELECT id FROM menus WHERE code = ? AND deleted_at IS NULL")
            .bind(capability)
            .fetch_one(pool)
            .await
            .unwrap();
    sqlx::query("INSERT OR IGNORE INTO role_menus (role_id, menu_id) VALUES (2, ?)")
        .bind(menu_id)
        .execute(pool)
        .await
        .unwrap();
}

pub(super) async fn revoke_all(pool: &SqlitePool) {
    sqlx::query("DELETE FROM role_menus WHERE role_id = 2").execute(pool).await.unwrap();
}

pub(super) async fn insert_notification(pool: &SqlitePool, id: &str, user_id: i64) {
    let mut transaction = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO notification_receipts
         (producer, event_id, payload_sha256, accepted_at, expires_at, retain_until, result)
         VALUES ('monitor', ?, ?, CURRENT_TIMESTAMP, datetime('now', '+1 day'), datetime('now', '+30 days'), 'stored')",
    )
    .bind(id)
    .bind("a".repeat(64))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO notifications
         (id, producer, event_id, topic, subject_kind, subject_id, subject_revision,
          occurred_at, accepted_at, title, summary, required_capability)
         VALUES (?, 'monitor', ?, 'monitor.incident.opened', 'incident', ?, 1,
                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ?, 'summary', 'monitor:incident:view')",
    )
    .bind(id)
    .bind(id)
    .bind(id)
    .bind(format!("title {id}"))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO notification_recipients (notification_id, user_id, created_at)
         VALUES (?, ?, CURRENT_TIMESTAMP)",
    )
    .bind(id)
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO notification_user_state (user_id, revision) VALUES (?, 1)
         ON CONFLICT(user_id) DO UPDATE SET revision = revision + 1",
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .unwrap();
    transaction.commit().await.unwrap();
}

pub(super) fn query(limit: Option<u16>, cursor: Option<String>) -> InboxListQuery {
    InboxListQuery { cursor, limit, unread_only: None }
}
