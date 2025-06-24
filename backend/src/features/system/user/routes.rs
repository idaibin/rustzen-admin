// src/features/user/routes.rs

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

use super::model::{
    CreateUserRequest, UpdateUserRequest, UserListResponse, UserQueryParams, UserResponse,
};
use super::service::UserService;
use crate::{
    common::api::{ApiResponse, AppResult, OptionItem, OptionsQuery},
    core::password::PasswordUtils,
};

/// Defines the routes for user management.
pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_user_list))
        .route("/", post(create_user))
        .route("/options", get(get_user_options))
        .route("/status-options", get(get_user_status_options))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
}

/// Handles the request to get a paginated list of users.
async fn get_user_list(
    State(pool): State<PgPool>,
    Query(params): Query<UserQueryParams>,
) -> AppResult<Json<ApiResponse<UserListResponse>>> {
    let user_list = UserService::get_user_list(&pool, params).await?;
    Ok(ApiResponse::success(user_list))
}

/// Handles the request to get a single user by their ID.
async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let user = UserService::get_user_by_id(&pool, id).await?;
    Ok(ApiResponse::success(user))
}

/// Handles the request to create a new user.
///
/// This endpoint supports admin user creation:
/// - Converts API request to internal CreateUserRequest
/// - Validates and assigns roles if provided
/// - For registration scenarios, use auth/register endpoint instead
async fn create_user(
    State(pool): State<PgPool>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    // 转换为统一的内部请求结构体
    let create_request = CreateUserRequest {
        username: request.username,
        email: request.email,
        password: PasswordUtils::hash_password(&request.password)?,
        real_name: request.real_name,
        status: request.status,
        role_ids: request.role_ids,
    };

    let new_user = UserService::create_user(&pool, &create_request).await?;
    Ok(ApiResponse::success(new_user))
}

/// Handles the request to update an existing user.
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let updated_user = UserService::update_user(&pool, id, request).await?;
    Ok(ApiResponse::success(updated_user))
}

/// Handles the request to delete a user.
async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    UserService::delete_user(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Handles the request to get user options for dropdowns
///
/// Extracts query parameters and delegates to the service layer for processing.
async fn get_user_options(
    State(pool): State<PgPool>,
    query: Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<OptionItem<i64>>>>> {
    let options = UserService::get_user_options(&pool, query).await?;
    Ok(ApiResponse::success(options))
}

/// Handles the request to get user status options for dropdowns
///
/// Returns all available user status options without requiring database access.
async fn get_user_status_options() -> AppResult<Json<ApiResponse<Vec<OptionItem<i16>>>>> {
    let options = vec![
        OptionItem { label: "正常".to_string(), value: 1 },
        OptionItem { label: "禁用".to_string(), value: 2 },
    ];
    Ok(ApiResponse::success(options))
}
