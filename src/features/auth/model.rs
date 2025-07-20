use serde::{Deserialize, Serialize};

use crate::common::error::ServiceError;

/// User status enum for authentication and account control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Normal = 1,   // Active
    Disabled = 2, // Disabled
    Pending = 3,  // Pending approval
    Locked = 4,   // Locked
}

impl TryFrom<i16> for UserStatus {
    type Error = ServiceError;

    /// Convert i16 to UserStatus, returns error if value is invalid.
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UserStatus::Normal),
            2 => Ok(UserStatus::Disabled),
            3 => Ok(UserStatus::Pending),
            4 => Ok(UserStatus::Locked),
            _ => Err(ServiceError::InvalidUserStatus),
        }
    }
}

impl UserStatus {
    /// Checks if the user status allows login.
    /// Returns Ok(()) if allowed, or an appropriate ServiceError otherwise.
    pub fn check_status(&self) -> Result<(), ServiceError> {
        match self {
            UserStatus::Normal => Ok(()),
            UserStatus::Disabled => Err(ServiceError::UserIsDisabled),
            UserStatus::Pending => Err(ServiceError::UserIsPending),
            UserStatus::Locked => Err(ServiceError::UserIsLocked),
        }
    }
}
/// Request payload for user authentication.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username or email for authentication
    pub username: String,
    /// User's password in plain text
    pub password: String,
}

/// Response payload for successful user login.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    /// JWT token for authenticating subsequent requests
    pub token: String,
    /// Username of the user
    pub username: String,
    /// Unique identifier of the user
    pub user_id: i64,
}

/// Minimal user info for authentication (login)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginCredentialsEntity {
    pub id: i64,
    pub password_hash: String,
    pub status: i16,
    pub is_system: bool,
}

/// Basic user info for session/profile
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUserEntity {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub is_system: bool,
}

/// Comprehensive user information for authenticated sessions.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoVo {
    /// Unique identifier of the user
    pub id: i64,
    /// Username of the user
    pub username: String,
    /// Full/display name of the user (optional)
    pub real_name: Option<String>,
    /// Email of the user
    pub email: Option<String>,
    /// Avatar URL of the user
    pub avatar_url: Option<String>,
    /// Whether the user is a system user
    pub is_system: bool,
    /// List of permission codes the user has access to
    pub permissions: Vec<String>,
}
