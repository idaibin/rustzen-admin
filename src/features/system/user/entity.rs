use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// User with roles (for view-based queries)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserWithRolesEntity {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub roles: serde_json::Value,
}
