use super::{
    extractor::CurrentUser,
    model::{LoginRequest, LoginResponse, UserInfoVo},
    permission::PermissionService,
    service::AuthService,
};
use crate::{
    common::api::{ApiResponse, AppResult},
    features::system::log::service::LogService,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    routing::{get, post},
};
use sqlx::PgPool;
use std::{net::SocketAddr, time::Instant};

/// Public auth routes (no token required)
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/login", post(login_handler))
}

/// Protected auth routes (JWT required)
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new().route("/me", get(get_login_info_handler)).route("/logout", get(logout_handler))
}

/// Login with username/password
#[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
async fn login_handler(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResponse> {
    let start_time = Instant::now();
    tracing::info!("Login attempt from {}", addr.ip());

    let username = request.username.clone();
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown");

    match AuthService::login(&pool, request).await {
        Ok(response) => {
            // 登录成功日志
            if let Err(e) = LogService::log_business_operation(
                &pool,
                response.user_id,
                &username,
                "AUTH_LOGIN",
                "User login successful",
                serde_json::json!({}),
                "SUCCESS",
                start_time.elapsed().as_millis() as i32,
                &ip_address,
                &user_agent,
            )
            .await
            {
                tracing::error!("Failed to log login operation: {:?}", e);
            }
            tracing::info!("Login successful");
            Ok(ApiResponse::success(response))
        }
        Err(err) => {
            // 登录失败日志
            let user_id = 0_i64; // 或 -1，或 None，看你的日志表设计
            if let Err(e) = LogService::log_business_operation(
                &pool,
                user_id,
                &username,
                "AUTH_LOGIN",
                &err.to_string(),
                serde_json::json!({}),
                "FAIL",
                start_time.elapsed().as_millis() as i32,
                &ip_address,
                &user_agent,
            )
            .await
            {
                tracing::error!("Failed to log failed login operation: {:?}", e);
            }
            tracing::error!("Login failed for user: {}", username);
            // 关键：抛出原始错误
            Err(err.into())
        }
    }
}

/// Get current user info with roles and menus
#[tracing::instrument(name = "get_login_info", skip(current_user, pool))]
async fn get_login_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<UserInfoVo> {
    tracing::debug!("Get me info");

    let user_info = AuthService::get_login_info(&pool, current_user.user_id).await?;

    tracing::debug!("Me info retrieved: {:?}", user_info);
    Ok(ApiResponse::success(user_info))
}

/// Logout and clear cache
#[tracing::instrument(name = "logout", skip(current_user))]
async fn logout_handler(current_user: CurrentUser) -> AppResult<()> {
    tracing::info!("Logout");

    // Clear user permission cache
    PermissionService::clear_user_cache(current_user.user_id);

    tracing::info!("Logout completed");
    Ok(ApiResponse::success(()))
}
