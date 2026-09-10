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
    async fn fresh_schema_has_exact_ledger_objects_and_disabled_seed() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        MIGRATOR.run(&pool).await.expect("migrate fresh schema");
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("migration history");
        assert_eq!(versions, vec![1]);
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name LIKE 'insights_%'
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("schema tables");
        assert_eq!(tables, ["insights_events", "insights_projects", "insights_settings"]);
        let indexes: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master
             WHERE type = 'index' AND name LIKE 'idx_insights_%'
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("schema indexes");
        assert_eq!(
            indexes,
            [
                "idx_insights_events_project_api_time",
                "idx_insights_events_project_name_time",
                "idx_insights_events_project_page_time",
                "idx_insights_events_project_time",
                "idx_insights_events_project_visitor_time",
            ]
        );
        let seed: (String, i64, String, i64) = sqlx::query_as(
            "SELECT project_key_hash, collection_enabled, allowed_origins,
                    (SELECT max_batch_events FROM insights_settings WHERE singleton = 1)
             FROM insights_projects WHERE id = 'default'",
        )
        .fetch_one(&pool)
        .await
        .expect("fresh disabled seed");
        assert_eq!(seed, (String::new(), 0, "[]".to_string(), 50));
        let invalid =
            sqlx::query("UPDATE insights_projects SET collection_enabled = 2").execute(&pool).await;
        assert!(invalid.is_err(), "collection_enabled CHECK must reject invalid state");
    }
}
