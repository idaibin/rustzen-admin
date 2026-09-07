use std::path::Path;

use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};

use crate::config;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
#[cfg(feature = "notifications")]
const NOTIFICATION_LEDGER: &str = "_sqlx_reports_notifications_migrations";

#[cfg(feature = "notifications")]
fn notification_migrator() -> sqlx::migrate::Migrator {
    let mut migrator = sqlx::migrate!("./migrations-notifications");
    migrator.dangerous_set_table_name(NOTIFICATION_LEDGER);
    migrator
}

pub async fn create_pool() -> Result<SqlitePool, rustzen_storage::CoreError> {
    create_pool_for_path(&config::CONFIG.database_path()).await
}

pub async fn create_pool_for_path(path: &Path) -> Result<SqlitePool, rustzen_storage::CoreError> {
    let options = DatabaseConnectionOptions {
        max_connections: config::CONFIG.database.max_connections(),
        min_connections: config::CONFIG.database.min_connections(),
        connect_timeout: config::CONFIG.database.connect_timeout(),
        idle_timeout: config::CONFIG.database.idle_timeout(),
    };
    connect_sqlite_with_options(&database_url_from_path(path), options).await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await?;
    #[cfg(feature = "notifications")]
    notification_migrator().run(pool).await?;
    Ok(())
}

pub async fn verify_existing_database_before_write(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("Reports database metadata is unavailable".into()),
    };
    if !metadata.is_file() {
        return Err("Reports database path is not a file".into());
    }
    if metadata.len() == 0 {
        return Ok(());
    }
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(true),
        )
        .await
        .map_err(|_| "Reports existing database is unavailable")?;
    let objects = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| "Reports existing schema inventory is unavailable")?;
    let result = if objects == 0 { Ok(()) } else { verify_selected_schema(&pool).await };
    pool.close().await;
    result
}

pub async fn verify_selected_schema(pool: &SqlitePool) -> Result<(), String> {
    verify_ledger(
        pool,
        "SELECT version,checksum,success FROM _sqlx_migrations ORDER BY version",
        &MIGRATOR,
        "Reports",
    )
    .await?;
    #[cfg(feature = "notifications")]
    verify_ledger(
        pool,
        "SELECT version,checksum,success FROM _sqlx_reports_notifications_migrations ORDER BY version",
        &notification_migrator(),
        "Reports notification",
    )
    .await?;
    let expected = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|_| "Reports selected schema fixture is unavailable")?;
    run_migrations(&expected).await.map_err(|_| "Reports selected schema fixture is invalid")?;
    let matches = inventory(pool).await? == inventory(&expected).await?;
    expected.close().await;
    if matches {
        Ok(())
    } else {
        Err("Reports observed schema differs from selected schema".into())
    }
}

async fn verify_ledger(
    pool: &SqlitePool,
    query: &'static str,
    migrator: &sqlx::migrate::Migrator,
    owner: &str,
) -> Result<(), String> {
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(query)
        .fetch_all(pool)
        .await
        .map_err(|_| format!("{owner} migration ledger is unavailable"))?;
    let expected = migrator.iter().collect::<Vec<_>>();
    if applied.len() == expected.len()
        && applied.iter().zip(expected).all(|((version, checksum, success), migration)| {
            *success
                && *version == migration.version
                && checksum.as_slice() == migration.checksum.as_ref()
        })
    {
        Ok(())
    } else {
        Err(format!("{owner} migration ledger differs from selected schema"))
    }
}

async fn inventory(pool: &SqlitePool) -> Result<Vec<(String, String, String, String)>, String> {
    #[cfg(feature = "notifications")]
    let sql = "SELECT type,name,tbl_name,COALESCE(sql,'') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name NOT IN('_sqlx_migrations','_sqlx_reports_notifications_migrations') ORDER BY type,name,tbl_name,sql";
    #[cfg(not(feature = "notifications"))]
    let sql = "SELECT type,name,tbl_name,COALESCE(sql,'') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name!='_sqlx_migrations' ORDER BY type,name,tbl_name,sql";
    sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|_| "Reports selected schema inventory is unavailable".into())
}

pub use rustzen_storage::sqlite::test_connection;

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
