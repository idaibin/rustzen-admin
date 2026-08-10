use std::error::Error;

use rustzen_storage::{
    SqlitePool, connect_sqlite_with_options, database_url_from_path, test_connection,
};

use crate::config;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn connect() -> Result<SqlitePool, Box<dyn Error + Send + Sync>> {
    let path = config::CONFIG.database_path();
    let url = database_url_from_path(&path);
    let pool = connect_sqlite_with_options(
        &url,
        rustzen_storage::DatabaseConnectionOptions {
            max_connections: config::CONFIG.database.max_connections(),
            min_connections: config::CONFIG.database.min_connections(),
            connect_timeout: config::CONFIG.database.connect_timeout(),
            idle_timeout: config::CONFIG.database.idle_timeout(),
        },
    )
    .await?;
    migrate(&pool).await?;
    test_connection(&pool).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::MIGRATOR;

    #[tokio::test]
    async fn existing_0001_database_clears_the_legacy_seed_before_enable() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        let mut connection = pool.acquire().await.expect("acquire");
        MIGRATOR.run_direct(Some(1), &mut *connection, false).await.expect("apply published 0001");
        drop(connection);

        let legacy_hash: String = sqlx::query_scalar(
            "SELECT project_key_hash FROM insights_projects WHERE id = 'default'",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy seed");
        assert_eq!(legacy_hash, "6ab538c2b9772ed3ea67476cf10035de9a31718833b1ab27c2d28c269f9a5b95");

        MIGRATOR.run(&pool).await.expect("upgrade 0001 to current");

        let upgraded_hash: String = sqlx::query_scalar(
            "SELECT project_key_hash FROM insights_projects WHERE id = 'default'",
        )
        .fetch_one(&pool)
        .await
        .expect("upgraded project key hash");
        assert!(upgraded_hash.is_empty());
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("migration history");
        assert_eq!(versions, vec![1, 2]);
    }
}
