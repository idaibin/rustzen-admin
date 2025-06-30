use super::{
    extractor::CurrentUser,
    model::{LoginRequest, LoginResponse, UserInfoResponse},
    permission::PermissionService,
    service::AuthService,
};
use crate::{
    common::api::{ApiResponse, AppResult},
    core::password::PasswordUtils,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;

/// Public auth routes (no token required)
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/login", post(login_handler)).route("/gen-hash", get(gen_hash))
}

/// Protected auth routes (JWT required)
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new().route("/me", get(get_me_info_handler)).route("/logout", get(logout_handler))
}

/// Login with username/password
/// Body: username, password
async fn login_handler(
    State(pool): State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    tracing::info!("Login attempt");

    let response = AuthService::login(&pool, request).await?;

    tracing::info!("Login successful");
    Ok(ApiResponse::success(response))
}

#[derive(Debug, Deserialize)]
struct HashRequest {
    password: String,
}

/// Generate hash with password
async fn gen_hash(Query(request): Query<HashRequest>) -> AppResult<Json<ApiResponse<String>>> {
    tracing::info!("Generate hash attempt");
    let password = PasswordUtils::hash_password(&request.password)?.to_string();
    tracing::info!("Hash generated: {}", password);
    Ok(ApiResponse::success(password))
}

/// Logout and clear cache
async fn logout_handler(current_user: CurrentUser) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Logout");

    // Clear user permission cache
    PermissionService::clear_user_cache(current_user.user_id);

    tracing::info!("Logout completed");
    Ok(ApiResponse::success(()))
}

/// Get current user info with roles and menus
async fn get_me_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<UserInfoResponse>>> {
    tracing::debug!("Get me info");

    let user_info = AuthService::get_me_info(&pool, current_user.user_id).await?;

    tracing::debug!("Me info retrieved");
    Ok(ApiResponse::success(user_info))
}
