use chrono::{DateTime, Duration, Utc};
use sqlx::{Sqlite, Transaction};

use crate::common::error::ServiceError;

use super::{
    super::types::{TaskRunItem, TaskRunRow, TaskRunStatus, TaskTriggerType},
    map_db_error,
    task_rows::{task_status_from_str, task_status_to_str, trigger_type_from_str, trigger_type_to_str},
    TaskRepository,
};

pub struct InsertTaskRunInput<'a> {
    pub task_key: &'a str,
    pub trigger_type: &'a TaskTriggerType,
    pub status: TaskRunStatus,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<&'a str>,
}

pub struct FinishTaskRunInput<'a> {
    pub run_id: i64,
    pub task_key: &'a str,
    pub trigger_type: TaskTriggerType,
    pub status: TaskRunStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub error_message: Option<&'a str>,
}

struct Summary<'a> {
    task_key: &'a str,
    run_id: i64,
    trigger_type: &'a str,
    status: &'a str,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    error_message: Option<&'a str>,
}

impl TaskRepository {
    pub async fn fail_stale_running_task_runs(&self, finished_at: DateTime<Utc>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;
        let rows: Vec<(i64, String, String, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id,task_key,trigger_type,started_at FROM system_task_runs WHERE status='running' ORDER BY started_at,id",
        )
        .fetch_all(&mut *tx).await.map_err(map_db_error)?;
        for (run_id, task_key, trigger_type, started_at) in rows {
            sqlx::query("UPDATE system_task_runs SET status='failed',finished_at=?,error_message='Task process stopped before completion',updated_at=? WHERE id=?")
                .bind(finished_at).bind(finished_at).bind(run_id).execute(&mut *tx).await.map_err(map_db_error)?;
            update_summary(&mut tx, Summary { task_key: &task_key, run_id, trigger_type: &trigger_type, status: "failed", started_at, finished_at: Some(finished_at), error_message: Some("Task process stopped before completion") }).await?;
        }
        sqlx::query("UPDATE system_tasks SET running=0,last_status=CASE WHEN last_status='running' THEN 'failed' ELSE last_status END,last_finished_at=CASE WHEN last_status='running' THEN ? ELSE last_finished_at END,last_error_message=CASE WHEN last_status='running' THEN 'Task process stopped before completion' ELSE last_error_message END,updated_at=CURRENT_TIMESTAMP WHERE running=1")
            .bind(finished_at).execute(&mut *tx).await.map_err(map_db_error)?;
        tx.commit().await.map_err(map_db_error)
    }

