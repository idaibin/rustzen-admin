use std::{sync::Arc, time::Duration};

use chrono::Utc;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::{Mutex, Notify, RwLock};
use axum::{Extension, body::{Body, to_bytes}, http::{Request, StatusCode}};
use tower::ServiceExt;
use rustzen_auth::auth::CurrentUser;

use crate::common::error::ServiceError;

use super::super::schedule::{parse_fixed_timezone, task_run_timeout_duration};
use super::super::{
    catalog::{ScheduledTask, TaskCatalog},
    repo::TaskRepository,
    types::{TaskExecutionContext, TaskExecutor, TaskRunQuery, TaskRunStatus, TaskTriggerType},
};
use super::TaskService;

struct BlockingExecutor {
    started: Arc<Notify>,
    release: Arc<Notify>,
}

#[async_trait::async_trait]
impl TaskExecutor for BlockingExecutor {
    async fn execute(&self, _: TaskExecutionContext) -> Result<(), ServiceError> {
        self.started.notify_waiters();
        self.release.notified().await;
        Ok(())
    }
}

async fn pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    crate::infra::db::run_migrations(&pool).await.expect("migrate");
    pool
}

fn service_with_task(pool: sqlx::SqlitePool, executor: Arc<dyn TaskExecutor>) -> TaskService {
    let repo = Arc::new(TaskRepository::new(pool.clone()));
    TaskService {
        pool,
        repo,
        catalog: Arc::new(RwLock::new(Some(TaskCatalog {
            tasks: vec![ScheduledTask {
                task_key: "test-task",
                name: "Test task",
                description: "test",
                expression: "0 0 0 * * * *",
                executor,
                run_lock: Arc::new(Mutex::new(())),
            }],
        }))),
        timezone: parse_fixed_timezone("UTC").expect("timezone"),
    }
}

#[test]
fn parses_supported_fixed_timezones() {
    assert_eq!(parse_fixed_timezone("UTC").unwrap().local_minus_utc(), 0);
    assert_eq!(parse_fixed_timezone("Asia/Shanghai").unwrap().local_minus_utc(), 8 * 3600);
    assert_eq!(parse_fixed_timezone("+08:00").unwrap().local_minus_utc(), 8 * 3600);
    assert_eq!(parse_fixed_timezone("-05:30").unwrap().local_minus_utc(), -((5 * 3600) + 1800));
}

#[test]
fn rejects_named_timezone_database_entries() {
    assert!(
        parse_fixed_timezone("America/New_York")
            .unwrap_err()
            .to_string()
            .contains("Invalid RUSTZEN_TIMEZONE")
    );
}

#[test]
fn task_run_timeout_duration_has_one_second_floor() {
    assert_eq!(task_run_timeout_duration(0), Duration::from_secs(1));
    assert_eq!(task_run_timeout_duration(30), Duration::from_secs(30));
}

#[tokio::test]
async fn bootstrap_registers_exactly_the_three_builtin_tasks() {
    let pool = pool().await;
    let service = TaskService::new(pool).expect("service");
    service.bootstrap().await.expect("bootstrap");
    let tasks = service.list_tasks().await.expect("tasks");
    assert_eq!(tasks.len(), 3);
    assert_eq!(
        tasks.iter().map(|task| task.task_key.as_str()).collect::<Vec<_>>(),
        [
            "cleanup-operation-logs-retention",
            "cleanup-task-runs-retention",
            "sqlite-storage-maintenance"
        ]
    );
}

#[tokio::test]
async fn manual_overlap_is_rejected_and_scheduled_overlap_is_skipped() {
    let pool = pool().await;
    sqlx::query("INSERT INTO system_tasks (task_key,name,schedule_type,schedule_json) VALUES ('test-task','Test task','cron','0 0 0 * * * *')").execute(&pool).await.expect("task");
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let service = service_with_task(
        pool,
        Arc::new(BlockingExecutor { started: started.clone(), release: release.clone() }),
    );
    let first = service.run_task("test-task").await.expect("first run");
    started.notified().await;
    assert_eq!(first.status, TaskRunStatus::Running);
    assert!(
        matches!(service.run_task("test-task").await, Err(ServiceError::TaskAlreadyRunning))
    );
    let skipped = service
        .start_scheduled_task("test-task", Utc::now(), Some(Utc::now()))
        .await
        .expect("scheduled overlap");
    assert_eq!(skipped.status, TaskRunStatus::Skipped);
    assert_eq!(skipped.trigger_type, TaskTriggerType::Scheduled);
    release.notify_waiters();
}

