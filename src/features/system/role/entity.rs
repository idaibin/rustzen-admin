use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleWithMenuEntity {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_system: Option<bool>,
    pub menus: serde_json::Value,
}
