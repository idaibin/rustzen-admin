mod relay;
mod relay_contract;
mod relay_diagnostics;
mod relay_timing;
mod support;
mod transitions;

use rustzen_storage::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{str::FromStr, time::Duration};
use uuid::Uuid;

async fn database() -> (SqlitePool, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("reports-notifications-{}.db", Uuid::new_v4()));
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .unwrap()
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
    crate::infra::db::run_migrations(&pool).await.unwrap();
    sqlx::query("INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')")
        .execute(&pool).await.unwrap();
    (pool, path)
}

async fn run(pool: &SqlitePool, id: &str, status: &str, initiator: Option<i64>) {
    sqlx::query(
        "INSERT INTO automation_runs
         (id,flow_id,status,input_json,created_at,started_at,initiator_user_id)
         VALUES(?,'flow',?,'{}','2026-09-07T00:00:00Z',
                CASE WHEN ?='running' THEN '2026-09-07T00:00:01Z' END,?)",
    )
    .bind(id)
    .bind(status)
    .bind(status)
    .bind(initiator)
    .execute(pool)
    .await
    .unwrap();
}

async fn close(pool: SqlitePool, path: std::path::PathBuf) {
    pool.close().await;
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
