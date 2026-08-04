pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{get, post};
use handler::{list_task_runs, list_tasks, run_task};
use rustzen_auth::capability::manage_task;
use sqlx::SqlitePool;

pub fn task_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListManageTasks,
            AccessPolicy::Require(manage_task::LIST),
            get(list_tasks),
        )
        .expect("static task contract")
        .get(
            "/{task_key}/runs",
            OperationDescriptor::ListTaskRuns,
            AccessPolicy::Require(manage_task::LIST),
            get(list_task_runs),
        )
        .expect("static task contract")
        .post(
            "/{task_key}/run",
            OperationDescriptor::RunTask,
            AccessPolicy::Require(manage_task::RUN),
            post(run_task),
        )
        .expect("static task contract")
}
