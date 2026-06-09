use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::error::ServiceError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TaskSchedule {
    Cron { expression: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskTriggerType {
    Scheduled,
    Manual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskRunStatus {
    Running,
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskItem {
    pub task_key: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub schedule: TaskSchedule,
    pub running: bool,
    pub last_run_id: Option<i64>,
    pub last_trigger_type: Option<TaskTriggerType>,
    pub last_status: Option<TaskRunStatus>,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_finished_at: Option<DateTime<Utc>>,
    pub last_error_message: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunItem {
    pub id: i64,
    pub task_key: String,
    pub trigger_type: TaskTriggerType,
    pub status: TaskRunStatus,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRow {
    pub task_key: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: i64,
    pub running: i64,
    pub last_run_id: Option<i64>,
    pub last_trigger_type: Option<String>,
    pub last_status: Option<String>,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_finished_at: Option<DateTime<Utc>>,
    pub last_error_message: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub schedule_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRunRow {
    pub id: i64,
    pub task_key: String,
    pub trigger_type: String,
    pub status: String,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TaskExecutionContext {
    pub task_key: String,
    pub task_name: String,
    pub trigger_type: TaskTriggerType,
    pub scheduled_for: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, ctx: TaskExecutionContext) -> Result<(), ServiceError>;
}
