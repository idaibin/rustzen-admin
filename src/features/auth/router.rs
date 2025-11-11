use super::{
    dto::LoginRequest,
    service::AuthService,
    vo::{LoginVo, UserInfoVo},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        files::save_avatar,
    },
    core::{extractor::CurrentUser, permission::PermissionService},
    features::system::log::service::LogService,
};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Multipart, State},
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
    Router::new()
        .route("/me", get(get_login_info_handler))
        .route("/logout", get(logout_handler))
        .route("/avatar", post(update_avatar))
}

/// Login with username/password
#[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
async fn login_handler(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginVo> {
    let start_time = Instant::now();
    tracing::info!("Login attempt from {}", addr.ip());

    let username = request.username.clone();
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown");

    match AuthService::login(&pool, request).await {
        Ok(response) => {
            if let Err(e) = LogService::log_business_operation(
                &pool,
                response.user_info.id,
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
            let user_id = 0_i64;
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

/// Update user profile
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    tracing::info!("Updating avatar for user: {}", current_user.user_id);

    let avatar_url = save_avatar(&mut multipart).await?;

    AuthService::update_avatar(&pool, current_user.user_id, &avatar_url).await?;

    Ok(ApiResponse::success(avatar_url))
}
