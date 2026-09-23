use rustzen_storage::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
    test_connection,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::config;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
#[cfg(feature = "notifications")]
const NOTIFICATION_LEDGER: &str = "_sqlx_monitor_notifications_migrations";
#[cfg(feature = "notifications")]
fn notification_migrator() -> sqlx::migrate::Migrator {
    let mut migrator = sqlx::migrate!("./migrations-notifications");
    migrator.dangerous_set_table_name(NOTIFICATION_LEDGER);
    migrator
}

pub async fn connect() -> Result<SqlitePool, rustzen_storage::CoreError> {
    let url = database_url_from_path(config::controller().database_path());
    connect_sqlite_with_options(
        &url,
        DatabaseConnectionOptions {
            max_connections: config::controller().database.max_connections(),
            min_connections: config::controller().database.min_connections(),
            connect_timeout: config::controller().database.connect_timeout(),
            idle_timeout: config::controller().database.idle_timeout(),
        },
    )
    .await
}

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await?;
    #[cfg(feature = "notifications")]
    notification_migrator().run(pool).await?;
    Ok(())
}

pub async fn verify(pool: &SqlitePool) -> Result<(), rustzen_storage::CoreError> {
    test_connection(pool).await
}

pub async fn verify_schema(pool: &SqlitePool) -> Result<(), String> {
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Monitor migration ledger is unavailable")?;
    let expected = MIGRATOR.iter().collect::<Vec<_>>();
    if applied.len() != expected.len()
        || applied.iter().zip(expected).any(|((version, checksum, success), migration)| {
            !success
                || *version != migration.version
                || checksum.as_slice() != migration.checksum.as_ref()
        })
    {
        return Err("Monitor migration ledger differs from selected schema".into());
    }
    #[cfg(feature = "notifications")]
    {
        let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(
            "SELECT version, checksum, success FROM _sqlx_monitor_notifications_migrations ORDER BY version",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| "Monitor notification migration ledger is unavailable")?;
        let notification_migrator = notification_migrator();
        let expected = notification_migrator.iter().collect::<Vec<_>>();
        if applied.len() != expected.len()
            || applied.iter().zip(expected).any(|((version, checksum, success), migration)| {
                !success
                    || *version != migration.version
                    || checksum.as_slice() != migration.checksum.as_ref()
            })
        {
            return Err("Monitor notification migration ledger differs from selected schema".into());
        }
    }
    verify_schema_shape(pool).await
}

async fn verify_schema_shape(pool: &SqlitePool) -> Result<(), String> {
    let expected_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|_| "Monitor selected schema fixture is unavailable")?;
    MIGRATOR.run(&expected_pool).await.map_err(|_| "Monitor selected schema fixture is invalid")?;
    #[cfg(feature = "notifications")]
    notification_migrator()
        .run(&expected_pool)
        .await
        .map_err(|_| "Monitor notification schema fixture is invalid")?;
    if schema_inventory(pool).await? != schema_inventory(&expected_pool).await? {
        return Err("Monitor observed schema differs from selected schema".into());
    }
    expected_pool.close().await;
    Ok(())
}

pub async fn verify_selected_database() -> Result<(), String> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(config::controller().database_path())
                .create_if_missing(false)
                .read_only(true),
        )
        .await
        .map_err(|_| "Monitor selected database is unavailable")?;
    let result = verify_schema(&pool).await;
    pool.close().await;
    result
}

async fn schema_inventory(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, String, String)>, String> {
    #[cfg(feature = "notifications")]
    let sql = "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' AND name != '_sqlx_monitor_notifications_migrations' ORDER BY type, name, tbl_name, sql";
    #[cfg(not(feature = "notifications"))]
    let sql = "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY type, name, tbl_name, sql";
    sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|_| "Monitor selected schema inventory is unavailable".into())
}

