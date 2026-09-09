use std::sync::Arc;

use rustzen_storage::{SqliteMaintenancePlan, run_sqlite_maintenance};
use tokio::sync::Mutex;

use crate::{common::error::ServiceError, features::manage::log::service::LogService};

use super::{
    repo::TaskRepository,
    types::{TaskExecutionContext, TaskExecutor},
};

#[derive(Clone)]
pub(super) struct ScheduledTask {
    pub(super) task_key: &'static str,
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) expression: &'static str,
    pub(super) executor: Arc<dyn TaskExecutor>,
    pub(super) run_lock: Arc<Mutex<()>>,
}

pub(super) struct TaskCatalog {
    pub(super) tasks: Vec<ScheduledTask>,
}

#[derive(Clone, Copy)]
struct TaskSpec {
    task_key: &'static str,
    name: &'static str,
    description: &'static str,
    expression: &'static str,
    kind: TaskKind,
}

#[derive(Clone, Copy)]
enum TaskKind {
    CleanupOperationLogs,
    CleanupTaskRuns,
    SqliteMaintenance,
}

const TASK_SPECS: [TaskSpec; 3] = [
    TaskSpec {
        task_key: "cleanup-operation-logs-retention",
        name: "清理操作日志",
        description: "删除超过配置保留天数的操作日志。",
        expression: "0 20 1 * * * *",
        kind: TaskKind::CleanupOperationLogs,
    },
    TaskSpec {
        task_key: "cleanup-task-runs-retention",
        name: "清理任务记录",
        description: "删除超过配置保留天数的定时任务执行记录。",
        expression: "0 30 1 * * * *",
        kind: TaskKind::CleanupTaskRuns,
    },
    TaskSpec {
        task_key: "sqlite-storage-maintenance",
        name: "SQLite 存储维护",
        description: "执行 WAL 检查点、优化 SQLite 查询规划统计并回收可复用页面。",
        expression: "0 0 2 * * * *",
        kind: TaskKind::SqliteMaintenance,
    },
];

impl TaskCatalog {
    pub(super) fn new(repo: Arc<TaskRepository>, pool: sqlx::SqlitePool) -> Self {
        Self {
            tasks: TASK_SPECS
                .iter()
                .map(|spec| ScheduledTask {
                    task_key: spec.task_key,
                    name: spec.name,
                    description: spec.description,
                    expression: spec.expression,
                    executor: spec.kind.executor(repo.clone(), pool.clone()),
                    run_lock: Arc::new(Mutex::new(())),
                })
                .collect(),
        }
    }

    pub(super) fn get(&self, task_key: &str) -> Option<ScheduledTask> {
        self.tasks.iter().find(|task| task.task_key == task_key).cloned()
    }
}

impl TaskKind {
    fn executor(&self, repo: Arc<TaskRepository>, pool: sqlx::SqlitePool) -> Arc<dyn TaskExecutor> {
        match self {
            Self::CleanupOperationLogs => Arc::new(CleanupOperationLogsExecutor { pool }),
            Self::CleanupTaskRuns => Arc::new(CleanupTaskRunsExecutor { repo }),
            Self::SqliteMaintenance => Arc::new(SqliteMaintenanceExecutor { pool }),
        }
    }
}

struct CleanupOperationLogsExecutor {
    pool: sqlx::SqlitePool,
}

#[async_trait::async_trait]
impl TaskExecutor for CleanupOperationLogsExecutor {
    async fn execute(&self, ctx: TaskExecutionContext) -> Result<(), ServiceError> {
        tracing::info!(task_key = %ctx.task_key, task_name = %ctx.task_name, trigger_type = ?ctx.trigger_type, scheduled_for = ?ctx.scheduled_for, "Cleaning operation logs");
        let deleted = LogService::cleanup_old_logs(&self.pool).await?;
        tracing::info!(deleted, "Operation log cleanup completed");
        Ok(())
    }
}

struct CleanupTaskRunsExecutor {
    repo: Arc<TaskRepository>,
}

#[async_trait::async_trait]
impl TaskExecutor for CleanupTaskRunsExecutor {
    async fn execute(&self, ctx: TaskExecutionContext) -> Result<(), ServiceError> {
        tracing::info!(task_key = %ctx.task_key, task_name = %ctx.task_name, trigger_type = ?ctx.trigger_type, scheduled_for = ?ctx.scheduled_for, "Cleaning task runs");
        let deleted = self.repo.cleanup_old_task_runs().await?;
        tracing::info!(deleted, "Task run cleanup completed");
        Ok(())
    }
}

struct SqliteMaintenanceExecutor {
    pool: sqlx::SqlitePool,
}

#[async_trait::async_trait]
impl TaskExecutor for SqliteMaintenanceExecutor {
    async fn execute(&self, ctx: TaskExecutionContext) -> Result<(), ServiceError> {
        tracing::info!(task_key = %ctx.task_key, task_name = %ctx.task_name, trigger_type = ?ctx.trigger_type, scheduled_for = ?ctx.scheduled_for, "Running SQLite storage maintenance");
        let report = run_sqlite_maintenance(&self.pool, SqliteMaintenancePlan::reclaim())
            .await
            .map_err(|err| {
                tracing::error!(%err, "SQLite storage maintenance failed");
                ServiceError::DatabaseQueryFailed
            })?;
        if let Some(checkpoint) = report.checkpoint {
            tracing::info!(
                busy = checkpoint.busy,
                log_frames = checkpoint.log_frames,
                checkpointed_frames = checkpoint.checkpointed_frames,
                "SQLite WAL checkpoint completed"
            );
        }
        tracing::info!(
            before_pages = report.before.page_count,
            before_freelist = report.before.freelist_count,
            before_freelist_bytes = report.before.freelist_bytes,
            after_pages = report.after.page_count,
            after_freelist = report.after.freelist_count,
            after_freelist_bytes = report.after.freelist_bytes,
            optimized = report.optimized,
            vacuumed = report.vacuumed,
            "SQLite storage maintenance completed"
        );
        Ok(())
    }
}
