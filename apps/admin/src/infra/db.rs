use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};
#[cfg(feature = "selected-distribution")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
#[cfg(feature = "full")]
use std::path::Path;
use std::time::Duration;
use tracing;

use crate::infra::config::CONFIG;

#[cfg(feature = "full")]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
#[cfg(feature = "selected-distribution")]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite-monitor");

#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
const NOTIFICATION_LEDGER: &str = "_sqlx_admin_notifications_migrations";

#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
fn notification_migrator() -> sqlx::migrate::Migrator {
    let mut migrator = sqlx::migrate!("./migrations/sqlite-notifications");
    migrator.dangerous_set_table_name(NOTIFICATION_LEDGER);
    migrator
}

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
    #[cfg(all(feature = "notifications", feature = "selected-distribution"))]
    {
        notification_migrator().run(pool).await?;
    }
    tracing::info!("Embedded database migrations completed successfully.");
    Ok(())
}

#[cfg(feature = "selected-distribution")]
pub async fn verify_selected_schema(pool: &SqlitePool) -> Result<(), String> {
    verify_selected_schema_for_identity(pool, &selected_identity()?).await
}

#[cfg(feature = "selected-distribution")]
async fn verify_selected_schema_for_identity(
    pool: &SqlitePool,
    identity: &(String, String, String, String),
) -> Result<(), String> {
    verify_selected_identity(pool, identity).await?;
    verify_migration_ledger(
        pool,
        "_sqlx_migrations",
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
        &MIGRATOR,
    )
    .await?;
    #[cfg(all(feature = "selected-distribution", feature = "notifications"))]
    verify_migration_ledger(
        pool,
        NOTIFICATION_LEDGER,
        "SELECT version, checksum, success FROM _sqlx_admin_notifications_migrations ORDER BY version",
        &notification_migrator(),
    )
    .await?;

    let expected_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|_| "Admin selected schema fixture is unavailable")?;
    run_migrations(&expected_pool).await.map_err(|_| "Admin selected schema fixture is invalid")?;
    if schema_inventory(pool).await? != schema_inventory(&expected_pool).await? {
        return Err("Admin observed schema differs from selected schema".into());
    }
    expected_pool.close().await;
    Ok(())
}

#[cfg(feature = "selected-distribution")]
async fn verify_migration_ledger(
    pool: &SqlitePool,
    ledger: &str,
    query: &'static str,
    migrator: &sqlx::migrate::Migrator,
) -> Result<(), String> {
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(query)
        .fetch_all(pool)
        .await
        .map_err(|_| format!("Admin migration ledger {ledger} is unavailable"))?;
    let expected = migrator.iter().collect::<Vec<_>>();
    if applied.len() != expected.len()
        || applied.iter().zip(expected).any(|((version, checksum, success), migration)| {
            !success
                || *version != migration.version
                || checksum.as_slice() != migration.checksum.as_ref()
        })
    {
        return Err(format!("Admin migration ledger {ledger} differs from selected schema"));
    }
    Ok(())
}

#[cfg(feature = "selected-distribution")]
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

#[cfg(feature = "selected-distribution")]
pub async fn bind_selected_identity(pool: &SqlitePool) -> Result<(), String> {
    let identity = selected_identity()?;
    bind_identity(pool, &identity).await
}

#[cfg(feature = "selected-distribution")]
async fn bind_identity(
    pool: &SqlitePool,
    identity: &(String, String, String, String),
) -> Result<(), String> {
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

#[cfg(feature = "selected-distribution")]
async fn verify_selected_identity(
    pool: &SqlitePool,
    expected: &(String, String, String, String),
) -> Result<(), String> {
    let observed = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, build_id, composition_id, schema_fingerprint, data_contract_id FROM rustzen_installation_identity ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Admin installation identity is unavailable")?;
    if observed.as_slice()
        == [(1, expected.0.clone(), expected.1.clone(), expected.2.clone(), expected.3.clone())]
    {
        Ok(())
    } else {
        Err("Admin installation identity differs from selected release".into())
    }
}

#[cfg(feature = "selected-distribution")]
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

#[cfg(feature = "selected-distribution")]
async fn schema_inventory(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, String, String)>, String> {
    #[cfg(feature = "notifications")]
    let query = "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master
         WHERE name NOT LIKE 'sqlite_%'
           AND name NOT IN ('_sqlx_migrations', '_sqlx_admin_notifications_migrations')
         ORDER BY type, name, tbl_name, sql";
    #[cfg(not(feature = "notifications"))]
    let query = "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master
         WHERE name NOT LIKE 'sqlite_%'
           AND name != '_sqlx_migrations'
         ORDER BY type, name, tbl_name, sql";
    sqlx::query_as(query)
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

    #[tokio::test]
    async fn default_full_creates_the_selected_inbox_owner() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("full migration");
        let objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name LIKE 'notification%' ORDER BY name",
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
                "notifications",
            ]
        );
    }
}