#[cfg(test)]
pub async fn migrated_test_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect test database");
    migrate(&pool).await.expect("migrate test database");
    pool
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "notifications"))]
    use super::verify_schema_shape;
    use super::{migrate, migrated_test_pool};
    use crate::protocol::{AgentReport, ByteUsage};
    use chrono::{TimeZone, Utc};
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    #[tokio::test]
    async fn fresh_monitor_database_migrates() {
        let pool = migrated_test_pool().await;
        pool.close().await;
    }

    #[cfg(not(feature = "notifications"))]
    #[tokio::test]
    async fn plain_monitor_rejects_notification_ledger_after_reopen() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let path = std::env::temp_dir().join(format!("rustzen-monitor-absence-{stamp}.db"));
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(SqliteConnectOptions::new().filename(&path).create_if_missing(true))
            .await
            .unwrap();
        migrate(&pool).await.unwrap();
        verify_schema_shape(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE _sqlx_monitor_notifications_migrations(version INTEGER PRIMARY KEY)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;
        let reopened = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(SqliteConnectOptions::new().filename(&path))
            .await
            .unwrap();
        assert!(verify_schema_shape(&reopened).await.is_err());
        reopened.close().await;
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn production_connection_enables_foreign_keys_and_incremental_auto_vacuum() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let path = std::env::temp_dir().join(format!("rustzen-monitor-pragmas-{stamp}.db"));
        let pool = rustzen_storage::connect_sqlite_with_options(
            &rustzen_storage::database_url_from_path(&path),
            rustzen_storage::DatabaseConnectionOptions {
                max_connections: 1,
                min_connections: 1,
                connect_timeout: std::time::Duration::from_secs(10),
                idle_timeout: None,
            },
        )
        .await
        .expect("production connection");
        migrate(&pool).await.expect("migrate production connection");

        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
                .fetch_one(&pool)
                .await
                .expect("foreign key mode"),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA auto_vacuum")
                .fetch_one(&pool)
                .await
                .expect("auto vacuum mode"),
            2
        );
        let missing_parent = sqlx::query("INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES('missing',1,1,2,'2026-09-02T00:00:00Z')")
            .execute(&pool)
            .await;
        assert!(missing_parent.is_err());

        pool.close().await;
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn baseline_initializes_only_monitoring_schema() {
        let pool = migrated_test_pool().await;
        let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("read migration ledger");
        assert_eq!(migration_count, 1);

        let column: String = sqlx::query_scalar(
            "SELECT name FROM pragma_table_info('monitor_nodes') WHERE name='last_received_at'",
        )
        .fetch_one(&pool)
        .await
        .expect("initialized monitor_nodes schema");
        assert_eq!(column, "last_received_at");

        let old_table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('monitor_metrics','monitor_settings','monitor_checks','monitor_check_results')",
        )
        .fetch_one(&pool)
        .await
        .expect("inspect initialized tables");
        assert_eq!(old_table_count, 0);

        let node_policy_table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='node_alert_settings'",
        )
        .fetch_one(&pool)
        .await
        .expect("inspect node alert settings table");
        assert_eq!(node_policy_table_count, 1);
    }

    #[tokio::test]
    async fn file_database_reopen_preserves_monitoring_state() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let path = std::env::temp_dir().join(format!("rustzen-monitor-{stamp}.db"));
        let options = SqliteConnectOptions::new().filename(&path).create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("open file database");
        migrate(&pool).await.expect("migrate file database");
        let boot = Uuid::new_v4();
        let at = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        sqlx::query("INSERT INTO monitor_nodes(node_id,hostname,agent_version,current_boot_id,last_sequence,last_report_at,last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at) VALUES('persisted','persisted','test',?,7,?,?,?,?,?,?,?)")
            .bind(boot.to_string())
            .bind(at.to_rfc3339())
            .bind(at.to_rfc3339())
            .bind(10.0_f64)
            .bind(1_i64)
            .bind(2_i64)
            .bind(at.to_rfc3339())
            .bind(at.to_rfc3339())
            .execute(&pool)
            .await
            .expect("insert persisted node");
        sqlx::query("INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES('persisted',10,1,2,?)")
            .bind(at.to_rfc3339())
            .execute(&pool)
            .await
            .expect("insert persisted sample");
        sqlx::query("INSERT INTO alert_counters(node_id,kind,target,abnormal_count,normal_count) VALUES('persisted','cpuHigh','cpu',2,0)")
            .execute(&pool)
            .await
            .expect("insert persisted counter");
        sqlx::query("INSERT INTO node_alert_settings(node_id,cpu_enabled,cpu_threshold_percent,memory_enabled,memory_threshold_percent,disk_enabled,disk_threshold_percent,offline_enabled,offline_after_seconds,updated_at) VALUES('persisted',1,88,1,89,1,90,1,120,?)")
            .bind(at.to_rfc3339())
            .execute(&pool)
            .await
            .expect("insert persisted node policy");
        sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES('persisted-memory','persisted','memoryHigh','memory','active','memory high',?,?)")
            .bind(at.to_rfc3339())
            .bind(at.to_rfc3339())
            .execute(&pool)
            .await
            .expect("insert persisted active incident");
        pool.close().await;

        let reopened = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(SqliteConnectOptions::new().filename(&path).create_if_missing(true))
            .await
            .expect("reopen file database");
        let sequence: i64 =
            sqlx::query_scalar("SELECT last_sequence FROM monitor_nodes WHERE node_id='persisted'")
                .fetch_one(&reopened)
                .await
                .expect("read persisted sequence");
        assert_eq!(sequence, 7);
        assert_eq!(
            sqlx::query_scalar::<_, f64>(
                "SELECT cpu_threshold_percent FROM alert_settings WHERE id=1"
            )
            .fetch_one(&reopened)
            .await
            .unwrap(),
            90.0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM resource_samples WHERE node_id='persisted'"
            )
            .fetch_one(&reopened)
            .await
            .unwrap(),
            1
        );
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT abnormal_count FROM alert_counters WHERE node_id='persisted' AND kind='cpuHigh'").fetch_one(&reopened).await.unwrap(), 2);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_incidents WHERE id='persisted-memory' AND status='active'").fetch_one(&reopened).await.unwrap(), 1);
        assert_eq!(
            sqlx::query_scalar::<_, f64>(
                "SELECT cpu_threshold_percent FROM node_alert_settings WHERE node_id='persisted'"
            )
            .fetch_one(&reopened)
            .await
            .unwrap(),
            88.0
        );
        let high = AgentReport {
            node_id: "persisted".into(),
            boot_id: boot,
            sequence: 8,
            hostname: "persisted".into(),
            agent_version: "test".into(),
            collected_at: at + chrono::Duration::seconds(30),
            cpu_percent: 95.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: Vec::new(),
        };
        crate::features::monitoring::record_at(&reopened, high, at + chrono::Duration::seconds(30))
            .await
            .expect("third high sample after reopen");
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_incidents WHERE node_id='persisted' AND kind='cpuHigh' AND status='active'").fetch_one(&reopened).await.unwrap(), 1);
        for sequence in 9_u64..=11 {
            let collected_at = at + chrono::Duration::seconds(30 * (sequence as i64 - 7));
            crate::features::monitoring::record_at(
                &reopened,
                AgentReport {
                    node_id: "persisted".into(),
                    boot_id: boot,
                    sequence,
                    hostname: "persisted".into(),
                    agent_version: "test".into(),
                    collected_at,
                    cpu_percent: 10.0,
                    memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
                    disks: Vec::new(),
                },
                collected_at,
            )
            .await
            .expect("normal recovery sample after reopen");
        }
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_incidents WHERE node_id='persisted' AND kind='cpuHigh' AND status='resolved'").fetch_one(&reopened).await.unwrap(), 1);
        reopened.close().await;
        let _ = std::fs::remove_file(path);
    }
}
