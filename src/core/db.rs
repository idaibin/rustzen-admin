use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing;

use crate::core::config::CONFIG;

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
    /// The timeout for an idle connection.
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    /// Creates a default database configuration from environment variables.
    ///
    /// # Panics
    ///
    /// This function will panic if the `DATABASE_URL` environment variable is not set.
    /// A valid database URL is essential for the application to run.
    fn default() -> Self {
        Self {
            url: CONFIG.db_url.to_string(),
            max_connections: CONFIG.db_max_conn,
            min_connections: CONFIG.db_min_conn,
            connect_timeout: Duration::from_secs(CONFIG.db_conn_timeout),
            idle_timeout: Duration::from_secs(CONFIG.db_idle_timeout),
        }
    }
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
