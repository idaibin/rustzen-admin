use super::{
    extractor::CurrentUser,
    model::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfoResponse},
    permission::PermissionService,
    service::AuthService,
};
use crate::common::api::{ApiResponse, AppResult};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use sqlx::PgPool;

/// Public auth routes (no token required)
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/register", post(register_handler)).route("/login", post(login_handler))
}

/// Protected auth routes (JWT required)
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new().route("/me", get(get_user_info_handler)).route("/logout", get(logout_handler))
}

/// Register new user account
/// Body: username, email, password
async fn register_handler(
    State(pool): State<PgPool>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<RegisterResponse>>> {
    tracing::info!("Registration attempt: {} ({})", request.username, request.email);

    let response = AuthService::register(&pool, request).await?;

    tracing::info!(
        "Registration successful: id={}, username={}",
        response.user.id,
        response.user.username
    );

    Ok(ApiResponse::success(response))
}

/// Login with username/password
/// Body: username, password
async fn login_handler(
    State(pool): State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    tracing::info!("Login attempt: {}", request.username);

    let response = AuthService::login(&pool, request).await?;

    tracing::info!(
        "Login successful: id={}, username={}, roles={}",
        response.user_info.id,
        response.user_info.username,
        response.user_info.roles.len()
    );

    Ok(ApiResponse::success(response))
}

/// Logout and clear cache
async fn logout_handler(current_user: CurrentUser) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Logout: {} ({})", current_user.username, current_user.user_id);

    // Clear user permission cache
    PermissionService::clear_user_cache(current_user.user_id);

    tracing::info!(
        "Logout completed: id={}, username={}",
        current_user.user_id,
        current_user.username
    );

    Ok(ApiResponse::success(()))
}

/// Get current user info with roles and menus
async fn get_user_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<UserInfoResponse>>> {
    tracing::debug!("Get user info: {} ({})", current_user.username, current_user.user_id);

    let user_info =
        AuthService::get_user_info(&pool, current_user.user_id, &current_user.username).await?;

    tracing::debug!(
        "User info retrieved: id={}, roles={}, menus={}",
        user_info.id,
        user_info.roles.len(),
        user_info.menus.len()
    );

    Ok(ApiResponse::success(user_info))
}
