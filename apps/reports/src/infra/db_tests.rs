#[cfg(feature = "notifications")]
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
#[cfg(feature = "notifications")]
use uuid::Uuid;

use super::{
    MIGRATOR, run_migrations, verify_existing_database_before_write, verify_selected_schema,
};

async fn observed_objects(path: &std::path::Path) -> Vec<(String, String, String, String)> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(true),
        )
        .await
        .unwrap();
    let objects = sqlx::query_as(
        "SELECT type,name,tbl_name,COALESCE(sql,'') FROM sqlite_master
         WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name,tbl_name,sql",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    pool.close().await;
    objects
}

async fn assert_preflight_preserves_mismatched_database(path: &std::path::Path) {
    let before_bytes = std::fs::read(path).unwrap();
    let before_objects = observed_objects(path).await;
    assert!(verify_existing_database_before_write(path).await.is_err());
    assert_eq!(observed_objects(path).await, before_objects);
    assert_eq!(std::fs::read(path).unwrap(), before_bytes);
}

#[tokio::test]
async fn fresh_reports_database_migrates() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    run_migrations(&pool).await.expect("migrate");
    let schedules: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='automation_schedules'",
    )
    .fetch_one(&pool)
    .await
    .expect("inspect fresh schema");
    assert_eq!(schedules, 1);
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .expect("migration history");
    assert_eq!(versions, vec![1]);
    verify_selected_schema(&pool).await.expect("selected schema");
    #[cfg(feature = "notifications")]
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_reports_notifications_migrations")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    let initiator: String = sqlx::query_scalar(
        "SELECT name FROM pragma_table_info('automation_runs') WHERE name='initiator_user_id'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(initiator, "initiator_user_id");
}

#[cfg(not(feature = "notifications"))]
#[tokio::test]
async fn pure_reports_rejects_notification_ledger() {
    let pool =
        SqlitePoolOptions::new().max_connections(2).connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    sqlx::query("CREATE TABLE _sqlx_reports_notifications_migrations(version INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();
    assert!(verify_selected_schema(&pool).await.is_err());
}

#[cfg(not(feature = "notifications"))]
#[tokio::test]
async fn pure_startup_rejects_selected_database_before_any_write() {
    let path =
        std::env::temp_dir().join(format!("reports-pure-preflight-{}.db", uuid::Uuid::new_v4()));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&path).create_if_missing(true),
        )
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    let mut notifications = sqlx::migrate!("./migrations-notifications");
    notifications.dangerous_set_table_name("_sqlx_reports_notifications_migrations");
    notifications.run(&pool).await.unwrap();
    pool.close().await;
    assert_preflight_preserves_mismatched_database(&path).await;
    let _ = std::fs::remove_file(path);
}

#[cfg(feature = "notifications")]
#[tokio::test]
async fn selected_file_database_validates_after_reopen_and_rejects_ledger_tamper() {
    let path = std::env::temp_dir().join(format!("reports-selected-schema-{}.db", Uuid::new_v4()));
    let options = SqliteConnectOptions::new().filename(&path).create_if_missing(true);
    let pool =
        SqlitePoolOptions::new().max_connections(2).connect_with(options.clone()).await.unwrap();
    run_migrations(&pool).await.unwrap();
    verify_selected_schema(&pool).await.unwrap();
    pool.close().await;
    let reopened = SqlitePoolOptions::new().max_connections(2).connect_with(options).await.unwrap();
    verify_selected_schema(&reopened).await.unwrap();
    sqlx::query("UPDATE _sqlx_reports_notifications_migrations SET checksum=zeroblob(32)")
        .execute(&reopened)
        .await
        .unwrap();
    assert!(verify_selected_schema(&reopened).await.is_err());
    reopened.close().await;
    let _ = std::fs::remove_file(path);
}

#[cfg(feature = "notifications")]
#[tokio::test]
async fn selected_startup_rejects_pure_database_before_any_write() {
    let path =
        std::env::temp_dir().join(format!("reports-selected-preflight-{}.db", Uuid::new_v4()));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path).create_if_missing(true))
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool.close().await;
    assert_preflight_preserves_mismatched_database(&path).await;
    let _ = std::fs::remove_file(path);
}
