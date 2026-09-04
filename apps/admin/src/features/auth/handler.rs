#[cfg(feature = "full")]
use super::types::LoginAuditCommand;
use super::{
    service::AuthService,
    types::{LoginRequest, LoginResp, UserInfoResp},
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use rustzen_auth::auth::CurrentUser;
use sqlx::SqlitePool;
use std::net::SocketAddr;

/// Login with username/password
#[tracing::instrument(name = "login", skip(pool, _addr, _headers, request))]
pub async fn login(
    State(pool): State<SqlitePool>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResp> {
    let LoginRequest { username, password } = request;
    #[cfg(feature = "full")]
    let audit_command = LoginAuditCommand {
        ip_address: _addr.ip().to_string(),
        user_agent: _headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string(),
    };

    #[cfg(feature = "full")]
    let response =
        AuthService::login_with_audit(&pool, &username, &password, audit_command).await?;
    #[cfg(feature = "monitor-distribution")]
    let response = AuthService::login(&pool, &username, &password).await?;
    Ok(ApiResponse::success(response))
}

/// Get current user info with roles and menus
#[tracing::instrument(name = "get_login_info", skip(current_user, pool))]
pub async fn get_login_info(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
) -> AppResult<UserInfoResp> {
    Ok(ApiResponse::success(AuthService::get_login_info(&pool, current_user.user_id).await?))
}

/// Logout and clear cache
#[tracing::instrument(name = "logout", skip(current_user))]
pub async fn logout(current_user: CurrentUser) -> AppResult<()> {
    AuthService::logout(current_user.user_id);
    Ok(ApiResponse::success(()))
}
