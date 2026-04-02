use super::{
    service::UserService,
    types::{
        CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload, UpdateUserStatusPayload,
        UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;
use tracing::instrument;

/// Get user list
#[instrument(skip(pool, query))]
pub async fn list_users(
    State(pool): State<PgPool>,
    Query(query): Query<UserQuery>,
) -> AppResult<Vec<UserItemResp>> {
    let (users, total) = UserService::list_users(&pool, query).await?;
    Ok(ApiResponse::page(users, total))
}

/// Create user
#[instrument(skip(pool, dto))]
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(dto): Json<CreateUserRequest>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(UserService::create_user(&pool, dto).await?))
}

/// Update user
#[instrument(skip(pool, id, dto))]
pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPayload>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(UserService::update_user(&pool, id, dto).await?))
}

/// Delete user
#[instrument(skip(pool, id))]
pub async fn delete_user(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    UserService::delete_user(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Get user status options
#[instrument]
pub async fn get_user_status_options() -> AppResult<Vec<UserOptionResp>> {
    Ok(ApiResponse::success(UserService::get_user_status_options()))
}

/// Get user options
#[instrument(skip(pool, query))]
pub async fn get_user_options(
    State(pool): State<PgPool>,
    Query(query): Query<UserOptionsQuery>,
) -> AppResult<Vec<UserOptionResp>> {
    Ok(ApiResponse::success(UserService::get_user_options(&pool, query).await?))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_password(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPasswordPayload>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(UserService::update_user_password(&pool, id, dto).await?))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserStatusPayload>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(UserService::update_user_status(&pool, id, dto).await?))
}
