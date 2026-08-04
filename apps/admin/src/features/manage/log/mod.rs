pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::get;
use handler::{export_logs, list_logs};
use rustzen_auth::capability::manage_log;
use sqlx::SqlitePool;

pub fn log_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListManageLogs,
            AccessPolicy::Require(manage_log::LIST),
            get(list_logs),
        )
        .expect("static log contract")
        .get(
            "/export",
            OperationDescriptor::ExportManageLogs,
            AccessPolicy::Require(manage_log::EXPORT),
            get(export_logs),
        )
        .expect("static log contract")
}
