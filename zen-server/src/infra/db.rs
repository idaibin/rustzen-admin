use sqlx::{
    sqlite::SqlitePoolOptions,
    SqlitePool,
};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing;

use crate::infra::config::CONFIG;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");

/// Configuration for the database connection pool.
///
/// This struct holds all the settings required to establish a connection
/// pool with the configured storage backend.
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
        assert!(
            CONFIG.storage == "sqlite",
            "Unsupported storage backend `{}`. V2 backend currently supports only `sqlite`.",
            CONFIG.storage
        );

        let path = &CONFIG.sqlite_path;
        let url = sqlite_connection_url(path);

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
    ensure_database_directory(&config.url)?;
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await?;
    tracing::info!("Database connection pool created successfully.");
    Ok(pool)
}

fn ensure_database_directory(database_url: &str) -> Result<(), sqlx::Error> {
    let db_path = if let Some(path) = database_url.strip_prefix("sqlite://") {
        path
    } else {
        database_url
    };

    if db_path == ":memory:" {
        return Ok(());
    }

    if let Some(parent) = Path::new(db_path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }
    Ok(())
}

fn sqlite_connection_url(path: &str) -> String {
    if path == ":memory:" || path.starts_with("sqlite:") {
        return path.to_string();
    }

    let absolute_path = to_absolute_path(path);
    format!("sqlite:///{}", absolute_path.display())
}

fn to_absolute_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(candidate))
            .unwrap_or_else(|_| candidate.to_path_buf())
    }
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

/// Tests the database connection by executing a simple query.
///
/// # Errors
///
/// Returns a `sqlx::Error` if the query fails, indicating a problem with the connection.
#[tracing::instrument(name = "test_db_connection", skip(pool))]
pub async fn test_connection(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    tracing::debug!("Executing database connection test query...");
    sqlx::query("SELECT 1").execute(pool).await?;
    tracing::info!("Database connection test successful.");
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

/// Runs embedded database migrations on startup.
///
/// # Errors
///
/// Returns a migration error if applying the embedded migrations fails.
#[tracing::instrument(name = "run_db_migrations", skip(pool))]
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running embedded database migrations...");
    MIGRATOR.run(pool).await?;
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}
