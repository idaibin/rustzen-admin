pub mod handler;
pub mod types;
pub mod repo;
pub mod service;

use axum::{Router, routing::get};
use sqlx::PgPool;

use handler::{get_health, get_metrics, get_stats, get_trends};

pub fn dashboard_routes() -> Router<PgPool> {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/health", get(get_health))
        .route("/metrics", get(get_metrics))
        .route("/trends", get(get_trends))
}
