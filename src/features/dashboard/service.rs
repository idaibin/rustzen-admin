use crate::common::error::ServiceError;

use super::{
    dto::{StatsResp, SystemMetricsDataResp, UserTrendsResp},
    repo::DashboardRepository,
};

use sqlx::PgPool;

pub struct DashboardService;

impl DashboardService {
    pub async fn get_stats(pool: &PgPool) -> Result<StatsResp, ServiceError> {
        let stats = DashboardRepository::get_stats(pool).await?;
        Ok(stats)
    }

    pub async fn get_metrics(pool: &PgPool) -> Result<SystemMetricsDataResp, ServiceError> {
        let metrics = DashboardRepository::get_metrics(pool).await?;
        Ok(metrics)
    }

    pub async fn get_trends(pool: &PgPool) -> Result<UserTrendsResp, ServiceError> {
        let operations = DashboardRepository::get_trends(pool).await?;
        Ok(operations)
    }
}
