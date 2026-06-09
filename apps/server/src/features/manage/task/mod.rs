pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use handler::{list_task_runs, list_tasks, run_task};
use rustzen_core::{
    capability::manage_task,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn task_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_tasks), PermissionsCheck::Require(manage_task::LIST))
        .route_with_permission(
            "/{task_key}/runs",
            get(list_task_runs),
            PermissionsCheck::Require(manage_task::LIST),
        )
        .route_with_permission(
            "/{task_key}/run",
            post(run_task),
            PermissionsCheck::Require(manage_task::RUN),
        )
}
