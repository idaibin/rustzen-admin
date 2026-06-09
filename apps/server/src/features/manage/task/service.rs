use std::sync::Arc;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use tokio::sync::{Mutex, RwLock};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    common::{api::ApiResponse, error::ServiceError, pagination::{Pagination, PaginationQuery}},
    infra::config::CONFIG,
};

use super::{
    repo::{InsertTaskRunInput, SyncTaskInput, TaskRepository},
    types::{TaskExecutionContext, TaskExecutor, TaskItem, TaskRunItem, TaskRunQuery, TaskRunStatus, TaskTriggerType},
};

#[derive(Clone)]
pub struct TaskService {
    repo: Arc<TaskRepository>,
    catalog: Arc<RwLock<Option<TaskCatalog>>>,
    scheduler: Arc<RwLock<Option<JobScheduler>>>,
    timezone: Tz,
}

#[derive(Clone)]
struct ScheduledTask {
    task_key: &'static str,
    name: &'static str,
    description: &'static str,
    expression: &'static str,
    executor: Arc<dyn TaskExecutor>,
    run_lock: Arc<Mutex<()>>,
}

struct TaskCatalog {
    tasks: Vec<ScheduledTask>,
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
}

const TASK_SPECS: [TaskSpec; 2] = [
    TaskSpec {
        task_key: "cleanup-operation-logs-retention",
        name: "Cleanup Operation Logs",
        description: "Delete operation logs older than the configured retention days.",
        expression: "0 20 1 * * * *",
        kind: TaskKind::CleanupOperationLogs,
    },
    TaskSpec {
        task_key: "cleanup-task-runs-retention",
        name: "Cleanup Task Runs",
        description: "Delete scheduled task run records older than the configured retention days.",
        expression: "0 30 1 * * * *",
        kind: TaskKind::CleanupTaskRuns,
    },
];

impl TaskService {
    pub fn new(pool: sqlx::SqlitePool) -> Result<Self, ServiceError> {
        let timezone = CONFIG.timezone.parse::<Tz>().map_err(|_| {
            ServiceError::InvalidOperation(format!("Invalid RUSTZEN_TIMEZONE: {}", CONFIG.timezone))
        })?;
        Ok(Self {
            repo: Arc::new(TaskRepository::new(pool)),
            catalog: Arc::new(RwLock::new(None)),
            scheduler: Arc::new(RwLock::new(None)),
            timezone,
        })
    }

