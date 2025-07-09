use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Query parameters for log list API
#[derive(Debug, Deserialize)]
pub struct LogListQuery {
    /// Search term for filtering log messages
    pub q: Option<String>,
    /// Page number (1-based)
    pub page: Option<i64>,
    /// Number of records per page
    pub page_size: Option<i64>,
}

/// Log entity from database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LogEntity {
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
    pub created_at: NaiveDateTime,
}

/// Log response for API
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResponse {
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

/// Request body for creating new log entries
#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    /// Log level (INFO, WARN, ERROR, DEBUG)
    pub level: String,
    /// Log message content
    pub message: String,
    /// Optional user ID associated with the log
    pub user_id: Option<i32>,
    /// Optional IP address
    pub ip_address: Option<String>,
}

/// Response for paginated log list
#[derive(Debug, Serialize)]
pub struct PaginatedLogsResponse {
    /// List of log entries
    pub items: Vec<LogResponse>,
    /// Total number of matching records
    pub total: i64,
    /// Current page number
    pub page: i64,
    /// Number of records per page
    pub page_size: i64,
}

/// Log query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQueryParams {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl From<LogEntity> for LogResponse {
    fn from(entity: LogEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            username: entity.username,
            action: entity.action,
            description: entity.description,
            ip_address: entity.ip_address,
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
