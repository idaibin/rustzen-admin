use super::{
    service::DashboardService,
    vo::{StatsVo, SystemMetricsDataVo, UserTrendsVo},
};
use crate::common::{
    api::{ApiResponse, AppResult},
    utils::system::{SystemInfo, SystemUtils},
};
use axum::{Router, extract::State, routing::get};

use sqlx::PgPool;
use tracing::instrument;

pub fn dashboard_routes() -> Router<PgPool> {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/health", get(get_health))
        .route("/metrics", get(get_metrics))
        .route("/trends", get(get_trends))
}

#[instrument(skip(pool,))]
pub async fn get_stats(State(pool): State<PgPool>) -> AppResult<StatsVo> {
    tracing::info!("Getting stats");
    let stats = DashboardService::get_stats(&pool).await?;
    Ok(ApiResponse::success(stats))
}

pub async fn get_health() -> AppResult<SystemInfo> {
    tracing::info!("Getting health");
    let system_info = SystemUtils::get_system_info();
    Ok(ApiResponse::success(system_info))
}

pub async fn get_metrics(State(pool): State<PgPool>) -> AppResult<SystemMetricsDataVo> {
    tracing::info!("Getting metrics");
    let metrics = DashboardService::get_metrics(&pool).await?;
    Ok(ApiResponse::success(metrics))
}

pub async fn get_trends(State(pool): State<PgPool>) -> AppResult<UserTrendsVo> {
    tracing::info!("Getting trends");
    let operations = DashboardService::get_trends(&pool).await?;
    Ok(ApiResponse::success(operations))
}
