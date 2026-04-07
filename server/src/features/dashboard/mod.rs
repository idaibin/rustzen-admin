pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{Router, routing::get};
use rustzen_core::permission::{PermissionsCheck, RouterExt};
use sqlx::PgPool;

use handler::{get_health, get_metrics, get_stats, get_trends};

pub fn dashboard_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/stats",
            get(get_stats),
            PermissionsCheck::Require("dashboard:view"),
        )
        .route_with_permission(
            "/health",
            get(get_health),
            PermissionsCheck::Require("dashboard:view"),
        )
        .route_with_permission(
            "/metrics",
            get(get_metrics),
            PermissionsCheck::Require("dashboard:view"),
        )
        .route_with_permission(
            "/trends",
            get(get_trends),
            PermissionsCheck::Require("dashboard:view"),
        )
}
