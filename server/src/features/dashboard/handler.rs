use super::{
    service::DashboardService,
    types::{StatsResp, SystemMetricsDataResp, UserTrendsResp},
};
use crate::common::api::{ApiResponse, AppResult};
use crate::infra::system_info::{SystemInfo, SystemUtils};
use axum::extract::State;

use sqlx::PgPool;
use tracing::instrument;

#[instrument(skip(pool))]
pub async fn get_stats(State(pool): State<PgPool>) -> AppResult<StatsResp> {
    Ok(ApiResponse::success(DashboardService::get_stats(&pool).await?))
}

pub async fn get_health() -> AppResult<SystemInfo> {
    Ok(ApiResponse::success(SystemUtils::get_system_info()))
}

pub async fn get_metrics(State(pool): State<PgPool>) -> AppResult<SystemMetricsDataResp> {
    Ok(ApiResponse::success(DashboardService::get_metrics(&pool).await?))
}

pub async fn get_trends(State(pool): State<PgPool>) -> AppResult<UserTrendsResp> {
    Ok(ApiResponse::success(DashboardService::get_trends(&pool).await?))
}
