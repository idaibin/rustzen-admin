use crate::features::system::{menu::model::MenuResponse, user::model::RoleInfo};
use serde::{Deserialize, Serialize};

/// Request body for user registration.
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Request body for user login.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Basic user information included in the registration response.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

/// Response body for a successful registration.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub user: UserInfo,
    pub token: String,
}

/// Response body for a successful login.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
}

/// Detailed user information for the "get user info" endpoint.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    /// List of roles assigned to the user.
    pub roles: Vec<RoleInfo>,
    /// The menu tree accessible to the user.
    pub menus: Vec<MenuResponse>,
}
