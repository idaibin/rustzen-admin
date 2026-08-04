pub mod handler;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::get;
use rustzen_auth::capability::dashboard;
use sqlx::SqlitePool;

use handler::get_stats;

pub fn dashboard_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/stats",
            OperationDescriptor::GetDashboardStats,
            AccessPolicy::Require(dashboard::VIEW),
            get(get_stats),
        )
        .expect("static dashboard contract")
}
