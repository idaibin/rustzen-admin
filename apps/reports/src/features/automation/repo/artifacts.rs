use super::*;

pub async fn insert_artifact(
    pool: &SqlitePool,
    id: &str,
    run_id: &str,
    kind: &str,
    file_name: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO automation_artifacts(id,run_id,kind,file_name,created_at) VALUES(?,?,?,?,?)",
    )
    .bind(id)
    .bind(run_id)
    .bind(kind)
    .bind(file_name)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn upsert_live_artifact(
    pool: &SqlitePool,
    id: &str,
    run_id: &str,
    file_name: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO automation_artifacts(id,run_id,kind,file_name,created_at)
         VALUES(?,?,'live-frame',?,?)
         ON CONFLICT(id) DO UPDATE SET created_at=excluded.created_at",
    )
    .bind(id)
    .bind(run_id)
    .bind(file_name)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn artifact(
    pool: &SqlitePool,
    id: &str,
    run_id: &str,
) -> Result<Option<Artifact>, sqlx::Error> {
    sqlx::query_as("SELECT id,run_id,kind,file_name,created_at FROM automation_artifacts WHERE id=? AND run_id=?").bind(id).bind(run_id).fetch_optional(pool).await
}
pub async fn artifacts(pool: &SqlitePool, run_id: &str) -> Result<Vec<Artifact>, sqlx::Error> {
    sqlx::query_as("SELECT id,run_id,kind,file_name,created_at FROM automation_artifacts WHERE run_id=? ORDER BY created_at,id")
        .bind(run_id)
        .fetch_all(pool)
        .await
}
