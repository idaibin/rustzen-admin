use super::{
    service::AuthService,
    types::{LoginRequest, LoginResp, UserInfoResp},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        files::save_avatar,
    },
    features::system::log::{service::LogService, types::LogWriteCommand},
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
    let LoginRequest { username, password } = request;
    let ip_address = addr.ip().to_string();
    let user_agent =
        headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown").to_string();

    match AuthService::login(&pool, &username, &password).await {
        Ok(response) => {
            record_login_operation(
                &pool,
                response.user_info.id,
                &username,
                "SUCCESS",
                "User login successful",
                start_time,
                &ip_address,
                &user_agent,
            )
            .await;
            Ok(ApiResponse::success(response))
        }
        Err(err) => {
            record_login_operation(
                &pool,
                0,
                &username,
                "FAIL",
                &err.to_string(),
                start_time,
                &ip_address,
                &user_agent,
            )
            .await;
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
    Ok(ApiResponse::success(AuthService::get_login_info(&pool, current_user.user_id).await?))
}

/// Logout and clear cache
#[tracing::instrument(name = "logout", skip(current_user))]
pub async fn logout_handler(current_user: CurrentUser) -> AppResult<()> {
    PermissionService::clear_user_cache(current_user.user_id);
    Ok(ApiResponse::success(()))
}

/// Update user profile
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
pub async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    let avatar_url = save_avatar(&mut multipart).await?;

    AuthService::update_avatar(&pool, current_user.user_id, &avatar_url).await?;

    Ok(ApiResponse::success(avatar_url))
}

async fn record_login_operation(
    pool: &PgPool,
    user_id: i64,
    username: &str,
    status: &str,
    description: &str,
    start_time: Instant,
    ip_address: &str,
    user_agent: &str,
) {
    if let Err(e) = LogService::record_operation(
        pool,
        LogWriteCommand {
            user_id,
            username: username.to_string(),
            action: "AUTH_LOGIN".to_string(),
            description: description.to_string(),
            data: Some(serde_json::json!({})),
            status: status.to_string(),
            duration_ms: start_time.elapsed().as_millis() as i32,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
        },
    )
    .await
    {
        tracing::error!("Failed to log login operation: {:?}", e);
    }
}
