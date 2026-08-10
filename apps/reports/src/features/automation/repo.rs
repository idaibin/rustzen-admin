use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use sqlx::FromRow;
use uuid::Uuid;

use super::types::*;

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

pub async fn flows(
    pool: &SqlitePool,
    system_id: Option<&str>,
) -> Result<Vec<FlowRow>, sqlx::Error> {
    sqlx::query_as("SELECT id,system_id,name,steps_json,created_at,updated_at FROM automation_flows WHERE (? IS NULL OR system_id=?) ORDER BY created_at DESC").bind(system_id).bind(system_id).fetch_all(pool).await
}

pub async fn flow_options(pool: &SqlitePool) -> Result<Vec<FlowOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT flows.id, flows.name, systems.enabled
         FROM automation_flows AS flows
         INNER JOIN automation_systems AS systems ON systems.id = flows.system_id
         ORDER BY flows.created_at DESC, flows.id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn flow(pool: &SqlitePool, id: &str) -> Result<Option<FlowRow>, sqlx::Error> {
    sqlx::query_as("SELECT id,system_id,name,steps_json,created_at,updated_at FROM automation_flows WHERE id=?").bind(id).fetch_optional(pool).await
}
pub async fn insert_flow(
    pool: &SqlitePool,
    id: &str,
    system_id: &str,
    name: &str,
    steps: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES(?,?,?,?,?,?)").bind(id).bind(system_id).bind(name).bind(steps).bind(now).bind(now).execute(pool).await?;
    Ok(())
}
pub async fn update_flow(
    pool: &SqlitePool,
    id: &str,
    system_id: &str,
    name: &str,
    steps: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_flows SET system_id=?,name=?,steps_json=?,updated_at=? WHERE id=?",
    )
    .bind(system_id)
    .bind(name)
    .bind(steps)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
}
pub async fn delete_flow(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM automation_flows WHERE id=?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() == 1)
}

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

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::{
        ScheduleEnqueueOutcome, ScheduleSkipOutcome, cancel_run, cleanup_retention,
        enqueue_schedule_occurrence, finish_cancelled, finish_run, insert_schedule,
        last_schedule_run, run, schedule, skip_schedule_occurrence, update_schedule,
    };

    #[tokio::test]
    async fn cancelling_a_running_run_keeps_it_non_terminal_until_execution_stops() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        sqlx::query(
            "INSERT INTO automation_runs(id,flow_id,status,input_json,created_at,started_at) VALUES('run','flow','running','{}','now','now')",
        )
        .execute(&pool)
        .await
        .expect("run");

        assert!(cancel_run(&pool, "run", "later").await.expect("request cancellation"));
        let status: String =
            sqlx::query_scalar("SELECT status FROM automation_runs WHERE id='run'")
                .fetch_one(&pool)
                .await
                .expect("status");
        assert_eq!(status, "running");
        assert_eq!(run(&pool, "run").await.expect("load run").expect("run").status, "cancelling");
        assert!(
            !finish_run(&pool, "run", "succeeded", None, "finished")
                .await
                .expect("reject normal finish")
        );

        finish_cancelled(&pool, "run", "finished").await.expect("finish cancellation");
        let status: String =
            sqlx::query_scalar("SELECT status FROM automation_runs WHERE id='run'")
                .fetch_one(&pool)
                .await
                .expect("status");
        assert_eq!(status, "cancelled");
    }

    #[tokio::test]
    async fn schedule_occurrence_decision_and_run_are_atomic_and_idempotent() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        sqlx::query(
            "INSERT INTO automation_schedules
             (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,created_at,updated_at)
             VALUES('schedule','flow','daily',NULL,'10:00','{}','',1,'2026-08-10T09:00:00+00:00','now','now')",
        )
        .execute(&pool)
        .await
        .expect("schedule");

        let first = enqueue_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("enqueue");
        assert!(matches!(first, ScheduleEnqueueOutcome::Enqueued(_)));
        let second = enqueue_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:45+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("idempotent enqueue");
        assert_eq!(second, ScheduleEnqueueOutcome::AlreadyDecided);

        let occurrences: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE schedule_id='schedule'",
        )
        .fetch_one(&pool)
        .await
        .expect("occurrence count");
        let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
            .fetch_one(&pool)
            .await
            .expect("run count");
        assert_eq!(occurrences, 1);
        assert_eq!(runs, 1);
        assert_eq!(
            skip_schedule_occurrence(
                &pool,
                "schedule",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                None,
                "2026-08-10T10:01:00+00:00",
                "missed",
                0,
                "now",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("idempotent skip"),
            ScheduleSkipOutcome::AlreadyDecided
        );
    }

    #[tokio::test]
    async fn schedule_snapshot_changes_never_create_runs_from_stale_input() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow-current','system','Current Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("current flow");
        for id in ["disabled", "updated", "deleted", "current"] {
            sqlx::query(
                "INSERT INTO automation_schedules
                 (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,created_at,updated_at)
                 VALUES(?, 'flow', 'daily', NULL, '10:00', '{}', '', 1, '2026-08-10T09:00:00+00:00', 'now', 'now')",
            )
            .bind(id)
            .execute(&pool)
            .await
            .expect("schedule");
        }
        sqlx::query(
            "UPDATE automation_schedules SET enabled=0,revision=revision+1,updated_at='disabled-v2'
             WHERE id='disabled'",
        )
        .execute(&pool)
        .await
        .expect("disable schedule");
        assert_eq!(
            enqueue_schedule_occurrence(
                &pool,
                "disabled",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                "2026-08-10T10:00:00+00:00",
                "2026-08-10T10:00:30+00:00",
                0,
                "now",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("disabled snapshot"),
            ScheduleEnqueueOutcome::ScheduleChanged
        );

        sqlx::query(
            "UPDATE automation_schedules SET input_json='{\"value\":\"new\"}',revision=revision+1,updated_at='updated-v2'
             WHERE id='updated'",
        )
        .execute(&pool)
        .await
        .expect("update schedule");
        assert_eq!(
            enqueue_schedule_occurrence(
                &pool,
                "updated",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                "2026-08-10T10:00:00+00:00",
                "2026-08-10T10:00:30+00:00",
                0,
                "now",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("updated snapshot"),
            ScheduleEnqueueOutcome::ScheduleChanged
        );

        sqlx::query("DELETE FROM automation_schedules WHERE id='deleted'")
            .execute(&pool)
            .await
            .expect("delete schedule");
        assert_eq!(
            enqueue_schedule_occurrence(
                &pool,
                "deleted",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                "2026-08-10T10:00:00+00:00",
                "2026-08-10T10:00:30+00:00",
                0,
                "now",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("deleted snapshot"),
            ScheduleEnqueueOutcome::ScheduleChanged
        );

        sqlx::query(
            "UPDATE automation_schedules SET flow_id='flow-current',input_json='{\"value\":\"current\"}',revision=revision+1,updated_at='current-v2'
             WHERE id='current'",
        )
        .execute(&pool)
        .await
        .expect("current schedule");
        let current_run = match enqueue_schedule_occurrence(
            &pool,
            "current",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            1,
            "current-v2",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("current snapshot")
        {
            ScheduleEnqueueOutcome::Enqueued(run_id) => run_id,
            other => panic!("unexpected current outcome: {other:?}"),
        };
        let current_run: (String, String) =
            sqlx::query_as("SELECT flow_id,input_json FROM automation_runs WHERE id=?")
                .bind(current_run)
                .fetch_one(&pool)
                .await
                .expect("current run snapshot");
        assert_eq!(current_run.0, "flow-current");
        assert_eq!(current_run.1, r#"{"value":"current"}"#);

        let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
            .fetch_one(&pool)
            .await
            .expect("run count");
        assert_eq!(runs, 1);
    }

    #[tokio::test]
    async fn retention_clears_live_run_fk_but_preserves_occurrence_decision_and_snapshot() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        sqlx::query(
            "INSERT INTO automation_schedules
             (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,created_at,updated_at)
             VALUES('schedule','flow','daily',NULL,'10:00','{}','',1,'2026-08-10T09:00:00+00:00','now','now')",
        )
        .execute(&pool)
        .await
        .expect("schedule");
        let outcome = enqueue_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("enqueue");
        let run_id = match outcome {
            ScheduleEnqueueOutcome::Enqueued(run_id) => run_id,
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_eq!(
            skip_schedule_occurrence(
                &pool,
                "schedule",
                "2026-08-11T10:00",
                "2026-08-11T10:00",
                Some("2026-08-11T10:00:00+00:00"),
                "2026-08-11T10:00:30+00:00",
                "missed",
                0,
                "now",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("skip latest occurrence"),
            ScheduleSkipOutcome::Skipped
        );
        assert!(last_schedule_run(&pool, "schedule").await.expect("last run").is_none());

        sqlx::query(
            "UPDATE automation_runs
             SET status='succeeded',created_at='2000-01-01T00:00:00+00:00',finished_at='2000-01-01T00:01:00+00:00'
             WHERE id=?",
        )
        .bind(&run_id)
        .execute(&pool)
        .await
        .expect("age run");
        sqlx::query(
            "INSERT INTO automation_artifacts(id,run_id,kind,file_name,created_at)
             VALUES('artifact',?,'screenshot','artifact.png','2000-01-01T00:00:00+00:00')",
        )
        .bind(&run_id)
        .execute(&pool)
        .await
        .expect("artifact");
        let (artifacts, runs) =
            cleanup_retention(&pool, "2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00")
                .await
                .expect("retention cleanup");
        assert_eq!((artifacts, runs), (1, 1));

        let occurrence: (Option<String>, Option<String>, String) = sqlx::query_as(
            "SELECT run_id,run_id_snapshot,decision FROM automation_schedule_occurrences
             WHERE schedule_id='schedule' AND occurrence_key='2026-08-10T10:00'",
        )
        .fetch_one(&pool)
        .await
        .expect("retained decision");
        assert_eq!(occurrence.0, None);
        assert_eq!(occurrence.1, Some(run_id));
        assert_eq!(occurrence.2, "enqueued");
        let remaining_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
            .fetch_one(&pool)
            .await
            .expect("remaining runs");
        let remaining_artifacts: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM automation_artifacts")
                .fetch_one(&pool)
                .await
                .expect("remaining artifacts");
        assert_eq!(remaining_runs, 0);
        assert_eq!(remaining_artifacts, 0);
    }

    #[tokio::test]
    async fn schedule_mutations_refresh_effective_at_and_revision() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        insert_schedule(
            &pool,
            "schedule",
            "flow",
            "daily",
            None,
            "10:00",
            "{}",
            "",
            true,
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("schedule");

        let initial =
            schedule(&pool, "schedule").await.expect("load initial").expect("initial schedule");
        assert_eq!(initial.revision, 0);
        assert_eq!(initial.effective_at, "2026-08-10T09:00:00+00:00");

        assert!(
            update_schedule(
                &pool,
                "schedule",
                "flow",
                "daily",
                None,
                "10:00",
                "{}",
                "",
                false,
                "2026-08-10T10:01:00+00:00",
            )
            .await
            .expect("disable")
        );
        let disabled =
            schedule(&pool, "schedule").await.expect("load disabled").expect("disabled schedule");
        assert!(!disabled.enabled);
        assert_eq!(disabled.revision, 1);
        assert_eq!(disabled.effective_at, "2026-08-10T10:01:00+00:00");

        assert!(
            update_schedule(
                &pool,
                "schedule",
                "flow",
                "weekly",
                Some(1),
                "11:30",
                r#"{"value":"new"}"#,
                "updated",
                true,
                "2026-08-10T10:02:00+00:00",
            )
            .await
            .expect("re-enable and update")
        );
        let reenabled =
            schedule(&pool, "schedule").await.expect("load reenabled").expect("reenabled schedule");
        assert!(reenabled.enabled);
        assert_eq!(reenabled.cadence, "weekly");
        assert_eq!(reenabled.weekday, Some(1));
        assert_eq!(reenabled.due_time, "11:30");
        assert_eq!(reenabled.input_json, r#"{"value":"new"}"#);
        assert_eq!(reenabled.revision, 2);
        assert_eq!(reenabled.effective_at, "2026-08-10T10:02:00+00:00");
    }

    #[tokio::test]
    async fn enqueue_rejects_a_due_slot_before_effective_at_without_side_effects() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        insert_schedule(
            &pool,
            "schedule",
            "flow",
            "daily",
            None,
            "10:00",
            "{}",
            "",
            true,
            "2026-08-10T10:00:30+00:00",
        )
        .await
        .expect("schedule");

        assert_eq!(
            enqueue_schedule_occurrence(
                &pool,
                "schedule",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                "2026-08-10T10:00:00+00:00",
                "2026-08-10T10:00:30+00:00",
                0,
                "2026-08-10T09:00:00+00:00",
                "2026-08-10T10:00:30+00:00",
            )
            .await
            .expect("stale due decision"),
            ScheduleEnqueueOutcome::ScheduleChanged
        );
        let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
            .fetch_one(&pool)
            .await
            .expect("run count");
        let occurrences: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE schedule_id='schedule'",
        )
        .fetch_one(&pool)
        .await
        .expect("occurrence count");
        assert_eq!((runs, occurrences), (0, 0));
    }

    #[tokio::test]
    async fn skip_rechecks_enabled_revision_updated_at_and_delete_before_writing() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        sqlx::query(
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
             VALUES('system','System','https://example.com',1,'','now','now')",
        )
        .execute(&pool)
        .await
        .expect("system");
        sqlx::query(
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
             VALUES('flow','system','Flow','[]','now','now')",
        )
        .execute(&pool)
        .await
        .expect("flow");
        for id in ["disabled", "updated", "deleted", "ok"] {
            insert_schedule(
                &pool,
                id,
                "flow",
                "daily",
                None,
                "10:00",
                "{}",
                "",
                true,
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("schedule");
        }
        sqlx::query(
            "UPDATE automation_schedules
             SET enabled=0,revision=revision+1,effective_at='2026-08-10T10:01:00+00:00',updated_at='disabled-v2'
             WHERE id='disabled'",
        )
        .execute(&pool)
        .await
        .expect("disable");
        sqlx::query(
            "UPDATE automation_schedules
             SET input_json='{\"value\":\"new\"}',revision=revision+1,effective_at='2026-08-10T10:01:00+00:00',updated_at='updated-v2'
             WHERE id='updated'",
        )
        .execute(&pool)
        .await
        .expect("update");
        sqlx::query("DELETE FROM automation_schedules WHERE id='deleted'")
            .execute(&pool)
            .await
            .expect("delete");

        for id in ["disabled", "updated", "deleted"] {
            let outcome = skip_schedule_occurrence(
                &pool,
                id,
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                Some("2026-08-10T10:00:00+00:00"),
                "2026-08-10T10:00:30+00:00",
                "missed",
                0,
                "2026-08-10T09:00:00+00:00",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("stale skip outcome");
            assert_eq!(outcome, ScheduleSkipOutcome::ScheduleChanged, "{id}");
        }
        assert_eq!(
            skip_schedule_occurrence(
                &pool,
                "ok",
                "2026-08-10T10:00",
                "2026-08-10T10:00",
                Some("2026-08-10T10:00:00+00:00"),
                "2026-08-10T10:00:30+00:00",
                "missed",
                0,
                "2026-08-10T09:00:00+00:00",
                "2026-08-10T09:00:00+00:00",
            )
            .await
            .expect("valid skip"),
            ScheduleSkipOutcome::Skipped
        );
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE decision='skipped'",
        )
        .fetch_one(&pool)
        .await
        .expect("skip count");
        assert_eq!(count, 1);
    }
}
