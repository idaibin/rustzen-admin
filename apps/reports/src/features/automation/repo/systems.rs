use super::*;

pub async fn systems(pool: &SqlitePool) -> Result<Vec<System>, sqlx::Error> {
    sqlx::query_as("SELECT id,name,base_url,enabled,notes,created_at,updated_at FROM automation_systems ORDER BY created_at DESC").fetch_all(pool).await
}
pub async fn system(pool: &SqlitePool, id: &str) -> Result<Option<System>, sqlx::Error> {
    sqlx::query_as("SELECT id,name,base_url,enabled,notes,created_at,updated_at FROM automation_systems WHERE id=?").bind(id).fetch_optional(pool).await
}
#[allow(clippy::too_many_arguments)]
pub async fn insert_system(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    base_url: &str,
    enabled: bool,
    notes: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES(?,?,?,?,?,?,?)").bind(id).bind(name).bind(base_url).bind(enabled).bind(notes).bind(now).bind(now).execute(pool).await?;
    Ok(())
}
pub async fn update_system(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    base_url: &str,
    enabled: bool,
    notes: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_systems SET name=?,base_url=?,enabled=?,notes=?,updated_at=? WHERE id=?",
    )
    .bind(name)
    .bind(base_url)
    .bind(enabled)
    .bind(notes)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
}
pub async fn delete_system(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM automation_systems WHERE id=?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() == 1)
}
