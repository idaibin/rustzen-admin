use crate::common::error::ServiceError;

use super::{
    repo::DashboardRepository,
    types::{StatsResp, SystemMetricsDataResp, UserTrendsResp},
};

use sqlx::PgPool;

pub struct DashboardService;

impl DashboardService {
    pub async fn get_stats(pool: &PgPool) -> Result<StatsResp, ServiceError> {
        DashboardRepository::get_stats(pool).await
    }

    pub async fn get_metrics(pool: &PgPool) -> Result<SystemMetricsDataResp, ServiceError> {
        DashboardRepository::get_metrics(pool).await
    }

    pub async fn get_trends(pool: &PgPool) -> Result<UserTrendsResp, ServiceError> {
        DashboardRepository::get_trends(pool).await
    }
}
