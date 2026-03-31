use super::{
    service::AuthService,
    types::{LoginRequest, LoginResp, UserInfoResp},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        files::save_avatar,
    },
    features::system::log::{
        service::LogService,
        types::LogWriteCommand,
    },
    infra::{extractor::CurrentUser, permission::PermissionService},
};

use axum::{
    Json,
    extract::{ConnectInfo, Multipart, State},
    http::HeaderMap,
};
use sqlx::PgPool;
use std::{net::SocketAddr, time::Instant};

/// Login with username/password
#[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
pub async fn login_handler(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResp> {
    let start_time = Instant::now();
    tracing::info!("Login attempt from {}", addr.ip());

    let username = request.username.clone();
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown");

    match AuthService::login(&pool, request).await {
        Ok(response) => {
            if let Err(e) = LogService::record_operation(
                &pool,
                LogWriteCommand {
                    user_id: response.user_info.id,
                    username: username.clone(),
                    action: "AUTH_LOGIN".to_string(),
                    description: "User login successful".to_string(),
                    data: Some(serde_json::json!({})),
                    status: "SUCCESS".to_string(),
                    duration_ms: start_time.elapsed().as_millis() as i32,
                    ip_address: ip_address.clone(),
                    user_agent: user_agent.to_string(),
                },
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
            if let Err(e) = LogService::record_operation(
                &pool,
                LogWriteCommand {
                    user_id,
                    username: username.clone(),
                    action: "AUTH_LOGIN".to_string(),
                    description: err.to_string(),
                    data: Some(serde_json::json!({})),
                    status: "FAIL".to_string(),
                    duration_ms: start_time.elapsed().as_millis() as i32,
                    ip_address: ip_address.clone(),
                    user_agent: user_agent.to_string(),
                },
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
pub async fn get_login_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<UserInfoResp> {
    tracing::debug!("Get me info");

    let user_info = AuthService::get_login_info(&pool, current_user.user_id).await?;

    tracing::debug!("Me info retrieved: {:?}", user_info);
    Ok(ApiResponse::success(user_info))
}

/// Logout and clear cache
#[tracing::instrument(name = "logout", skip(current_user))]
pub async fn logout_handler(current_user: CurrentUser) -> AppResult<()> {
    tracing::info!("Logout");

    PermissionService::clear_user_cache(current_user.user_id);

    tracing::info!("Logout completed");
    Ok(ApiResponse::success(()))
}

/// Update user profile
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
pub async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    tracing::info!("Updating avatar for user: {}", current_user.user_id);

    let avatar_url = save_avatar(&mut multipart).await?;

    AuthService::update_avatar(&pool, current_user.user_id, &avatar_url).await?;

    Ok(ApiResponse::success(avatar_url))
}
