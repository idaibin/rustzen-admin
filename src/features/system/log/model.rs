use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use std::net::IpAddr;

/// Log entity from database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LogEntity {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub action: String,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub status: String,
    pub duration_ms: i32,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub created_at: NaiveDateTime,
}

/// Log response for API
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogListVo {
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

/// Log query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQueryDto {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
}

impl From<LogEntity> for LogListVo {
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
