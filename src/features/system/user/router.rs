use super::{
    dto::{
        CreateUserDto, UpdateUserDto, UpdateUserPasswordDto, UpdateUserStatusDto, UserOptionsDto,
        UserQueryDto,
    },
    service::UserService,
    vo::{UserItemVo, UserOptionVo},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        error::ServiceError,
        router_ext::RouterExt,
    },
    core::permission::PermissionsCheck,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;
use tracing::instrument;

/// User management routes
pub fn user_routes() -> Router<sqlx::PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_user_list),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
        .route_with_permission(
            "/",
            post(create_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:create"]),
        )
        .route_with_permission(
            "/{id}",
            put(update_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:update"]),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:delete"]),
        )
        .route_with_permission(
            "/options",
            get(get_user_options),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
        .route_with_permission(
            "/status-options",
            get(get_user_status_options),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
        .route_with_permission(
            "/{id}/password",
            put(update_user_password),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:password"]),
        )
        .route_with_permission(
            "/{id}/status",
            put(update_user_status),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:status"]),
        )
}

/// Get user list
#[instrument(skip(pool, query))]
pub async fn get_user_list(
    State(pool): State<PgPool>,
    Query(query): Query<UserQueryDto>,
) -> AppResult<Vec<UserItemVo>> {
    tracing::info!("Getting user list");

    let (users, total) = UserService::get_user_list(&pool, query).await?;

    tracing::info!("Successfully retrieved {} users", users.len());
    Ok(ApiResponse::page(users, total))
}

/// Create user
#[instrument(skip(pool, dto))]
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(dto): Json<CreateUserDto>,
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
    Json(dto): Json<UpdateUserDto>,
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
pub async fn get_user_status_options() -> AppResult<Vec<UserOptionVo>> {
    tracing::info!("Getting user status options");

    let result = UserService::get_user_status_options();

    tracing::info!("Successfully retrieved {} status options", result.len());
    Ok(ApiResponse::success(result))
}

/// Get user options
#[instrument(skip(pool, query))]
pub async fn get_user_options(
    State(pool): State<PgPool>,
    Query(query): Query<UserOptionsDto>,
) -> AppResult<Vec<UserOptionVo>> {
    tracing::info!("Getting user options");

    let result = UserService::get_user_options(&pool, query).await?;

    tracing::info!("Successfully retrieved {} user options", result.len());
    Ok(ApiResponse::success(result))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_password(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPasswordDto>,
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
    Json(dto): Json<UpdateUserStatusDto>,
) -> AppResult<bool> {
    tracing::info!("Updating user status for user: {}", id);

    let result = UserService::update_user_status(&pool, id, dto).await?;

    tracing::info!("Successfully updated user status");
    Ok(ApiResponse::success(result))
}
