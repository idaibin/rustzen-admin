use super::types::{StatsResp, SystemMetricsDataResp, TrendResp, UserTrendsResp};
use crate::common::error::ServiceError;
use sqlx::SqlitePool;

pub struct DashboardRepository;

impl DashboardRepository {
    pub async fn get_stats(pool: &SqlitePool) -> Result<StatsResp, ServiceError> {
        // 并行执行所有查询
        let (
            total_users,
            active_users,
            today_logins,
            system_uptime,
            pending_users,
        ) = tokio::join!(
            // 获取总用户数
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
                .fetch_one(pool),

            // 获取活跃用户数（7天内登录）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE last_login_at > datetime('now', '-7 day') AND deleted_at IS NULL"
            )
            .fetch_one(pool),

            // 获取今日登录数
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE last_login_at > datetime('now', '-1 day') AND deleted_at IS NULL"
            )
            .fetch_one(pool),

            // 计算系统运行时间
            // 计算系统运行时间（近似）：基于数据库中第一条日志时间
            sqlx::query_scalar::<_, String>(
                r#"
                SELECT
                    ((CAST(strftime('%s', 'now') AS INTEGER) - CAST(COALESCE(
                        (SELECT CAST(strftime('%s', created_at) AS INTEGER) FROM operation_logs ORDER BY created_at ASC LIMIT 1),
                        strftime('%s', 'now')
                    ) AS INTEGER)) / 86400) || '天 ' ||
                    (((CAST(strftime('%s', 'now') AS INTEGER) - CAST(COALESCE(
                        (SELECT CAST(strftime('%s', created_at) AS INTEGER) FROM operation_logs ORDER BY created_at ASC LIMIT 1),
                        strftime('%s', 'now')
                    ) AS INTEGER)) % 86400) / 3600) || '小时 '
                "#
            )
            .fetch_one(pool),

            // 获取待审核用户数
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE status = 3 AND deleted_at IS NULL"
            )
            .fetch_one(pool)
        );

        // 处理查询结果
        let total_users = total_users.map_err(|e| {
            tracing::error!("Database error getting total users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let active_users = active_users.map_err(|e| {
            tracing::error!("Database error getting active users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let today_logins = today_logins.map_err(|e| {
            tracing::error!("Database error getting today logins: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let system_uptime = system_uptime.map_err(|e| {
            tracing::error!("Database error getting system uptime: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let pending_users = pending_users.map_err(|e| {
            tracing::error!("Database error getting pending users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let stats =
            StatsResp { total_users, active_users, today_logins, system_uptime, pending_users };
        Ok(stats)
    }

    pub async fn get_metrics(pool: &SqlitePool) -> Result<SystemMetricsDataResp, ServiceError> {
        // 并行获取系统指标
        let (
            total_requests,
            error_requests,
            avg_response_time,
        ) = tokio::join!(
            // 获取总请求数（从日志表统计）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM operation_logs WHERE created_at > datetime('now', '-7 day')"
            )
            .fetch_one(pool),

            // 获取错误请求数（状态为 FAILED 或 ERROR）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM operation_logs WHERE status IN ('FAILED', 'ERROR') AND created_at > datetime('now', '-7 day')"
            )
            .fetch_one(pool),

            // 获取平均响应时间（毫秒）
            sqlx::query_scalar::<_, f64>(
                "SELECT COALESCE(AVG(CAST(duration_ms AS REAL)), 0) FROM operation_logs WHERE created_at > datetime('now', '-7 day') AND duration_ms IS NOT NULL"
            )
            .fetch_one(pool)
        );

        // 处理查询结果
        let total_requests = total_requests.map_err(|e| {
            tracing::error!("Database error getting total requests: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let error_requests = error_requests.map_err(|e| {
            tracing::error!("Database error getting error requests: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let avg_response_time = avg_response_time.map_err(|e| {
            tracing::error!("Database error getting avg response time: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // 计算错误率
        let error_rate = if total_requests > 0 {
            (error_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        // 计算平均响应时间（毫秒）
        let avg_response_time_ms = avg_response_time as i64;

        let metrics = SystemMetricsDataResp {
            avg_response_time: avg_response_time_ms,
            error_rate,
            total_requests,
        };

        Ok(metrics)
    }

    pub async fn get_trends(pool: &SqlitePool) -> Result<UserTrendsResp, ServiceError> {
        // 并行获取趋势数据
        let (daily_logins, hourly_active) = tokio::join!(
            // 获取最近30天的登录趋势
            Self::get_daily_login_trends(pool),
            // 获取24小时活跃用户分布
            Self::get_hourly_active_users(pool)
        );

        let daily_logins = daily_logins?;
        let hourly_active = hourly_active?;

        Ok(UserTrendsResp { daily_logins, hourly_active })
    }

    /// 获取每日登录趋势（最近30天）
    async fn get_daily_login_trends(pool: &SqlitePool) -> Result<Vec<TrendResp>, ServiceError> {
        let daily_logins = sqlx::query_as(
            r#"
            SELECT
                strftime('%Y-%m-%d', created_at) as date,
                COUNT(*) as count
            FROM operation_logs
            WHERE action = 'AUTH_LOGIN'
                AND status = 'SUCCESS'
                AND created_at > datetime('now', '-30 day')
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error getting daily login trends: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(daily_logins)
    }

    /// 获取24小时活跃用户分布
    async fn get_hourly_active_users(pool: &SqlitePool) -> Result<Vec<TrendResp>, ServiceError> {
        let hourly_active: Vec<TrendResp> = sqlx::query_as(
            r#"
            WITH RECURSIVE hour_series AS (
                SELECT 0 as hour
                UNION ALL
                SELECT hour + 1 FROM hour_series WHERE hour < 23
            )
            SELECT
                CAST(hs.hour AS TEXT) as date,
                COALESCE(COUNT(DISTINCT ol.user_id), 0) as count
            FROM hour_series hs
            LEFT JOIN operation_logs ol ON CAST(strftime('%H', ol.created_at) AS INTEGER) = hs.hour
                AND ol.created_at > datetime('now', '-24 hour')
                AND ol.user_id IS NOT NULL
            GROUP BY hs.hour
            ORDER BY hs.hour
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error getting hourly active users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(hourly_active)
    }
}
