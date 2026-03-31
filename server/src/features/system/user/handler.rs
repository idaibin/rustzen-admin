use super::{
    service::UserService,
    types::{
        CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload, UpdateUserStatusPayload,
        UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        error::ServiceError,
    },
};

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
    tracing::info!("Getting user list");

    let (users, total) = UserService::list_users(&pool, query).await?;

    tracing::info!("Successfully retrieved {} users", users.len());
    Ok(ApiResponse::page(users, total))
}

/// Create user
#[instrument(skip(pool, dto))]
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(dto): Json<CreateUserRequest>,
) -> AppResult<i64> {
    tracing::info!("Creating user: {}", dto.username);

    let user_id = UserService::create_user(&pool, dto).await?;

    tracing::info!("Successfully created user");
    Ok(ApiResponse::success(user_id))
}

/// Update user
#[instrument(skip(pool, id, dto))]
pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPayload>,
) -> AppResult<i64> {
    tracing::info!("Updating user ID: {}", id);

    if id == 1 {
        return Err(ServiceError::UserIsAdmin.into());
    }

    let user_id = UserService::update_user(&pool, id, dto).await?;

    tracing::info!("Successfully updated user");
    Ok(ApiResponse::success(user_id))
}

/// Delete user
#[instrument(skip(pool, id))]
pub async fn delete_user(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Deleting user ID: {}", id);

    UserService::delete_user(&pool, id).await?;

    tracing::info!("Successfully deleted user ID: {}", id);
    Ok(ApiResponse::success(()))
}

/// Get user status options
#[instrument]
pub async fn get_user_status_options() -> AppResult<Vec<UserOptionResp>> {
    tracing::info!("Getting user status options");

    let result = UserService::get_user_status_options();

    tracing::info!("Successfully retrieved {} status options", result.len());
    Ok(ApiResponse::success(result))
}

/// Get user options
#[instrument(skip(pool, query))]
pub async fn get_user_options(
    State(pool): State<PgPool>,
    Query(query): Query<UserOptionsQuery>,
) -> AppResult<Vec<UserOptionResp>> {
    tracing::info!("Getting user options");

    let result = UserService::get_user_options(&pool, query).await?;

    tracing::info!("Successfully retrieved {} user options", result.len());
    Ok(ApiResponse::success(result))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_password(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPasswordPayload>,
) -> AppResult<bool> {
    tracing::info!("Updating user password for user: {}", id);

    let result = UserService::update_user_password(&pool, id, dto).await?;

    tracing::info!("Successfully updated user password");
    Ok(ApiResponse::success(result))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserStatusPayload>,
) -> AppResult<bool> {
    tracing::info!("Updating user status for user: {}", id);

    let result = UserService::update_user_status(&pool, id, dto).await?;

    tracing::info!("Successfully updated user status");
    Ok(ApiResponse::success(result))
}
