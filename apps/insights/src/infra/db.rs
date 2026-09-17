use std::error::Error;

use rustzen_storage::{
    SqlitePool, connect_sqlite_with_options, database_url_from_path, test_connection,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

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

pub async fn verify_selected_database() -> Result<(), String> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(config::CONFIG.database_path())
                .create_if_missing(false)
                .read_only(true),
        )
        .await
        .map_err(|_| "Insights selected database is unavailable")?;
    let result = verify_selected_schema(&pool).await;
    pool.close().await;
    result
}

pub async fn bind_selected_identity(pool: &SqlitePool) -> Result<(), String> {
    let identity = selected_identity()?;
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM rustzen_installation_identity")
        .fetch_one(pool)
        .await
        .map_err(|_| "Insights installation identity table is unavailable")?;
    if count != 0 {
        return Err("Insights installation identity is already bound".into());
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
    .map_err(|_| "Insights installation identity binding failed")?
    .rows_affected();
    if changed == 1 { Ok(()) } else { Err("Insights installation identity binding failed".into()) }
}

async fn verify_selected_schema(pool: &SqlitePool) -> Result<(), String> {
    verify_selected_identity(pool).await?;
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Insights migration ledger is unavailable")?;
    let expected = MIGRATOR.iter().collect::<Vec<_>>();
    if applied.len() != expected.len()
        || applied.iter().zip(expected).any(|((version, checksum, success), migration)| {
            !success
                || *version != migration.version
                || checksum.as_slice() != migration.checksum.as_ref()
        })
    {
        return Err("Insights migration ledger differs from selected schema".into());
    }
    verify_schema_shape(pool).await
}

async fn verify_selected_identity(pool: &SqlitePool) -> Result<(), String> {
    let expected = selected_identity()?;
    let observed = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, build_id, composition_id, schema_fingerprint, data_contract_id FROM rustzen_installation_identity ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Insights installation identity is unavailable")?;
    if observed.as_slice() == [(1, expected.0, expected.1, expected.2, expected.3)] {
        Ok(())
    } else {
        Err("Insights installation identity differs from selected release".into())
    }
}

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
        value("RUSTZEN_INSIGHTS_SCHEMA_FINGERPRINT")?,
        value("RUSTZEN_INSIGHTS_DATA_CONTRACT_ID")?,
    ))
}

async fn schema_inventory(
    pool: &SqlitePool,
) -> Result<Vec<(String, String, String, String)>, String> {
    let sql = "SELECT type, name, tbl_name, COALESCE(sql, '') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY type, name, tbl_name, sql";
    Ok(sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|_| "Insights selected schema inventory is unavailable")?)
}

async fn verify_schema_shape(pool: &SqlitePool) -> Result<(), String> {
    let expected_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|_| "Insights selected schema fixture is unavailable")?;
    MIGRATOR
        .run(&expected_pool)
        .await
        .map_err(|_| "Insights selected schema fixture is invalid")?;
    if schema_inventory(pool).await? != schema_inventory(&expected_pool).await? {
        return Err("Insights observed schema differs from selected schema".into());
    }
    expected_pool.close().await;
    Ok(())
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
        assert_eq!(versions, vec![1, 2]);
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
