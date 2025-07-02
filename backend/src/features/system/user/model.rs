use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a user entity in the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserEntity {
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
}

/// Represents a user in API responses.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a user with roles in API responses.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub roles: Vec<RoleInfo>,
}

/// Represents the request body for creating a new user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub real_name: Option<String>,
    /// User status: Defaults to 1.
    pub status: Option<i16>,
    /// A list of role IDs to assign to the user. If empty, will use default role.
    #[serde(default)]
    pub role_ids: Vec<i64>,
}

/// Represents the request body for updating an existing user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub real_name: Option<String>,
    /// User status: 1 (正常) or 2 (禁用).
    pub status: Option<i16>,
    /// A list of role IDs to assign to the user. If provided, replaces all existing roles.
    pub role_ids: Option<Vec<i64>>,
}

/// Represents the query parameters for listing users.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQueryParams {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by username (case-insensitive search).
    pub username: Option<String>,
    /// Filter by user status. Accepts: "normal"/"1", "disabled"/"2", or "all".
    pub status: Option<String>,
}

/// Represents the response body for a list of users.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    /// The list of users for the current page.
    pub list: Vec<UserResponse>,
    /// The total number of users matching the query.
    pub total: i64,
    /// The current page number.
    pub page: i64,
    /// The number of items per page.
    pub page_size: i64,
}

/// Represents basic information about a role, used within the `UserResponse`.
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RoleInfo {
    pub id: i64,
    pub role_name: String,
}

impl From<UserEntity> for UserResponse {
    fn from(user: UserEntity) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: user.status,
            last_login_at: user
                .last_login_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            created_at: DateTime::from_naive_utc_and_offset(user.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(user.updated_at, Utc),
        }
    }
}

impl From<UserEntity> for UserDetailResponse {
    fn from(user: UserEntity) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: user.status,
            roles: vec![],
        }
    }
}
