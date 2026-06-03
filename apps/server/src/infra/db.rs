use rustzen_storage::{
    migration,
    sqlite::{
        DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
    },
};
use std::time::Duration;
use tracing;

use crate::infra::config::CONFIG;

/// Configuration for the database connection pool.
///
/// This struct holds all the settings required to establish a SQLite
/// connection pool.
#[derive(Debug)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
    /// The maximum number of connections the pool is allowed to maintain.
    pub max_connections: u32,
    /// The minimum number of connections the pool should maintain.
    pub min_connections: u32,
    /// The timeout for a single connection attempt.
    pub connect_timeout: Duration,
    /// The timeout for an idle connection. `None` disables idle reaping.
    pub idle_timeout: Option<Duration>,
}

impl Default for DatabaseConfig {
    /// Creates a database configuration from `RUSTZEN_*` runtime config.
    fn default() -> Self {
        let path = CONFIG.sqlite_database_path();
        let url = database_url_from_path(path.as_path());

        Self {
            url,
            max_connections: CONFIG.db_max_conn,
            min_connections: CONFIG.db_min_conn,
            connect_timeout: Duration::from_secs(CONFIG.db_conn_timeout),
            idle_timeout: db_idle_timeout(CONFIG.db_idle_timeout),
        }
    }
}

fn db_idle_timeout(timeout_secs: u64) -> Option<Duration> {
    if timeout_secs == 0 { None } else { Some(Duration::from_secs(timeout_secs)) }
}

/// Creates a new database connection pool based on the provided configuration.
///
/// # Errors
///
/// Returns a `sqlx::Error` if connecting to the database fails.
#[tracing::instrument(name = "create_db_pool", skip_all)]
pub async fn create_pool(config: DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    tracing::info!("Creating database connection pool...");
    tracing::debug!("Connecting to SQLite URL: {}", config.url);
    let options = DatabaseConnectionOptions {
        max_connections: config.max_connections,
        min_connections: config.min_connections,
        connect_timeout: config.connect_timeout,
        idle_timeout: config.idle_timeout,
    };
    let pool = connect_sqlite_with_options(&config.url, options).await?;
    tracing::info!("Database connection pool created successfully.");
    Ok(pool)
}

/// Creates a new database connection pool using the default configuration.
///
/// # Errors
///
/// Returns a `sqlx::Error` if connecting to the database fails.
#[tracing::instrument(name = "create_default_db_pool")]
pub async fn create_default_pool() -> Result<SqlitePool, sqlx::Error> {
    let config = DatabaseConfig::default();
    create_pool(config).await
}

pub use rustzen_storage::sqlite::test_connection;

/// Runs embedded database migrations on startup.
///
/// # Errors
///
/// Returns a migration error if applying the embedded migrations fails.
#[tracing::instrument(name = "run_db_migrations", skip(pool))]
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running embedded database migrations...");
    migration::run_migrations(pool).await?;
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::db_idle_timeout;
    use std::time::Duration;

    #[test]
    fn db_idle_timeout_disables_reaping_for_zero() {
        assert_eq!(db_idle_timeout(0), None);
    }

    #[test]
    fn db_idle_timeout_uses_seconds_for_positive_values() {
        assert_eq!(db_idle_timeout(600), Some(Duration::from_secs(600)));
    }
}
