use std::{path::Path, time::Duration};

use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};

use crate::infra::config::CONFIG;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Option<Duration>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let path = CONFIG.admin_database_path();
        Self {
            url: database_url_from_path(path.as_path()),
            max_connections: CONFIG.database.max_connections(),
            min_connections: CONFIG.database.min_connections(),
            connect_timeout: CONFIG.database.connect_timeout(),
            idle_timeout: CONFIG.database.idle_timeout(),
        }
    }
}

#[tracing::instrument(name = "create_db_pool", skip_all)]
pub async fn create_pool(config: DatabaseConfig) -> Result<SqlitePool, rustzen_storage::CoreError> {
    let options = DatabaseConnectionOptions {
        max_connections: config.max_connections,
        min_connections: config.min_connections,
        connect_timeout: config.connect_timeout,
        idle_timeout: config.idle_timeout,
    };
    connect_sqlite_with_options(&config.url, options).await
}

#[tracing::instrument(name = "create_default_db_pool")]
pub async fn create_default_pool() -> Result<SqlitePool, rustzen_storage::CoreError> {
    create_pool(DatabaseConfig::default()).await
}

pub async fn create_pool_for_path(path: &Path) -> Result<SqlitePool, rustzen_storage::CoreError> {
    create_pool(DatabaseConfig {
        url: database_url_from_path(path),
        max_connections: CONFIG.database.max_connections(),
        min_connections: CONFIG.database.min_connections(),
        connect_timeout: CONFIG.database.connect_timeout(),
        idle_timeout: CONFIG.database.idle_timeout(),
    })
    .await
}

pub use rustzen_storage::sqlite::test_connection;

#[tracing::instrument(name = "run_db_migrations", skip(pool))]
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fresh_schema_excludes_removed_dictionary_storage() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("full migration");
        let objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE name = 'dicts' OR name LIKE 'idx_dicts_%'",
        )
        .fetch_all(&pool)
        .await
        .expect("dictionary object inventory");
        assert!(objects.is_empty(), "unexpected dictionary storage: {objects:?}");
    }

    #[tokio::test]
    async fn full_schema_contains_notification_tables() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("full migration");
        let objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE 'notification%' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("notification object inventory");
        assert_eq!(
            objects,
            [
                "notification_accounting",
                "notification_receipts",
                "notification_recipients",
                "notification_user_state",
                "notifications"
            ]
        );
    }
}
