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
