use serde::Serialize;

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsResp {
    pub total_users: i64,
    pub active_users: i64,
    pub today_logins: i64,
    pub pending_users: i64,
}
