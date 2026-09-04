use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};
#[cfg(feature = "full")]
use std::path::Path;
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
        let path = CONFIG.admin_database_path();
        let url = database_url_from_path(path.as_path());

        Self {
            url,
            max_connections: CONFIG.database.max_connections(),
            min_connections: CONFIG.database.min_connections(),
            connect_timeout: CONFIG.database.connect_timeout(),
            idle_timeout: CONFIG.database.idle_timeout(),
        }
    }
}

/// Creates a new database connection pool based on the provided configuration.
///
/// # Errors
///
/// Returns a storage error if connecting to the database fails.
#[tracing::instrument(name = "create_db_pool", skip_all)]
pub async fn create_pool(config: DatabaseConfig) -> Result<SqlitePool, rustzen_storage::CoreError> {
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
/// Returns a storage error if connecting to the database fails.
#[tracing::instrument(name = "create_default_db_pool")]
pub async fn create_default_pool() -> Result<SqlitePool, rustzen_storage::CoreError> {
    let config = DatabaseConfig::default();
    create_pool(config).await
}

#[cfg(feature = "full")]
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

/// Runs embedded database migrations on startup.
///
/// # Errors
///
/// Returns a migration error if applying the embedded migrations fails.
#[tracing::instrument(name = "run_db_migrations", skip(pool))]
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running embedded database migrations...");
    #[cfg(feature = "full")]
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
    #[cfg(feature = "monitor-distribution")]
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite-monitor");
    MIGRATOR.run(pool).await?;
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}

#[cfg(all(test, feature = "full"))]
mod full_schema_tests {
    use super::*;

    #[tokio::test]
    async fn fresh_schema_excludes_removed_dictionary_storage() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("full migration");

        let dictionary_objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE name = 'dicts' OR name LIKE 'idx_dicts_%'",
        )
        .fetch_all(&pool)
        .await
        .expect("dictionary object inventory");

        assert!(
            dictionary_objects.is_empty(),
            "unexpected dictionary storage: {dictionary_objects:?}"
        );
    }
}

#[cfg(all(test, feature = "monitor-distribution"))]
mod monitor_distribution_tests {
    use super::*;
    use crate::{
        features::auth::service::AuthService,
        infra::{password::PasswordUtils, permission::PermissionService},
    };

    #[tokio::test]
    async fn fresh_schema_contains_only_access_and_monitor_host_owners() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("minimal migration");
        let objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("schema inventory");
        for required in [
            "users",
            "roles",
            "menus",
            "user_roles",
            "role_menus",
            "modules",
            "module_navigation",
            "user_with_roles",
            "user_permissions",
            "role_with_menus",
        ] {
            assert!(objects.iter().any(|name| name == required), "missing {required}");
        }
        for excluded in
            ["dicts", "operation_logs", "system_tasks", "system_task_runs", "deploy_versions"]
        {
            assert!(!objects.iter().any(|name| name == excluded), "unexpected {excluded}");
        }
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT id FROM modules ORDER BY id")
                .fetch_all(&pool)
                .await
                .expect("selected modules"),
            ["monitor"]
        );
    }

    #[tokio::test]
    async fn fresh_schema_supports_owner_login_and_access_permissions() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("minimal migration");
        PermissionService::sync_permissions(&pool).await.expect("permission sync");
        let password_hash =
            PasswordUtils::hash_password("minimal-admin-password").expect("test password hash");
        sqlx::query("UPDATE users SET password_hash = ? WHERE username = 'owner'")
            .bind(password_hash)
            .execute(&pool)
            .await
            .expect("owner test password");

        let login = AuthService::login(&pool, "owner", "minimal-admin-password")
            .await
            .expect("minimal owner login");
        assert_eq!(login.user_info.username, "owner");
        assert!(login.user_info.permissions.iter().any(|permission| permission == "*"));
        assert!(!login.token.is_empty());
    }
}
