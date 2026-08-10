use rustzen_storage::SqlitePool;

use super::types::{CollectionPolicyRow, Settings};

pub async fn get(pool: &SqlitePool) -> Result<Settings, sqlx::Error> {
    sqlx::query_as(
        "SELECT event_retention_days, default_query_days, max_query_days,
                max_batch_events, business_timezone, updated_at
         FROM insights_settings WHERE singleton = 1",
    )
    .fetch_one(pool)
    .await
}

pub async fn get_collection_policy(pool: &SqlitePool) -> Result<CollectionPolicyRow, sqlx::Error> {
    sqlx::query_as(
        "SELECT collection_enabled, project_key_hash, allowed_origins
         FROM insights_projects WHERE id = 'default' AND archived_at IS NULL",
    )
    .fetch_one(pool)
    .await
}

pub async fn update_collection_policy(
    pool: &SqlitePool,
    collection_enabled: bool,
    project_key_hash: Option<&str>,
    allowed_origins: &str,
    updated_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE insights_projects
         SET collection_enabled = ?,
             project_key_hash = COALESCE(?, project_key_hash),
             allowed_origins = ?,
             updated_at = ?
         WHERE id = 'default' AND archived_at IS NULL",
    )
    .bind(i64::from(collection_enabled))
    .bind(project_key_hash)
    .bind(allowed_origins)
    .bind(updated_at)
    .execute(pool)
    .await
    .map(|_| ())
}
