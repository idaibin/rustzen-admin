use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};
#[cfg(feature = "monitor-distribution")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
#[cfg(feature = "full")]
use std::path::Path;
use std::time::Duration;
use tracing;

use crate::infra::config::CONFIG;

#[cfg(feature = "full")]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
#[cfg(feature = "monitor-distribution")]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite-monitor");

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
    MIGRATOR.run(pool).await?;
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}

#[cfg(feature = "monitor-distribution")]
pub async fn verify_selected_schema(pool: &SqlitePool) -> Result<(), String> {
    verify_selected_identity(pool).await?;
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Admin migration ledger is unavailable")?;
    let expected = MIGRATOR.iter().collect::<Vec<_>>();
    if applied.len() != expected.len()
        || applied.iter().zip(expected).any(|((version, checksum, success), migration)| {
            !success
                || *version != migration.version
                || checksum.as_slice() != migration.checksum.as_ref()
        })
    {
        return Err("Admin migration ledger differs from selected schema".into());
    }
    let expected_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|_| "Admin selected schema fixture is unavailable")?;
    MIGRATOR.run(&expected_pool).await.map_err(|_| "Admin selected schema fixture is invalid")?;
    if schema_inventory(pool).await? != schema_inventory(&expected_pool).await? {
        return Err("Admin observed schema differs from selected schema".into());
    }
    expected_pool.close().await;
    Ok(())
}

#[cfg(feature = "monitor-distribution")]
pub async fn verify_selected_database() -> Result<(), String> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(CONFIG.admin_database_path())
                .create_if_missing(false)
                .read_only(true),
        )
        .await
        .map_err(|_| "Admin selected database is unavailable")?;
    let result = verify_selected_schema(&pool).await;
    pool.close().await;
    result
}

#[cfg(feature = "monitor-distribution")]
pub async fn bind_selected_identity(pool: &SqlitePool) -> Result<(), String> {
    let identity = selected_identity()?;
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM rustzen_installation_identity")
        .fetch_one(pool)
        .await
        .map_err(|_| "Admin installation identity table is unavailable")?;
    if count != 0 {
        return Err("Admin installation identity is already bound".into());
    }
    let changed = sqlx::query(
        "INSERT INTO rustzen_installation_identity (id, build_id, composition_id, schema_fingerprint, data_contract_id) VALUES (1, ?, ?, ?, ?)",
    )
    .bind(&identity.0)
    .bind(&identity.1)
    .bind(&identity.2)
    .bind(&identity.3)
    .execute(pool)
    .await
    .map_err(|_| "Admin installation identity binding failed")?
    .rows_affected();
    if changed == 1 { Ok(()) } else { Err("Admin installation identity binding failed".into()) }
}

#[cfg(feature = "monitor-distribution")]
async fn verify_selected_identity(pool: &SqlitePool) -> Result<(), String> {
    let expected = selected_identity()?;
    let observed = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, build_id, composition_id, schema_fingerprint, data_contract_id FROM rustzen_installation_identity ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Admin installation identity is unavailable")?;
    if observed.as_slice() == [(1, expected.0, expected.1, expected.2, expected.3)] {
        Ok(())
    } else {
        Err("Admin installation identity differs from selected release".into())
    }
}

#[cfg(feature = "monitor-distribution")]
fn selected_identity() -> Result<(String, String, String, String), String> {
    let value = |name| {
        std::env::var(name)
            .ok()
            .filter(|value| {
                value.len() == 64
                    && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            .ok_or_else(|| format!("{name} is invalid"))
    };
    Ok((
        value("RUSTZEN_BUILD_ID")?,
        value("RUSTZEN_COMPOSITION_ID")?,
        value("RUSTZEN_ADMIN_SCHEMA_FINGERPRINT")?,
        value("RUSTZEN_ADMIN_DATA_CONTRACT_ID")?,
    ))
}

#[cfg(feature = "monitor-distribution")]
async fn schema_inventory(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, String, String)>, String> {
    sqlx::query_as(
        "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY type, name, tbl_name, sql",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Admin selected schema inventory is unavailable".into())
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
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT status FROM users WHERE username = 'owner'")
                .fetch_one(&pool)
                .await
                .expect("disabled owner"),
            2
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE username IN ('admin', 'viewer')"
            )
            .fetch_one(&pool)
            .await
            .expect("no default accounts"),
            0
        );
        sqlx::query("UPDATE users SET password_hash = ?, status = 1 WHERE username = 'owner'")
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
