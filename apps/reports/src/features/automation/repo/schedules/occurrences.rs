use super::super::*;

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
