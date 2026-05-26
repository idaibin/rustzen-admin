use super::{
    service::AuthService,
    types::{LoginAuditCommand, LoginRequest, LoginResp, UserInfoResp},
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use rustzen_core::auth::CurrentUser;
use sqlx::SqlitePool;
use std::net::SocketAddr;

/// Login with username/password
#[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
pub async fn login(
    State(pool): State<SqlitePool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResp> {
    let LoginRequest { username, password } = request;
    let audit_command = LoginAuditCommand {
        ip_address: addr.ip().to_string(),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string(),
    };

    Ok(ApiResponse::success(
        AuthService::login_with_audit(&pool, &username, &password, audit_command).await?,
    ))
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
