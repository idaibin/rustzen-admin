use super::super::*;

pub async fn schedules(pool: &SqlitePool) -> Result<Vec<ScheduleRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,revision,created_at,updated_at
         FROM automation_schedules ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn enabled_schedules(pool: &SqlitePool) -> Result<Vec<ScheduleRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,revision,created_at,updated_at
         FROM automation_schedules WHERE enabled=1 ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn schedule(pool: &SqlitePool, id: &str) -> Result<Option<ScheduleRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,revision,created_at,updated_at
         FROM automation_schedules WHERE id=?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_schedule(
    pool: &SqlitePool,
    id: &str,
    flow_id: &str,
    cadence: &str,
    weekday: Option<u8>,
    due_time: &str,
    input_json: &str,
    description: &str,
    enabled: bool,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO automation_schedules
         (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,revision,created_at,updated_at)
         VALUES(?,?,?,?,?,?,?,?,?,0,?,?)",
    )
    .bind(id)
    .bind(flow_id)
    .bind(cadence)
    .bind(weekday.map(i64::from))
    .bind(due_time)
    .bind(input_json)
    .bind(description)
    .bind(enabled)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_schedule(
    pool: &SqlitePool,
    id: &str,
    flow_id: &str,
    cadence: &str,
    weekday: Option<u8>,
    due_time: &str,
    input_json: &str,
    description: &str,
    enabled: bool,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_schedules
         SET flow_id=?,cadence=?,weekday=?,due_time=?,input_json=?,description=?,enabled=?,effective_at=?,revision=revision+1,updated_at=?
         WHERE id=?",
    )
    .bind(flow_id)
    .bind(cadence)
    .bind(weekday.map(i64::from))
    .bind(due_time)
    .bind(input_json)
    .bind(description)
    .bind(enabled)
    .bind(now)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected() == 1)
}

pub async fn delete_schedule(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM automation_schedules WHERE id=?")
        .bind(id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() == 1)
}

pub async fn last_schedule_occurrence(
    pool: &SqlitePool,
    schedule_id: &str,
) -> Result<Option<ScheduleOccurrence>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id,schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot
         FROM automation_schedule_occurrences
         WHERE schedule_id=? ORDER BY due_local DESC,id DESC LIMIT 1",
    )
    .bind(schedule_id)
    .fetch_optional(pool)
    .await
}

pub async fn last_schedule_run(
    pool: &SqlitePool,
    schedule_id: &str,
) -> Result<Option<Run>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id,r.flow_id,
                CASE WHEN r.status='running' AND r.cancel_requested_at IS NOT NULL
                     THEN 'cancelling' ELSE r.status END AS status,
                r.input_json,r.error,r.created_at,r.started_at,r.finished_at
         FROM automation_schedule_occurrences o
         JOIN automation_runs r ON r.id=o.run_id
         WHERE o.schedule_id=?
           AND o.decision='enqueued'
           AND o.run_id IS NOT NULL
           AND o.id = (
               SELECT id FROM automation_schedule_occurrences
               WHERE schedule_id=? ORDER BY due_local DESC,id DESC LIMIT 1
           )",
    )
    .bind(schedule_id)
    .bind(schedule_id)
    .fetch_optional(pool)
    .await
}
