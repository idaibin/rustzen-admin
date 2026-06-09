pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{Router, routing::get};
use handler::{export_logs, list_logs};
use rustzen_core::{
    capability::manage_log,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn log_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_logs), PermissionsCheck::Require(manage_log::LIST))
        .route_with_permission(
            "/export",
            get(export_logs),
            PermissionsCheck::Require(manage_log::EXPORT),
        )
}
