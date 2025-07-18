use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Role entity (single table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleEntity {
    pub id: i64,
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub is_system: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleWithMenuEntity {
    pub id: i64,
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub is_system: Option<bool>,
    pub menus: serde_json::Value,
}
