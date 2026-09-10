use std::{sync::Arc, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tokio::sync::Barrier;

use super::service::AuthService;
use crate::infra::password::PasswordUtils;

#[tokio::test]
async fn concurrent_wal_writer_cannot_turn_login_into_a_busy_snapshot() {
    let path = std::env::temp_dir().join(format!("rustzen-login-wal-{}.db", uuid::Uuid::new_v4()));
    let options = || {
        SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
    };
    let primary = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options())
        .await
        .expect("primary pool");
    crate::infra::db::run_migrations(&primary).await.expect("migrations");
    let password = PasswordUtils::hash_password("ConcurrentPassw0rd!").expect("password");
    sqlx::query(
        "INSERT INTO users (id,username,email,password_hash,status)
         VALUES (80,'concurrent-login','concurrent@example.test',?,1)",
    )
    .bind(password)
    .execute(&primary)
    .await
    .expect("user");
    let writer = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options())
        .await
        .expect("writer pool");
    let barrier = Arc::new(Barrier::new(2));
    let writer_task = tokio::spawn(write_during_logins(writer.clone(), barrier.clone()));
    barrier.wait().await;
    for _ in 0..4 {
        AuthService::login(&primary, "concurrent-login", "ConcurrentPassw0rd!")
            .await
            .expect("login while writer remains active");
    }
    tokio::time::timeout(Duration::from_secs(10), writer_task)
        .await
        .expect("writer timeout")
        .expect("writer task");
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM access_sessions WHERE user_id=80 AND revoked_at IS NULL",
        )
        .fetch_one(&primary)
        .await
        .expect("sessions"),
        4
    );
    primary.close().await;
    writer.close().await;
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}

async fn write_during_logins(pool: SqlitePool, barrier: Arc<Barrier>) {
    barrier.wait().await;
    for _ in 0..100 {
        sqlx::query(if cfg!(feature = "analytics-distribution") {
            "UPDATE modules SET enabled=NOT enabled WHERE id='insights'"
        } else {
            "UPDATE modules SET enabled=NOT enabled WHERE id='monitor'"
        })
        .execute(&pool)
        .await
        .expect("concurrent writer");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
