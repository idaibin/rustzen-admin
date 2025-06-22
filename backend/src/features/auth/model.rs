use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub real_name: Option<String>,
}

/// 注册响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub message: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub user_info: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub roles: Vec<String>,
}

/// 用户详细信息响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user: UserDetail,
    pub menus: Vec<crate::features::system::menu::model::MenuResponse>,
}

/// 用户详细信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetail {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub last_login_at: Option<DateTime<Utc>>,
}
