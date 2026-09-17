use chrono::{DateTime, Utc};

use crate::common::error::ServiceError;

use super::super::types::{TaskItem, TaskRow, TaskRunStatus, TaskSchedule, TaskTriggerType};
use super::{TaskRepository, map_db_error};

pub struct SyncTaskInput<'a> {
    pub task_key: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub cron_expression: &'a str,
    pub next_run_at: Option<DateTime<Utc>>,
}

impl TaskRepository {
    pub async fn sync_tasks(&self, tasks: &[SyncTaskInput<'_>]) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;
        for task in tasks {
            sqlx::query(
                "INSERT INTO system_tasks (task_key,name,description,schedule_type,schedule_json,enabled,next_run_at,created_at,updated_at) VALUES (?, ?, ?, 'cron', ?, 1, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT(task_key) DO UPDATE SET name=excluded.name,description=excluded.description,schedule_type=excluded.schedule_type,schedule_json=excluded.schedule_json,enabled=1,next_run_at=excluded.next_run_at,updated_at=CURRENT_TIMESTAMP",
            )
            .bind(task.task_key)
            .bind(task.name)
            .bind(task.description)
            .bind(task.cron_expression)
            .bind(task.next_run_at)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }
        tx.commit().await.map_err(map_db_error)
    }

    pub async fn list_tasks(&self) -> Result<Vec<TaskItem>, ServiceError> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            "SELECT task_key,name,description,enabled,running,last_run_id,last_trigger_type,last_status,last_started_at,last_finished_at,last_error_message,next_run_at,schedule_json,created_at,updated_at FROM system_tasks ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(row_to_task_item).collect()
    }

    pub async fn update_task_next_run_at(
        &self,
        task_key: &str,
        next_run_at: Option<DateTime<Utc>>,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE system_tasks SET next_run_at=?,updated_at=CURRENT_TIMESTAMP WHERE task_key=?",
        )
        .bind(next_run_at)
        .bind(task_key)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }
}

fn row_to_task_item(row: TaskRow) -> Result<TaskItem, ServiceError> {
    Ok(TaskItem {
        task_key: row.task_key,
        name: row.name,
        description: row.description,
        enabled: row.enabled != 0,
        schedule: TaskSchedule::Cron { expression: row.schedule_json },
        running: row.running != 0,
        last_run_id: row.last_run_id,
        last_trigger_type: row
            .last_trigger_type
            .as_deref()
            .map(trigger_type_from_str)
            .transpose()?,
        last_status: row.last_status.as_deref().map(task_status_from_str).transpose()?,
        last_started_at: row.last_started_at,
        last_finished_at: row.last_finished_at,
        last_error_message: row.last_error_message,
        next_run_at: row.next_run_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub(super) fn trigger_type_to_str(value: &TaskTriggerType) -> &'static str {
    match value {
        TaskTriggerType::Scheduled => "scheduled",
        TaskTriggerType::Manual => "manual",
    }
}

pub(super) fn task_status_to_str(value: TaskRunStatus) -> &'static str {
    match value {
        TaskRunStatus::Running => "running",
        TaskRunStatus::Success => "success",
        TaskRunStatus::Failed => "failed",
        TaskRunStatus::Skipped => "skipped",
    }
}

pub(super) fn trigger_type_from_str(value: &str) -> Result<TaskTriggerType, ServiceError> {
    match value {
        "scheduled" => Ok(TaskTriggerType::Scheduled),
        "manual" => Ok(TaskTriggerType::Manual),
        _ => Err(ServiceError::InvalidOperation(format!("Invalid task trigger type: {value}"))),
    }
}

pub(super) fn task_status_from_str(value: &str) -> Result<TaskRunStatus, ServiceError> {
    match value {
        "running" => Ok(TaskRunStatus::Running),
        "success" => Ok(TaskRunStatus::Success),
        "failed" => Ok(TaskRunStatus::Failed),
        "skipped" => Ok(TaskRunStatus::Skipped),
        _ => Err(ServiceError::InvalidOperation(format!("Invalid task status: {value}"))),
    }
}
