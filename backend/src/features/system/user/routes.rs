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
use crate::common::api::ApiResponse;

/// 用户路由
pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_users))
        .route("/", post(create_user))
        .route("/{id}", get(get_user))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
}

/// 获取用户列表
async fn get_users(
    State(pool): State<PgPool>,
    Query(params): Query<UserQueryParams>,
) -> Json<ApiResponse<UserListResponse>> {
    match UserService::get_user_list(&pool, params).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取用户列表失败: {}", e);
            ApiResponse::fail(500, "获取用户列表失败".to_string())
        }
    }
}

/// 根据 ID 获取用户
async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<UserResponse>> {
    match UserService::get_user_by_id(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取用户失败: {}", e);
            ApiResponse::fail(500, "获取用户失败".to_string())
        }
    }
}

/// 创建用户
async fn create_user(
    State(pool): State<PgPool>,
    Json(request): Json<CreateUserRequest>,
) -> Json<ApiResponse<UserResponse>> {
    match UserService::create_user(&pool, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("创建用户失败: {}", e);
            ApiResponse::fail(500, "创建用户失败".to_string())
        }
    }
}

/// 更新用户
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateUserRequest>,
) -> Json<ApiResponse<UserResponse>> {
    match UserService::update_user(&pool, id, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("更新用户失败: {}", e);
            ApiResponse::fail(500, "更新用户失败".to_string())
        }
    }
}

/// 删除用户
async fn delete_user(State(pool): State<PgPool>, Path(id): Path<i64>) -> Json<ApiResponse<()>> {
    match UserService::delete_user(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("删除用户失败: {}", e);
            ApiResponse::fail(500, "删除用户失败".to_string())
        }
    }
}
