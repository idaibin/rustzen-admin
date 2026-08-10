use rustzen_storage::SqlitePool;
use sqlx::SqliteConnection;

use super::types::NewEvent;

#[derive(Debug, Clone)]
pub struct ProjectPolicy {
    pub project_id: String,
    pub collection_enabled: bool,
    pub allowed_origins: String,
}

pub async fn find_project_policy(
    pool: &SqlitePool,
    project_key_hash: &str,
) -> Result<Option<ProjectPolicy>, sqlx::Error> {
    sqlx::query_as::<_, ProjectPolicyRow>(
        "SELECT id, collection_enabled, allowed_origins
         FROM insights_projects
         WHERE project_key_hash = ? AND archived_at IS NULL",
    )
    .bind(project_key_hash)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|row| ProjectPolicy {
            project_id: row.id,
            collection_enabled: row.collection_enabled != 0,
            allowed_origins: row.allowed_origins,
        })
    })
}

#[derive(Debug, sqlx::FromRow)]
struct ProjectPolicyRow {
    id: String,
    collection_enabled: i64,
    allowed_origins: String,
}

pub async fn insert_event(
    connection: &mut SqliteConnection,
    event: &NewEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO insights_events (
           project_id, event_name, visitor_id, user_id, session_id, platform,
           page_path, referrer, api_path, api_method, status_code, duration_ms,
           is_error, properties, occurred_at, received_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&event.project_id)
    .bind(&event.event_name)
    .bind(&event.visitor_id)
    .bind(&event.user_id)
    .bind(&event.session_id)
    .bind(&event.platform)
    .bind(&event.page_path)
    .bind(&event.referrer)
    .bind(&event.api_path)
    .bind(&event.api_method)
    .bind(event.status_code)
    .bind(event.duration_ms)
    .bind(event.is_error)
    .bind(&event.properties)
    .bind(&event.occurred_at)
    .bind(&event.received_at)
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn delete_events_before(
    connection: &mut SqliteConnection,
    cutoff: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM insights_events WHERE occurred_at < ?")
        .bind(cutoff)
        .execute(connection)
        .await
        .map(|result| result.rows_affected())
}
