use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsVo {
    pub total_users: i64,
    pub active_users: i64,
    pub today_logins: i64,
    pub system_uptime: String,
    pub pending_users: i64,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetricsDataVo {
    pub avg_response_time: i64,
    pub error_rate: f64,
    pub total_requests: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TrendVo {
    pub date: Option<String>,
    pub count: Option<i64>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrendsVo {
    pub daily_logins: Vec<TrendVo>,
    pub hourly_active: Vec<TrendVo>,
}
