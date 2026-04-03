use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResp {
    pub total_users: i64,
    pub active_users: i64,
    pub today_logins: i64,
    pub system_uptime: String,
    pub pending_users: i64,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetricsDataResp {
    pub avg_response_time: i64,
    pub error_rate: f64,
    pub total_requests: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TrendResp {
    pub date: Option<String>,
    pub count: Option<i64>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrendsResp {
    pub daily_logins: Vec<TrendResp>,
    pub hourly_active: Vec<TrendResp>,
}