#[tokio::test]
async fn scheduled_run_records_the_original_due_time() {
    let pool = pool().await;
    sqlx::query("INSERT INTO system_tasks (task_key,name,schedule_type,schedule_json) VALUES ('test-task','Test task','cron','0 0 0 * * * *')").execute(&pool).await.expect("task");
    let started = Arc::new(Notify::new()); let release = Arc::new(Notify::new());
    let service = service_with_task(pool, Arc::new(BlockingExecutor { started: started.clone(), release: release.clone() }));
    let due = Utc::now() - chrono::Duration::minutes(5);
    let next = Utc::now() + chrono::Duration::minutes(5);
    let run = service.start_scheduled_task("test-task", due, Some(next)).await.expect("scheduled run");
    assert_eq!(run.scheduled_for, Some(due));
    assert_eq!(service.list_tasks().await.expect("tasks")[0].next_run_at, Some(next));
    release.notify_waiters();
}

#[tokio::test]
async fn router_returns_conflict_for_a_second_manual_run_while_first_is_running() {
    let pool = pool().await;
    sqlx::query("INSERT INTO system_tasks (task_key,name,schedule_type,schedule_json) VALUES ('test-task','Test task','cron','0 0 0 * * * *')").execute(&pool).await.expect("task");
    let started = Arc::new(Notify::new()); let release = Arc::new(Notify::new());
    let service = Arc::new(service_with_task(pool.clone(), Arc::new(BlockingExecutor { started: started.clone(), release: release.clone() })));
    let (router, _) = super::super::task_routes().into_parts();
    let router = router.layer(Extension(service)).with_state(pool.clone());
    let request = || { let mut request = Request::post("/test-task/run").body(Body::empty()).expect("request"); request.extensions_mut().insert(CurrentUser::new(1, "owner", vec!["manage:task:run".into()], false)); request };
    let first = router.clone().oneshot(request()).await.expect("first response");
    assert_eq!(first.status(), StatusCode::OK);
    started.notified().await;
    let second = router.clone().oneshot(request()).await.expect("second response");
    assert_eq!(second.status(), StatusCode::CONFLICT);
    let body = to_bytes(second.into_body(), usize::MAX).await.expect("body");
    assert!(std::str::from_utf8(&body).expect("json").contains("10203"));
    let running: String = sqlx::query_scalar("SELECT status FROM system_task_runs ORDER BY id LIMIT 1").fetch_one(&pool).await.expect("running");
    assert_eq!(running, "running");
    release.notify_waiters();
    for _ in 0..50 { let status: String = sqlx::query_scalar("SELECT status FROM system_task_runs ORDER BY id LIMIT 1").fetch_one(&pool).await.expect("status"); if status == "success" { return; } tokio::time::sleep(Duration::from_millis(10)).await; }
    panic!("first run did not finish");
}

#[tokio::test]
async fn bootstrap_repairs_stale_running_records() {
    let pool = pool().await;
    let now = Utc::now();
    sqlx::query("INSERT INTO system_tasks (task_key,name,schedule_type,schedule_json,running,last_status,last_started_at) VALUES ('cleanup-operation-logs-retention','Old','cron','0 20 1 * * * *',1,'running',?)").bind(now).execute(&pool).await.expect("task");
    sqlx::query("INSERT INTO system_task_runs (task_key,trigger_type,status,started_at) VALUES ('cleanup-operation-logs-retention','manual','running',?)").bind(now).execute(&pool).await.expect("run");
    let service = TaskService::new(pool).expect("service");
    service.bootstrap().await.expect("bootstrap");
    let repaired = service
        .list_task_runs(
            "cleanup-operation-logs-retention",
            TaskRunQuery { current: Some(1), page_size: Some(10) },
        )
        .await
        .expect("records")
        .0;
    assert_eq!(repaired[0].status, TaskRunStatus::Failed);
    assert_eq!(
        repaired[0].error_message.as_deref(),
        Some("Task process stopped before completion")
    );
}
