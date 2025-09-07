use crate::common::error::ServiceError;

use super::{
    repo::DashboardRepository,
    vo::{StatsVo, SystemMetricsDataVo, UserTrendsVo},
};

use sqlx::PgPool;

pub struct DashboardService;

impl DashboardService {
    pub async fn get_stats(pool: &PgPool) -> Result<StatsVo, ServiceError> {
        let stats = DashboardRepository::get_stats(pool).await?;
        Ok(stats)
    }

    pub async fn get_metrics(pool: &PgPool) -> Result<SystemMetricsDataVo, ServiceError> {
        let metrics = DashboardRepository::get_metrics(pool).await?;
        Ok(metrics)
    }

    pub async fn get_trends(pool: &PgPool) -> Result<UserTrendsVo, ServiceError> {
        let operations = DashboardRepository::get_trends(pool).await?;
        Ok(operations)
    }
}
