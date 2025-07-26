use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;

use super::entity::LogEntity;

/// Log item for list display
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogItemVo {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub action: String,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub status: String,
    pub duration_ms: i32,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: NaiveDateTime,
}

impl From<LogEntity> for LogItemVo {
    fn from(entity: LogEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            username: entity.username,
            action: entity.action,
            description: entity.description,
            data: entity.data,
            ip_address: entity.ip_address.to_string(),
            user_agent: entity.user_agent,
            status: entity.status,
            duration_ms: entity.duration_ms,
            created_at: entity.created_at,
        }
    }
}
