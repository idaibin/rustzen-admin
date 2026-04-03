pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::{common::router_ext::RouterExt, infra::permission::PermissionsCheck};
use axum::{Router, routing::get};
use handler::{export_logs, list_logs};
use sqlx::PgPool;

pub fn log_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission("/", get(list_logs), PermissionsCheck::Require("system:log:list"))
        .route_with_permission(
            "/export",
            get(export_logs),
            PermissionsCheck::Require("system:log:export"),
        )
}
