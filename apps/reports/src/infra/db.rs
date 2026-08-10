use std::path::Path;

use rustzen_storage::sqlite::{
    DatabaseConnectionOptions, SqlitePool, connect_sqlite_with_options, database_url_from_path,
};

use crate::config;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

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
    MIGRATOR.run(pool).await
}

pub use rustzen_storage::sqlite::test_connection;

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::MIGRATOR;

    #[tokio::test]
    async fn fresh_reports_database_migrates() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        MIGRATOR.run(&pool).await.expect("migrate");
    }

    #[test]
    fn first_reports_migration_keeps_the_published_checksum() {
        let migration =
            MIGRATOR.iter().find(|migration| migration.version == 1).expect("0001 migration");
        let checksum =
            migration.checksum.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(
            checksum,
            "55e563d9fd5ae957e944f9eef78cddecd70328e0e84164ad40347f2b0a1f0ae057d078c31242cc3525015f4d06ecef01"
        );
    }

    #[tokio::test]
    async fn existing_0001_database_upgrades_only_through_0002() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        let mut connection = pool.acquire().await.expect("acquire");
        MIGRATOR.run_direct(Some(1), &mut *connection, false).await.expect("apply published 0001");
        drop(connection);

        let before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='automation_schedules'",
        )
        .fetch_one(&pool)
        .await
        .expect("inspect legacy schema");
        assert_eq!(before, 0);

        MIGRATOR.run(&pool).await.expect("upgrade 0001 to current");
        let after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='automation_schedules'",
        )
        .fetch_one(&pool)
        .await
        .expect("inspect upgraded schema");
        assert_eq!(after, 1);
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("migration history");
        assert_eq!(versions, vec![1, 2]);
    }
}