    pub async fn bootstrap(&self) -> Result<(), ServiceError> {
        let catalog = TaskCatalog::new(self.repo.clone());
        let mut scheduler = JobScheduler::new().await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create task scheduler: {err}"))
        })?;
        self.repo.fail_stale_running_task_runs(Utc::now()).await?;

        let mut sync_inputs = Vec::with_capacity(catalog.tasks.len());
        for task in &catalog.tasks {
            let next_run_at = self.add_job(&mut scheduler, task).await?;
            sync_inputs.push(SyncTaskInput {
                task_key: task.task_key,
                name: task.name,
                description: Some(task.description),
                cron_expression: task.expression,
                next_run_at,
            });
        }

        self.repo.sync_tasks(&sync_inputs).await?;
        scheduler.start().await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to start task scheduler: {err}"))
        })?;

        *self.catalog.write().await = Some(catalog);
        *self.scheduler.write().await = Some(scheduler);
        Ok(())
    }

    async fn add_job(
        &self,
        scheduler: &mut JobScheduler,
        task: &ScheduledTask,
    ) -> Result<Option<DateTime<Utc>>, ServiceError> {
        let service = self.clone();
        let task_key = task.task_key.to_string();
        let expression = task.expression.to_string();
        let timezone = self.timezone;
        let job = Job::new_async_tz(task.expression, timezone, move |_uuid, _lock| {
            let service = service.clone();
            let task_key = task_key.clone();
            let expression = expression.clone();
            Box::pin(async move {
                let next_run_at = match service.next_run_at_for_expression(&expression).await {
                    Ok(value) => value,
                    Err(err) => {
                        tracing::error!("Failed to calculate next run for {}: {}", task_key, err);
                        None
                    }
                };
                if let Err(err) = service.start_scheduled_task(&task_key, next_run_at).await {
                    tracing::error!("Scheduled task {} failed: {}", task_key, err);
                }
            })
        })
        .map_err(|err| ServiceError::InvalidOperation(format!("Invalid cron expression: {err}")))?;

        let job_id = scheduler.add(job).await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to register scheduled task: {err}"))
        })?;
        scheduler.next_tick_for_job(job_id).await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to read next task run time: {err}"))
        })
    }

    pub async fn list_tasks(&self) -> Result<Vec<TaskItem>, ServiceError> {
        self.repo.list_tasks().await
    }

    pub async fn list_task_runs(
        &self,
        task_key: &str,
        query: TaskRunQuery,
    ) -> Result<ApiResponse<Vec<TaskRunItem>>, ServiceError> {
        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        let (items, total) = self
            .repo
            .list_task_runs(task_key, pagination.offset.into(), pagination.limit.into())
            .await?;
        Ok(ApiResponse::new(items, Some(total)))
    }

    pub async fn run_task(&self, task_key: &str) -> Result<TaskRunItem, ServiceError> {
        self.start_task_by_key(task_key, TaskTriggerType::Manual, None).await
    }

    async fn start_scheduled_task(
        &self,
        task_key: &str,
        next_run_at: Option<DateTime<Utc>>,
    ) -> Result<TaskRunItem, ServiceError> {
        self.repo.update_task_next_run_at(task_key, next_run_at).await?;
        self.start_task_by_key(task_key, TaskTriggerType::Scheduled, Some(Utc::now()))
            .await
    }

    async fn next_run_at_for_expression(
        &self,
        expression: &str,
    ) -> Result<Option<DateTime<Utc>>, ServiceError> {
        let mut scheduler = JobScheduler::new().await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create task scheduler: {err}"))
        })?;
        let job = Job::new_async_tz(expression, self.timezone, |_uuid, _lock| Box::pin(async {}))
            .map_err(|err| ServiceError::InvalidOperation(format!("Invalid cron expression: {err}")))?;
        let job_id = scheduler.add(job).await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to register scheduled task: {err}"))
        })?;
        scheduler.next_tick_for_job(job_id).await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to read next task run time: {err}"))
        })
    }

    async fn start_task_by_key(
        &self,
        task_key: &str,
        trigger_type: TaskTriggerType,
        scheduled_for: Option<DateTime<Utc>>,
    ) -> Result<TaskRunItem, ServiceError> {
        let task = self
            .catalog
            .read()
            .await
            .as_ref()
            .ok_or_else(|| ServiceError::InvalidOperation("Task scheduler is not initialized".to_string()))?
            .get(task_key)
            .ok_or_else(|| ServiceError::NotFound(format!("Task {task_key}")))?;

        self.start_task_run(task, trigger_type, scheduled_for).await
    }

    async fn start_task_run(
        &self,
        task: ScheduledTask,
        trigger_type: TaskTriggerType,
        scheduled_for: Option<DateTime<Utc>>,
    ) -> Result<TaskRunItem, ServiceError> {
        let started_at = Utc::now();
        let guard = match task.run_lock.clone().try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => {
                if trigger_type == TaskTriggerType::Manual {
                    return Err(ServiceError::InvalidOperation(format!(
                        "Task {} is already running",
                        task.task_key
                    )));
                }
                return self
                    .repo
                    .insert_skipped_task_run(InsertTaskRunInput {
                        task_key: task.task_key,
                        trigger_type: &trigger_type,
                        status: TaskRunStatus::Skipped,
                        scheduled_for,
                        started_at,
                        finished_at: Some(started_at),
                        error_message: Some("Task is already running"),
                    })
                    .await;
            }
        };

        let run = self
            .repo
            .insert_task_run(InsertTaskRunInput {
                task_key: task.task_key,
                trigger_type: &trigger_type,
                status: TaskRunStatus::Running,
                scheduled_for,
                started_at,
                finished_at: None,
                error_message: None,
            })
            .await?;

        let repo = self.repo.clone();
        let task_key = task.task_key.to_string();
        let task_name = task.name.to_string();
        let executor = task.executor.clone();
        let run_id = run.id;

        tokio::spawn(async move {
            let ctx = TaskExecutionContext {
                task_key: task_key.clone(),
                task_name,
                trigger_type,
                scheduled_for,
            };
            let result = executor.execute(ctx).await;
            let finished_at = Utc::now();
            let (status, message) = match result {
                Ok(()) => (TaskRunStatus::Success, None),
                Err(err) => (TaskRunStatus::Failed, Some(err.to_string())),
            };

            if let Err(err) = repo
                .finish_task_run(
                    run_id,
                    &task_key,
                    trigger_type,
                    status,
                    started_at,
                    finished_at,
                    message.as_deref(),
                )
                .await
            {
                tracing::error!("Failed to finish task run {}: {}", run_id, err);
            }

            drop(guard);
        });

        Ok(run)
    }
}

impl TaskCatalog {
    fn new(repo: Arc<TaskRepository>) -> Self {
        let tasks = TASK_SPECS
            .iter()
            .map(|spec| ScheduledTask {
                task_key: spec.task_key,
                name: spec.name,
                description: spec.description,
                expression: spec.expression,
                executor: spec.kind.executor(repo.clone()),
                run_lock: Arc::new(Mutex::new(())),
            })
            .collect();
        Self { tasks }
    }

    fn get(&self, task_key: &str) -> Option<ScheduledTask> {
        self.tasks
            .iter()
            .find(|task| task.task_key == task_key)
            .cloned()
    }
}

impl TaskKind {
    fn executor(&self, repo: Arc<TaskRepository>) -> Arc<dyn TaskExecutor> {
        match self {
            TaskKind::CleanupOperationLogs => Arc::new(CleanupOperationLogsExecutor { repo }),
            TaskKind::CleanupTaskRuns => Arc::new(CleanupTaskRunsExecutor { repo }),
        }
    }
}

struct CleanupOperationLogsExecutor {
    repo: Arc<TaskRepository>,
}

#[async_trait::async_trait]
impl TaskExecutor for CleanupOperationLogsExecutor {
    async fn execute(&self, ctx: TaskExecutionContext) -> Result<(), ServiceError> {
        tracing::info!(
            task_key = %ctx.task_key,
            task_name = %ctx.task_name,
            trigger_type = ?ctx.trigger_type,
            scheduled_for = ?ctx.scheduled_for,
            "Cleaning operation logs"
        );
        let deleted = self
            .repo
            .cleanup_old_operation_logs(CONFIG.log_retention_days as i64)
            .await?;
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
        tracing::info!(
            task_key = %ctx.task_key,
            task_name = %ctx.task_name,
            trigger_type = ?ctx.trigger_type,
            scheduled_for = ?ctx.scheduled_for,
            "Cleaning task runs"
        );
        let deleted = self
            .repo
            .cleanup_old_task_runs(CONFIG.task_run_retention_days)
            .await?;
        tracing::info!(deleted, "Task run cleanup completed");
        Ok(())
    }
}
