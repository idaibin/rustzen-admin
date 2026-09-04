use super::{repo, types::*, validation::required};
use crate::common::error::AppError;
use chrono::Utc;
use rustzen_storage::SqlitePool;
use url::Url;
use uuid::Uuid;

pub async fn systems(pool: &SqlitePool) -> Result<Vec<System>, AppError> {
    Ok(repo::systems(pool).await?)
}
pub async fn create_system(pool: &SqlitePool, input: SaveSystem) -> Result<System, AppError> {
    let (id, name, url, enabled, notes, now) = validated_system(input)?;
    repo::insert_system(pool, &id, &name, &url, enabled, &notes, &now).await?;
    system(pool, &id).await
}
pub async fn update_system(
    pool: &SqlitePool,
    id: &str,
    input: SaveSystem,
) -> Result<System, AppError> {
    let (_, name, url, enabled, notes, now) = validated_system(input)?;
    if !repo::update_system(pool, id, &name, &url, enabled, &notes, &now).await? {
        return Err(AppError::NotFound("system not found".into()));
    }
    system(pool, id).await
}
pub async fn system(pool: &SqlitePool, id: &str) -> Result<System, AppError> {
    repo::system(pool, id).await?.ok_or_else(|| AppError::NotFound("system not found".into()))
}
pub async fn delete_system(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    match repo::delete_system(pool, id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::NotFound("system not found".into())),
        Err(sqlx::Error::Database(error)) if error.is_foreign_key_violation() => {
            Err(AppError::Conflict("system is still referenced".into()))
        }
        Err(error) => Err(error.into()),
    }
}
fn validated_system(
    input: SaveSystem,
) -> Result<(String, String, String, bool, String, String), AppError> {
    let name = required(input.name, "name", 100)?;
    let parsed = Url::parse(input.base_url.trim())
        .map_err(|_| AppError::InvalidInput("baseUrl must be an absolute HTTP/HTTPS URL".into()))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::InvalidInput("baseUrl must be an HTTP/HTTPS origin".into()));
    }
    Ok((
        Uuid::new_v4().to_string(),
        name,
        parsed.origin().ascii_serialization(),
        input.enabled.unwrap_or(true),
        input.notes.unwrap_or_default().trim().chars().take(1000).collect(),
        Utc::now().to_rfc3339(),
    ))
}