#[cfg(all(test, feature = "selected-distribution"))]
mod monitor_distribution_tests {
    use super::*;
    use crate::{
        features::auth::service::AuthService,
        infra::{password::PasswordUtils, permission::PermissionService},
    };

    async fn fresh_schema_objects() -> (SqlitePool, Vec<String>) {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("minimal migration");
        let objects = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("schema inventory");
        (pool, objects)
    }

    async fn assert_access_and_monitor_objects(pool: &SqlitePool, objects: &[String]) {
        for required in [
            "users",
            "roles",
            "menus",
            "user_roles",
            "role_menus",
            "access_policy_state",
            "access_sessions",
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
                .fetch_all(pool)
                .await
                .expect("selected modules"),
            ["monitor"]
        );
    }

    #[cfg(not(feature = "notifications"))]
    #[tokio::test]
    async fn pure_monitor_schema_has_no_notification_tables_or_ledger() {
        let (pool, objects) = fresh_schema_objects().await;
        assert_access_and_monitor_objects(&pool, &objects).await;
        for excluded in [
            "notification_accounting",
            "notifications",
            "notification_receipts",
            "notification_recipients",
            "notification_user_state",
            "_sqlx_admin_notifications_migrations",
        ] {
            assert!(!objects.iter().any(|name| name == excluded), "unexpected {excluded}");
        }
    }

    #[cfg(feature = "notifications")]
    #[tokio::test]
    async fn monitor_notify_schema_has_notification_tables_and_second_ledger() {
        let (pool, objects) = fresh_schema_objects().await;
        assert_access_and_monitor_objects(&pool, &objects).await;
        for required in [
            "notification_accounting",
            "notifications",
            "notification_receipts",
            "notification_recipients",
            "notification_user_state",
            NOTIFICATION_LEDGER,
        ] {
            assert!(objects.iter().any(|name| name == required), "missing {required}");
        }
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

    #[tokio::test]
    async fn file_backed_selected_schema_survives_bind_validate_and_reopen() {
        let path = std::env::temp_dir()
            .join(format!("rustzen-admin-selected-schema-{}.db", uuid::Uuid::new_v4()));
        let identity = ("a".repeat(64), "b".repeat(64), "c".repeat(64), "d".repeat(64));
        let connect = |create_if_missing| {
            SqlitePoolOptions::new().max_connections(2).connect_with(
                SqliteConnectOptions::new().filename(&path).create_if_missing(create_if_missing),
            )
        };

        let pool = connect(true).await.expect("file-backed selected pool");
        run_migrations(&pool).await.expect("selected migrations");
        bind_identity(&pool, &identity).await.expect("selected identity bind");
        verify_selected_schema_for_identity(&pool, &identity)
            .await
            .expect("selected schema validation");
        pool.close().await;

        let reopened = connect(false).await.expect("reopened selected pool");
        verify_selected_schema_for_identity(&reopened, &identity)
            .await
            .expect("reopened selected schema validation");
        #[cfg(feature = "notifications")]
        {
            sqlx::query("DELETE FROM _sqlx_admin_notifications_migrations")
                .execute(&reopened)
                .await
                .expect("tamper notification migration ledger");
            let error = verify_selected_schema_for_identity(&reopened, &identity)
                .await
                .expect_err("tampered notification ledger must fail");
            assert!(error.contains(NOTIFICATION_LEDGER), "unexpected validation error: {error}");
            sqlx::query("DROP TABLE _sqlx_admin_notifications_migrations")
                .execute(&reopened)
                .await
                .expect("remove notification migration ledger");
            let error = verify_selected_schema_for_identity(&reopened, &identity)
                .await
                .expect_err("missing notification ledger must fail");
            assert!(error.contains(NOTIFICATION_LEDGER), "unexpected validation error: {error}");
        }
        #[cfg(not(feature = "notifications"))]
        {
            sqlx::query(
                "CREATE TABLE _sqlx_admin_notifications_migrations (version INTEGER PRIMARY KEY)",
            )
            .execute(&reopened)
            .await
            .expect("inject unselected notification ledger");
            let error = verify_selected_schema_for_identity(&reopened, &identity)
                .await
                .expect_err("unselected notification ledger must fail");
            assert_eq!(error, "Admin observed schema differs from selected schema");
        }
        reopened.close().await;
        std::fs::remove_file(&path).expect("remove selected database fixture");
    }
}
