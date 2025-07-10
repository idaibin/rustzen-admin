use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;

/// Log entity from database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LogEntity {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub description: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub status: String,
    pub duration_ms: Option<i32>,
    pub created_at: NaiveDateTime,
}

/// Log response for API
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogListVo {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub status: String,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Log query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQueryDto {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

impl From<LogEntity> for LogListVo {
    fn from(entity: LogEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            username: entity.username,
            action: entity.action,
            description: entity.description,
            ip_address: entity.ip_address.map(|ip| ip.to_string()),
            user_agent: entity.user_agent,
            request_id: entity.request_id,
            resource_type: entity.resource_type,
            resource_id: entity.resource_id,
            status: entity.status,
            duration_ms: entity.duration_ms,
            created_at: DateTime::from_naive_utc_and_offset(entity.created_at, Utc),
        }
    }
}
