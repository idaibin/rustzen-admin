use crate::features::system::{menu::model::MenuResponse, user::model::RoleInfo};
use serde::{Deserialize, Serialize};

/// Request payload for user registration.
///
/// This struct represents the data required to create a new user account
/// through the registration endpoint. All fields are mandatory for account creation.
#[derive(Deserialize)]
pub struct RegisterRequest {
    /// Unique username for the new account (must be unique across the system)
    pub username: String,
    /// Email address for the new account (must be unique and valid format)
    pub email: String,
    /// Plain text password (will be hashed before storage)
    pub password: String,
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

/// Response payload for successful user registration.
///
/// This struct is returned when a user successfully creates a new account.
/// It includes basic user information and an authentication token for
/// immediate login after registration.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    /// Basic information about the newly created user
    pub user: UserInfo,
    /// JWT token for immediate authentication after registration
    pub token: String,
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
    /// Detailed user information including roles and menu access
    pub user_info: UserInfoResponse,
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
    /// List of roles assigned to the user with their details
    pub roles: Vec<RoleInfo>,
    /// Hierarchical menu structure accessible to the user based on their roles
    pub menus: Vec<MenuResponse>,
}
