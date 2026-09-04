use super::*;

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

#[allow(clippy::too_many_arguments)]
pub async fn skip_schedule_occurrence(
    pool: &SqlitePool,
    schedule_id: &str,
    occurrence_key: &str,
    due_local: &str,
    due_at: Option<&str>,
    decided_at: &str,
    reason: &str,
    expected_revision: i64,
    expected_updated_at: &str,
    expected_effective_at: &str,
) -> Result<ScheduleSkipOutcome, sqlx::Error> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    let Some(snapshot) = sqlx::query_as::<_, ScheduleSnapshot>(
        "SELECT flow_id,input_json,enabled,revision,updated_at,effective_at
         FROM automation_schedules WHERE id=?",
    )
    .bind(schedule_id)
    .fetch_optional(&mut *transaction)
    .await?
    else {
        transaction.rollback().await?;
        return Ok(ScheduleSkipOutcome::ScheduleChanged);
    };
    if !snapshot.enabled
        || !snapshot_matches(
            &snapshot,
            expected_revision,
            expected_updated_at,
            expected_effective_at,
        )
        || !due_at_is_effective(due_at, &snapshot.effective_at)
    {
        transaction.rollback().await?;
        return Ok(ScheduleSkipOutcome::ScheduleChanged);
    }

    let inserted = sqlx::query(
        "INSERT INTO automation_schedule_occurrences
         (schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot)
         VALUES(?,?,?,?,?,'skipped',?,NULL,NULL)
         ON CONFLICT(schedule_id,occurrence_key) DO NOTHING",
    )
    .bind(schedule_id)
    .bind(occurrence_key)
    .bind(due_local)
    .bind(due_at)
    .bind(decided_at)
    .bind(reason)
    .execute(&mut *transaction)
    .await?;
    if inserted.rows_affected() == 0 {
        transaction.rollback().await?;
        return Ok(ScheduleSkipOutcome::AlreadyDecided);
    }
    transaction.commit().await?;
    Ok(ScheduleSkipOutcome::Skipped)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScheduleEnqueueOutcome {
    Enqueued(String),
    AlreadyDecided,
    ScheduleChanged,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScheduleSkipOutcome {
    Skipped,
    AlreadyDecided,
    ScheduleChanged,
}

#[derive(Debug, FromRow)]
struct ScheduleSnapshot {
    flow_id: String,
    input_json: String,
    enabled: bool,
    revision: i64,
    updated_at: String,
    effective_at: String,
}

fn snapshot_matches(
    snapshot: &ScheduleSnapshot,
    expected_revision: i64,
    expected_updated_at: &str,
    expected_effective_at: &str,
) -> bool {
    snapshot.revision == expected_revision
        && snapshot.updated_at == expected_updated_at
        && snapshot.effective_at == expected_effective_at
}

fn due_at_is_effective(due_at: Option<&str>, effective_at: &str) -> bool {
    let Some(due_at) = due_at else {
        // A DST gap has no UTC instant. The scheduler compares its local wall
        // time against effective_at before entering this transaction.
        return true;
    };
    let Ok(due_at) = DateTime::parse_from_rfc3339(due_at) else {
        return false;
    };
    let Ok(effective_at) = DateTime::parse_from_rfc3339(effective_at) else {
        return false;
    };
    due_at.with_timezone(&Utc) >= effective_at.with_timezone(&Utc)
}

/// Atomically creates one ordinary queued run and its `enqueued` occurrence decision.
/// A conflict means another scheduler pass already decided this occurrence.
#[allow(clippy::too_many_arguments)]
pub async fn enqueue_schedule_occurrence(
    pool: &SqlitePool,
    schedule_id: &str,
    occurrence_key: &str,
    due_local: &str,
    due_at: &str,
    decided_at: &str,
    expected_revision: i64,
    expected_updated_at: &str,
    expected_effective_at: &str,
) -> Result<ScheduleEnqueueOutcome, sqlx::Error> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    let Some(snapshot) = sqlx::query_as::<_, ScheduleSnapshot>(
        "SELECT flow_id,input_json,enabled,revision,updated_at,effective_at
         FROM automation_schedules WHERE id=?",
    )
    .bind(schedule_id)
    .fetch_optional(&mut *transaction)
    .await?
    else {
        transaction.rollback().await?;
        return Ok(ScheduleEnqueueOutcome::ScheduleChanged);
    };
    if !snapshot.enabled
        || !snapshot_matches(
            &snapshot,
            expected_revision,
            expected_updated_at,
            expected_effective_at,
        )
        || !due_at_is_effective(Some(due_at), &snapshot.effective_at)
    {
        transaction.rollback().await?;
        return Ok(ScheduleEnqueueOutcome::ScheduleChanged);
    }

    let run_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO automation_runs(id,flow_id,status,input_json,created_at)
         VALUES(?,?,'queued',?,?)",
    )
    .bind(&run_id)
    .bind(&snapshot.flow_id)
    .bind(&snapshot.input_json)
    .bind(decided_at)
    .execute(&mut *transaction)
    .await?;
    let inserted = sqlx::query(
        "INSERT INTO automation_schedule_occurrences
         (schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot)
         VALUES(?,?,?,?,?,'enqueued',NULL,?,?)
         ON CONFLICT(schedule_id,occurrence_key) DO NOTHING",
    )
    .bind(schedule_id)
    .bind(occurrence_key)
    .bind(due_local)
    .bind(due_at)
    .bind(decided_at)
    .bind(&run_id)
    .bind(&run_id)
    .execute(&mut *transaction)
    .await?;
    if inserted.rows_affected() == 0 {
        transaction.rollback().await?;
        return Ok(ScheduleEnqueueOutcome::AlreadyDecided);
    }
    transaction.commit().await?;
    Ok(ScheduleEnqueueOutcome::Enqueued(run_id))
}
