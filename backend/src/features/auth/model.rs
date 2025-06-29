use serde::{Deserialize, Serialize};

use crate::common::error::ServiceError;

/// User status enum for authentication and account control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Normal = 1,   // Active
    Disabled = 2, // Disabled
    Pending = 3,  // Pending approval
    Locked = 4,   // Locked
    Deleted = 5,  // Deleted
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
            5 => Ok(UserStatus::Deleted),
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
            UserStatus::Deleted => Err(ServiceError::InvalidUserStatus),
        }
    }
}
/// Request payload for user authentication.
///
/// This struct contains the credentials required for user login.
/// Both fields are mandatory for authentication.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username or email for authentication
    pub username: String,
    /// User's password in plain text
    pub password: String,
}

/// Basic user information included in authentication responses.
///
/// This struct contains minimal user data that is safe to include
/// in authentication responses. It's used in registration and
/// login responses where only basic identification is needed.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    /// Unique identifier of the user
    pub id: i64,
    /// Username of the user
    pub username: String,
}

/// Response payload for successful user login.
///
/// This struct is returned when a user successfully authenticates.
/// It includes both an authentication token and comprehensive user
/// information needed for the application session.
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

/// Comprehensive user information for authenticated sessions.
///
/// This struct contains detailed user information that is returned
/// after successful authentication or when requesting current user details.
/// It includes profile data, role assignments, and accessible menu structure.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    /// Unique identifier of the user
    pub id: i64,
    /// Username of the user
    pub username: String,
    /// Full/display name of the user (optional)
    pub real_name: Option<String>,
    /// URL to the user's avatar image (optional)
    pub avatar_url: Option<String>,
    /// Hierarchical menu structure accessible to the user based on their roles
    pub menus: Vec<AuthMenuInfoEntity>,
}

/// Minimal user info for authentication (login)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginCredentialsEntity {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub status: i16,
    pub is_super_admin: bool,
}

/// Basic user info for session/profile
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUserInfo {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Minimal menu information entity for frontend menu tree display.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuthMenuInfoEntity {
    /// Unique menu ID
    pub id: i64,
    /// Parent menu ID
    pub parent_id: Option<i64>,
    /// Menu title
    pub title: String,
    /// Route path
    pub path: String,
    /// Frontend component name
    pub component: Option<String>,
    /// Menu icon
    pub icon: Option<String>,
    /// Display order
    pub order_num: Option<i32>,
    /// Visibility flag
    pub visible: Option<bool>,
    /// Keep-alive flag
    pub keep_alive: Option<bool>,
    /// Menu type (e.g., directory, menu, button)
    pub menu_type: Option<i16>,
}
