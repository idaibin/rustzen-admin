use super::{
    extractor::CurrentUser,
    model::{LoginRequest, LoginResponse, UserInfoResponse},
    permission::PermissionService,
    service::AuthService,
};
use crate::{
    common::api::{ApiResponse, AppResult},
    core::password::PasswordUtils,
    features::system::log::service::LogService,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use std::net::SocketAddr;

/// Public auth routes (no token required)
pub fn public_auth_routes() -> Router<PgPool> {
    let mut router = Router::new().route("/login", post(login_handler));
    #[cfg(debug_assertions)]
    {
        // DEV-NOTE: This is a debug route for generating hash.
        // It is only available in debug mode.
        router = router.route("/gen-hash", get(gen_hash));
    }
    router
}

/// Protected auth routes (JWT required)
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new().route("/me", get(get_login_info_handler)).route("/logout", get(logout_handler))
}

/// Login with username/password
/// Body: username, password
async fn login_handler(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResponse> {
    tracing::info!("Login attempt from {}", addr.ip());

    let response = AuthService::login(&pool, request).await?;

    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown");

    let _ = LogService::log_business_operation(
        &pool,
        response.user_id,
        &response.username,
        "LOGIN",
        "AUTH",
        None,
        &ip_address,
        user_agent,
        true,
        Some("User login successful"),
    )
    .await;

    tracing::info!("Login successful");
    Ok(ApiResponse::success(response))
}

#[derive(Debug, Deserialize)]
struct HashRequest {
    password: String,
}

/// Generate hash with password
async fn gen_hash(Query(request): Query<HashRequest>) -> AppResult<String> {
    tracing::info!("Generate hash attempt");
    let password = PasswordUtils::hash_password(&request.password)?.to_string();
    tracing::info!("Hash generated: {}", password);
    Ok(ApiResponse::success(password))
}

/// Logout and clear cache
async fn logout_handler(current_user: CurrentUser) -> AppResult<()> {
    tracing::info!("Logout");

    // Clear user permission cache
    PermissionService::clear_user_cache(current_user.user_id);

    tracing::info!("Logout completed");
    Ok(ApiResponse::success(()))
}

/// Get current user info with roles and menus
async fn get_login_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<UserInfoResponse> {
    tracing::debug!("Get me info");

    let user_info = AuthService::get_login_info(&pool, current_user.user_id).await?;

    tracing::debug!("Me info retrieved");
    Ok(ApiResponse::success(user_info))
}
