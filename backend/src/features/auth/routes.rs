use super::{
    model::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfoResponse},
    service::AuthService,
};
use crate::{
    common::api::{ApiResponse, AppResult},
    core::jwt::Claims,
};
use axum::{
    Extension, Json, Router,
    extract::State,
    routing::{get, post},
};
use sqlx::PgPool;
use tracing::info;

/// Defines the public routes for authentication that do not require a token.
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/register", post(register_handler)).route("/login", post(login_handler))
}

/// Defines the protected routes for authentication that require a token.
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new().route("/me", get(get_user_info_handler))
}

/// Handles user registration requests.
async fn register_handler(
    axum::extract::State(pool): axum::extract::State<PgPool>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<RegisterResponse>>> {
    let response = AuthService::register(&pool, request).await?;
    Ok(ApiResponse::success(response))
}

/// Handles user login requests.
async fn login_handler(
    axum::extract::State(pool): axum::extract::State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    let response = AuthService::login(&pool, request).await?;
    Ok(ApiResponse::success(response))
}

/// Handles requests to get the current user's information.
/// This route is protected and requires a valid JWT.
async fn get_user_info_handler(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<ApiResponse<UserInfoResponse>>> {
    info!("claims: {:?}", claims);
    let response = AuthService::get_user_info(&pool, claims).await?;
    Ok(ApiResponse::success(response))
}
