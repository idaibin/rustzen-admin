use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity (single table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntity {
    pub id: i64,
    pub username: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
}

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
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: serde_json::Value,
}

// impl From<UserEntity> for UserWithRolesEntity {
//     fn from(user: UserEntity) -> Self {
//         Self {
//             id: user.id,
//             username: user.username,
//             email: user.email,
//             password_hash: user.password_hash,
//             real_name: user.real_name,
//             avatar_url: user.avatar_url,
//             status: user.status,
//             last_login_at: user
//                 .last_login_at
//                 .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
//             created_at: DateTime::from_naive_utc_and_offset(user.created_at, Utc),
//             updated_at: DateTime::from_naive_utc_and_offset(user.updated_at, Utc),
//             roles: serde_json::Value::Null,
//         }
//     }
// }
