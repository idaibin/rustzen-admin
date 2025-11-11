use serde::{Deserialize, Serialize};

/// Response payload for successful user login.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginVo {
    /// JWT token for authenticating subsequent requests
    pub token: String,
    /// User information
    pub user_info: UserInfoVo,
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