    pub async fn list_task_runs(&self, task_key: &str, offset: i64, limit: i64) -> Result<(Vec<TaskRunItem>, i64), ServiceError> {
        let total = sqlx::query_scalar("SELECT COUNT(*) FROM system_task_runs WHERE task_key=?")
            .bind(task_key).fetch_one(&self.pool).await.map_err(map_db_error)?;
        let rows: Vec<TaskRunRow> = sqlx::query_as("SELECT id,task_key,trigger_type,status,scheduled_for,started_at,finished_at,error_message,created_at,updated_at FROM system_task_runs WHERE task_key=? ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(task_key).bind(limit).bind(offset).fetch_all(&self.pool).await.map_err(map_db_error)?;
        Ok((rows.into_iter().map(row_to_item).collect::<Result<_, _>>()?, total))
    }

    pub async fn insert_task_run(&self, input: InsertTaskRunInput<'_>) -> Result<TaskRunItem, ServiceError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;
        let row = insert_row(&mut *tx, &input).await?;
        update_summary(&mut tx, Summary { task_key: input.task_key, run_id: row.id, trigger_type: trigger_type_to_str(input.trigger_type), status: task_status_to_str(input.status), started_at: input.started_at, finished_at: input.finished_at, error_message: input.error_message }).await?;
        tx.commit().await.map_err(map_db_error)?;
        row_to_item(row)
    }

    pub async fn insert_skipped_task_run(&self, input: InsertTaskRunInput<'_>) -> Result<TaskRunItem, ServiceError> {
        row_to_item(insert_row(&self.pool, &input).await?)
    }

    pub async fn finish_task_run(&self, input: FinishTaskRunInput<'_>) -> Result<TaskRunItem, ServiceError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;
        let row: TaskRunRow = sqlx::query_as("UPDATE system_task_runs SET status=?,finished_at=?,error_message=?,updated_at=? WHERE id=? RETURNING id,task_key,trigger_type,status,scheduled_for,started_at,finished_at,error_message,created_at,updated_at")
            .bind(task_status_to_str(input.status)).bind(input.finished_at).bind(input.error_message).bind(input.finished_at).bind(input.run_id).fetch_one(&mut *tx).await.map_err(map_db_error)?;
        update_summary(&mut tx, Summary { task_key: input.task_key, run_id: input.run_id, trigger_type: trigger_type_to_str(&input.trigger_type), status: task_status_to_str(input.status), started_at: input.started_at, finished_at: Some(input.finished_at), error_message: input.error_message }).await?;
        tx.commit().await.map_err(map_db_error)?;
        row_to_item(row)
    }

    pub async fn cleanup_old_task_runs(&self) -> Result<u64, ServiceError> {
        let cutoff = Utc::now() - Duration::days(rustzen_config::RETENTION_DAYS as i64);
        Ok(sqlx::query("DELETE FROM system_task_runs WHERE created_at<?").bind(cutoff).execute(&self.pool).await.map_err(map_db_error)?.rows_affected())
    }
}

async fn insert_row<'e, E>(executor: E, input: &InsertTaskRunInput<'_>) -> Result<TaskRunRow, ServiceError>
where E: sqlx::Executor<'e, Database = Sqlite> {
    sqlx::query_as("INSERT INTO system_task_runs (task_key,trigger_type,status,scheduled_for,started_at,finished_at,error_message,created_at,updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id,task_key,trigger_type,status,scheduled_for,started_at,finished_at,error_message,created_at,updated_at")
        .bind(input.task_key).bind(trigger_type_to_str(input.trigger_type)).bind(task_status_to_str(input.status)).bind(input.scheduled_for).bind(input.started_at).bind(input.finished_at).bind(input.error_message).bind(input.started_at).bind(input.started_at).fetch_one(executor).await.map_err(map_db_error)
}

async fn update_summary(tx: &mut Transaction<'_, Sqlite>, input: Summary<'_>) -> Result<(), ServiceError> {
    sqlx::query("UPDATE system_tasks SET running=?,last_run_id=?,last_trigger_type=?,last_status=?,last_started_at=?,last_finished_at=?,last_error_message=?,updated_at=CURRENT_TIMESTAMP WHERE task_key=?")
        .bind(if input.status == "running" { 1 } else { 0 }).bind(input.run_id).bind(input.trigger_type).bind(input.status).bind(input.started_at).bind(input.finished_at).bind(input.error_message).bind(input.task_key).execute(&mut **tx).await.map_err(map_db_error)?;
    Ok(())
}

fn row_to_item(row: TaskRunRow) -> Result<TaskRunItem, ServiceError> {
    Ok(TaskRunItem { id: row.id, task_key: row.task_key, trigger_type: trigger_type_from_str(&row.trigger_type)?, status: task_status_from_str(&row.status)?, scheduled_for: row.scheduled_for, started_at: row.started_at, finished_at: row.finished_at, error_message: row.error_message, created_at: row.created_at, updated_at: row.updated_at })
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sqlx::sqlite::SqlitePoolOptions;

    use super::TaskRepository;

    #[tokio::test]
    async fn cleanup_keeps_task_runs_inside_thirty_day_window() {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("connect");
        crate::infra::db::run_migrations(&pool).await.expect("migrate");
        sqlx::query("INSERT INTO system_tasks (task_key,name,schedule_type,schedule_json) VALUES ('retention-test','Retention test','cron','0 0 0 * * * *')").execute(&pool).await.expect("task");
        for created_at in [Utc::now() - Duration::days(31), Utc::now() - Duration::days(29)] {
            sqlx::query("INSERT INTO system_task_runs (task_key,trigger_type,status,started_at,created_at,updated_at) VALUES ('retention-test','manual','success',?,?,?)")
                .bind(created_at).bind(created_at).bind(created_at).execute(&pool).await.expect("task run");
        }
        let repo = TaskRepository::new(pool.clone());
        assert_eq!(repo.cleanup_old_task_runs().await.expect("cleanup"), 1);
        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM system_task_runs").fetch_one(&pool).await.expect("remaining");
        assert_eq!(remaining, 1);
    }
}
