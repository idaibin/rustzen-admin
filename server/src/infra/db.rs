use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing;

use crate::infra::config::CONFIG;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Configuration for the database connection pool.
///
/// This struct holds all the settings required to establish a connection
/// pool with the PostgreSQL database.
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
    /// Creates a database configuration from `DATABASE_URL` and `RUSTZEN_*` runtime config.
    fn default() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
pub async fn create_pool(config: DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    tracing::info!("Creating database connection pool...");
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await?;
    tracing::info!("Database connection pool created successfully.");
    Ok(pool)
}

/// Creates a new database connection pool using the default configuration.
///
/// # Errors
///
/// Returns a `sqlx::Error` if connecting to the database fails.
#[tracing::instrument(name = "create_default_db_pool")]
pub async fn create_default_pool() -> Result<PgPool, sqlx::Error> {
    let config = DatabaseConfig::default();
    create_pool(config).await
}

/// Tests the database connection by executing a simple query.
///
/// # Errors
///
/// Returns a `sqlx::Error` if the query fails, indicating a problem with the connection.
#[tracing::instrument(name = "test_db_connection", skip(pool))]
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
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
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running embedded database migrations...");
    MIGRATOR.run(pool).await?;
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}
