use std::{str::FromStr, sync::Arc, time::Duration};

use chrono::{DateTime, FixedOffset, Utc};
use croner::Cron;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

use crate::{
    common::{
        error::ServiceError,
        pagination::{Pagination, PaginationQuery},
    },
    infra::config::CONFIG,
};

use super::{
    catalog::{ScheduledTask, TaskCatalog},
    repo::{FinishTaskRunInput, InsertTaskRunInput, SyncTaskInput, TaskRepository},
    schedule::{parse_fixed_timezone, task_run_timeout_duration},
    types::{
        TaskExecutionContext, TaskItem, TaskRunItem, TaskRunQuery, TaskRunStatus, TaskTriggerType,
    },
};

#[derive(Clone)]
pub struct TaskService {
    pool: sqlx::SqlitePool,
    repo: Arc<TaskRepository>,
    catalog: Arc<RwLock<Option<TaskCatalog>>>,
    timezone: FixedOffset,
}

impl TaskService {
    pub fn new(pool: sqlx::SqlitePool) -> Result<Self, ServiceError> {
        let timezone = parse_fixed_timezone(CONFIG.timezone())?;
        Ok(Self {
            pool: pool.clone(),
            repo: Arc::new(TaskRepository::new(pool)),
            catalog: Arc::new(RwLock::new(None)),
            timezone,
        })
    }

    pub async fn bootstrap(&self) -> Result<(), ServiceError> {
        let catalog = TaskCatalog::new(self.repo.clone(), self.pool.clone());
        self.repo.fail_stale_running_task_runs(Utc::now()).await?;

        let mut sync_inputs = Vec::with_capacity(catalog.tasks.len());
        let mut task_crons = Vec::with_capacity(catalog.tasks.len());
        for task in &catalog.tasks {
            let cron = Self::cron_from_expression(task.expression)?;
            let next_run_at = Self::next_run_at_for_cron(&cron, self.timezone)?;
            sync_inputs.push(SyncTaskInput {
                task_key: task.task_key,
                name: task.name,
                description: Some(task.description),
                cron_expression: task.expression,
                next_run_at: Some(next_run_at),
            });
            task_crons.push((task.clone(), cron));
        }

        self.repo.sync_tasks(&sync_inputs).await?;

        *self.catalog.write().await = Some(catalog);
        for (task, cron) in task_crons {
            self.schedule_task(task, cron);
        }
        Ok(())
    }

    fn schedule_task(&self, task: ScheduledTask, cron: Cron) {
        let service = self.clone();
        let timezone = self.timezone;

        tokio::spawn(async move {
            loop {
                let scheduled_for = match Self::next_run_at_for_cron(&cron, timezone) {
                    Ok(value) => value,
                    Err(err) => {
                        tracing::error!(
                            task_key = task.task_key,
                            task_name = task.name,
                            "Failed to calculate next run time"
                        );
                        tracing::debug!("Scheduling retry detail: {}", err);
                        sleep(Duration::from_secs(10)).await;
                        continue;
                    }
                };

                let wait = (scheduled_for - Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(10));
                sleep(wait).await;

                let next_run_at =
                    match Self::next_run_after_for_cron(&cron, timezone, scheduled_for) {
                        Ok(value) => Some(value),
                        Err(err) => {
                            tracing::error!(
                                task_key = task.task_key,
                                task_name = task.name,
                                "Failed to calculate following run time"
                            );
                            tracing::debug!("Following run calculation detail: {}", err);
                            None
                        }
                    };

                if let Err(err) = service
                    .start_scheduled_task(task.task_key, scheduled_for, next_run_at)
                    .await
                {
                    tracing::error!("Scheduled task {} failed: {}", task.task_key, err);
                }
            }
        });
    }

    fn cron_from_expression(expression: &str) -> Result<Cron, ServiceError> {
        Cron::from_str(expression).map_err(|err| {
            ServiceError::InvalidOperation(format!("Invalid cron expression: {err}"))
        })
    }

    fn next_run_at_for_cron(
        cron: &Cron,
        timezone: FixedOffset,
    ) -> Result<DateTime<Utc>, ServiceError> {
        let now = Utc::now().with_timezone(&timezone);
        let next_run_at = cron.find_next_occurrence(&now, false).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to calculate next task run time: {err}"))
        })?;
        Ok(next_run_at.with_timezone(&Utc))
    }

    pub async fn list_tasks(&self) -> Result<Vec<TaskItem>, ServiceError> {
        self.repo.list_tasks().await
    }

    pub async fn list_task_runs(
        &self,
        task_key: &str,
        query: TaskRunQuery,
    ) -> Result<(Vec<TaskRunItem>, i64), ServiceError> {
        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        self.repo.list_task_runs(task_key, pagination.offset.into(), pagination.limit.into()).await
    }

    pub async fn run_task(&self, task_key: &str) -> Result<TaskRunItem, ServiceError> {
        self.start_task_by_key(task_key, TaskTriggerType::Manual, None).await
    }

    async fn start_scheduled_task(
        &self,
        task_key: &str,
        scheduled_for: DateTime<Utc>,
        next_run_at: Option<DateTime<Utc>>,
    ) -> Result<TaskRunItem, ServiceError> {
        self.repo.update_task_next_run_at(task_key, next_run_at).await?;
        self.start_task_by_key(task_key, TaskTriggerType::Scheduled, Some(scheduled_for)).await
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
            .ok_or_else(|| {
                ServiceError::InvalidOperation("Task scheduler is not initialized".to_string())
            })?
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
                    return Err(ServiceError::TaskAlreadyRunning);
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
        let timeout_duration = task_run_timeout_duration(CONFIG.task_run_timeout_seconds());

        tokio::spawn(async move {
            let ctx = TaskExecutionContext {
                task_key: task_key.clone(),
                task_name,
                trigger_type,
                scheduled_for,
            };
            let result = match timeout(timeout_duration, executor.execute(ctx)).await {
                Ok(result) => result,
                Err(_) => Err(ServiceError::InvalidOperation(format!(
                    "Task exceeded timeout of {} seconds",
                    timeout_duration.as_secs()
                ))),
            };
            let finished_at = Utc::now();
            let (status, message) = match result {
                Ok(()) => (TaskRunStatus::Success, None),
                Err(err) => (TaskRunStatus::Failed, Some(err.to_string())),
            };

            if let Err(err) = repo
                .finish_task_run(FinishTaskRunInput {
                    run_id,
                    task_key: &task_key,
                    trigger_type,
                    status,
                    started_at,
                    finished_at,
                    error_message: message.as_deref(),
                })
                .await
            {
                tracing::error!("Failed to finish task run {}: {}", run_id, err);
            }

            drop(guard);
        });

        Ok(run)
    }

    fn next_run_after_for_cron(
        cron: &Cron,
        timezone: FixedOffset,
        after: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, ServiceError> {
        let after = after.with_timezone(&timezone);
        let next_run_at = cron.find_next_occurrence(&after, false).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to calculate next task run time: {err}"))
        })?;
        Ok(next_run_at.with_timezone(&Utc))
    }
}

#[cfg(test)]
mod service_tests;
