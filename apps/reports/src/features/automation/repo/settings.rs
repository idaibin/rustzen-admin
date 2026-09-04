use super::*;

pub async fn settings(pool: &SqlitePool) -> Result<Settings, sqlx::Error> {
    sqlx::query_as("SELECT run_retention_days,artifact_retention_days,default_step_timeout_seconds,max_run_timeout_seconds,updated_at FROM automation_settings WHERE singleton=1").fetch_one(pool).await
}
pub async fn expired_artifacts(
    pool: &SqlitePool,
    cutoff: &str,
) -> Result<Vec<Artifact>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id,run_id,kind,file_name,created_at FROM automation_artifacts WHERE created_at<?",
    )
    .bind(cutoff)
    .fetch_all(pool)
    .await
}

pub async fn cleanup_retention(
    pool: &SqlitePool,
    artifact_cutoff: &str,
    run_cutoff: &str,
) -> Result<(u64, u64), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let artifacts = sqlx::query("DELETE FROM automation_artifacts WHERE created_at<?")
        .bind(artifact_cutoff)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    let runs = sqlx::query("DELETE FROM automation_runs WHERE created_at<? AND status IN('succeeded','failed','cancelled')")
        .bind(run_cutoff)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    transaction.commit().await?;
    Ok((artifacts, runs))
}
