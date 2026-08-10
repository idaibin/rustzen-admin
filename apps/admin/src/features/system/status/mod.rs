pub mod handler;
pub mod logs;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::get;
use handler::get_status_overview;
use logs::module_log_routes;
use rustzen_auth::capability::system_status;
use sqlx::SqlitePool;

pub fn status_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::GetStatusOverview,
            AccessPolicy::Require(system_status::VIEW),
            get(get_status_overview),
        )
        .expect("static status contract")
        .nest("/module-logs", module_log_routes())
        .expect("static status contract")
}
