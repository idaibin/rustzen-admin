use super::types::LoginAuditCommand;
use super::{
    service::AuthService,
    types::{LoginRequest, LoginResp, UserInfoResp},
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use rustzen_auth::auth::{AuthClaims, CurrentUser};
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
    let audit_command = LoginAuditCommand {
        ip_address: _addr.ip().to_string(),
        user_agent: _headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string(),
    };

    let response =
        AuthService::login_with_audit(&pool, &username, &password, audit_command).await?;
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
#[tracing::instrument(name = "logout", skip(current_user, claims, pool))]
pub async fn logout(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
) -> AppResult<()> {
    AuthService::logout(&pool, current_user.user_id, &claims.sid).await?;
    Ok(ApiResponse::success(()))
}
