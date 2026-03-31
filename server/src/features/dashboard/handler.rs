use super::{
    service::DashboardService,
    types::{StatsResp, SystemMetricsDataResp, UserTrendsResp},
};
use crate::common::api::{ApiResponse, AppResult};
use crate::infra::system_info::{SystemInfo, SystemUtils};
use axum::extract::State;

use sqlx::PgPool;
use tracing::instrument;

#[instrument(skip(pool,))]
pub async fn get_stats(State(pool): State<PgPool>) -> AppResult<StatsResp> {
    tracing::info!("Getting stats");
    let stats = DashboardService::get_stats(&pool).await?;
    Ok(ApiResponse::success(stats))
}

pub async fn get_health() -> AppResult<SystemInfo> {
    tracing::info!("Getting health");
    let system_info = SystemUtils::get_system_info();
    Ok(ApiResponse::success(system_info))
}

pub async fn get_metrics(State(pool): State<PgPool>) -> AppResult<SystemMetricsDataResp> {
    tracing::info!("Getting metrics");
    let metrics = DashboardService::get_metrics(&pool).await?;
    Ok(ApiResponse::success(metrics))
}

pub async fn get_trends(State(pool): State<PgPool>) -> AppResult<UserTrendsResp> {
    tracing::info!("Getting trends");
    let operations = DashboardService::get_trends(&pool).await?;
    Ok(ApiResponse::success(operations))
}
