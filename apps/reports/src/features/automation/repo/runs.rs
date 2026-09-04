use super::*;

pub async fn runs(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    status: Option<&str>,
) -> Result<(Vec<Run>, i64), sqlx::Error> {
    let rows=sqlx::query_as("SELECT id,flow_id,CASE WHEN status='running' AND cancel_requested_at IS NOT NULL THEN 'cancelling' ELSE status END AS status,input_json,error,created_at,started_at,finished_at FROM automation_runs WHERE (? IS NULL OR CASE WHEN status='running' AND cancel_requested_at IS NOT NULL THEN 'cancelling' ELSE status END=?) ORDER BY created_at DESC LIMIT ? OFFSET ?").bind(status).bind(status).bind(limit).bind(offset).fetch_all(pool).await?;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_runs WHERE (? IS NULL OR CASE WHEN status='running' AND cancel_requested_at IS NOT NULL THEN 'cancelling' ELSE status END=?)",
    )
            .bind(status)
            .bind(status)
            .fetch_one(pool)
            .await?;
    Ok((rows, total))
}
pub async fn run(pool: &SqlitePool, id: &str) -> Result<Option<Run>, sqlx::Error> {
    sqlx::query_as("SELECT id,flow_id,CASE WHEN status='running' AND cancel_requested_at IS NOT NULL THEN 'cancelling' ELSE status END AS status,input_json,error,created_at,started_at,finished_at FROM automation_runs WHERE id=?").bind(id).fetch_optional(pool).await
}
pub async fn insert_run(
    pool: &SqlitePool,
    id: &str,
    flow_id: &str,
    input: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query("INSERT INTO automation_runs(id,flow_id,status,input_json,created_at) VALUES(?,?,'queued',?,?)").bind(id).bind(flow_id).bind(input).bind(now).execute(pool).await.map(|r|r.rows_affected()==1)
}
pub enum RetryRunOutcome {
    Retry(Run),
    SourceNotFound,
    SourceNotRetryable,
}

pub async fn retry_run(
    pool: &SqlitePool,
    id: &str,
    source_id: &str,
    now: &str,
) -> Result<RetryRunOutcome, sqlx::Error> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    if let Some(existing) = sqlx::query_as(
        "SELECT id,flow_id,CASE WHEN status='running' AND cancel_requested_at IS NOT NULL THEN 'cancelling' ELSE status END AS status,input_json,error,created_at,started_at,finished_at
         FROM automation_runs WHERE retry_source_run_id=?",
    )
    .bind(source_id)
    .fetch_optional(&mut *transaction)
    .await
    ? {
        transaction.commit().await?;
        return Ok(RetryRunOutcome::Retry(existing));
    }

    if let Some(created) = sqlx::query_as(
        "INSERT INTO automation_runs(id,flow_id,retry_source_run_id,status,input_json,created_at)
         SELECT ?,flow_id,?,'queued',input_json,?
         FROM automation_runs
         WHERE id=? AND status IN ('failed','cancelled')
         RETURNING id,flow_id,status,input_json,error,created_at,started_at,finished_at",
    )
    .bind(id)
    .bind(source_id)
    .bind(now)
    .bind(source_id)
    .fetch_optional(&mut *transaction)
    .await?
    {
        transaction.commit().await?;
        return Ok(RetryRunOutcome::Retry(created));
    }

    let source_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM automation_runs WHERE id=?)")
            .bind(source_id)
            .fetch_one(&mut *transaction)
            .await?;
    transaction.commit().await?;
    Ok(if source_exists {
        RetryRunOutcome::SourceNotRetryable
    } else {
        RetryRunOutcome::SourceNotFound
    })
}
pub async fn claim_run(pool: &SqlitePool, id: &str, now: &str) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_runs SET status='running',started_at=? WHERE id=? AND status='queued'",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
}
pub async fn next_queued(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT id FROM automation_runs WHERE status='queued' ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}
pub async fn finish_run(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    error: Option<&str>,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_runs SET status=?,error=?,finished_at=? WHERE id=? AND status='running' AND cancel_requested_at IS NULL",
    )
    .bind(status)
    .bind(error)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected() == 1)
}
pub async fn cancel_run(pool: &SqlitePool, id: &str, now: &str) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_runs SET
           status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,
           finished_at=CASE WHEN status='queued' THEN ? ELSE finished_at END,
           cancel_requested_at=CASE WHEN status='running' THEN ? ELSE cancel_requested_at END
         WHERE id=? AND status IN('queued','running')",
    )
    .bind(now)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
}
pub async fn run_cancel_requested(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT status='cancelled' OR cancel_requested_at IS NOT NULL FROM automation_runs WHERE id=?",
    )
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn finish_cancelled(pool: &SqlitePool, id: &str, now: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE automation_runs SET status='cancelled',finished_at=? WHERE id=? AND status='running' AND cancel_requested_at IS NOT NULL",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn recover_runs(pool: &SqlitePool, now: &str) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_runs SET
           status=CASE WHEN cancel_requested_at IS NOT NULL THEN 'cancelled' ELSE 'failed' END,
           error=CASE WHEN cancel_requested_at IS NOT NULL THEN error ELSE 'Interrupted by service restart' END,
           finished_at=?
         WHERE status='running'",
    )
    .bind(now)
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
}
#[allow(clippy::too_many_arguments)]
pub async fn insert_run_step(
    pool: &SqlitePool,
    run_id: &str,
    index: i64,
    action: &str,
    status: &str,
    duration: i64,
    message: Option<&str>,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO automation_run_steps(run_id,step_index,action,status,duration_ms,message,created_at) VALUES(?,?,?,?,?,?,?)").bind(run_id).bind(index).bind(action).bind(status).bind(duration).bind(message).bind(now).execute(pool).await?;
    Ok(())
}
pub async fn run_steps(pool: &SqlitePool, id: &str) -> Result<Vec<RunStep>, sqlx::Error> {
    sqlx::query_as("SELECT id,run_id,step_index,action,status,duration_ms,message,created_at FROM automation_run_steps WHERE run_id=? ORDER BY step_index,id").bind(id).fetch_all(pool).await
}
