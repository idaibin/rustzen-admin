use crate::common::api::{ApiResponse, AppResult};
use crate::common::router_ext::RouterExt;
use crate::features::auth::extractor::CurrentUser;
use crate::features::auth::permission::PermissionsCheck;
use crate::features::system::user::dto::{
    CreateUserDto, UpdateUserDto, UserOptionsDto, UserQueryDto,
};
use crate::features::system::user::service::UserService;
use crate::features::system::user::vo::{
    UserDetailVo, UserListVo, UserOptionVo, UserStatusOptionVo,
};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::{
    Router,
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
            get(get_user_by_id),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:detail"]),
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
}

/// Get user list
#[instrument(skip(pool, query, current_user))]
pub async fn get_user_list(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Query(query): Query<UserQueryDto>,
) -> AppResult<Vec<UserListVo>> {
    tracing::info!("Getting user list for user: {}", current_user.username);

    let (users, total) = UserService::get_user_list(&pool, query).await?;

    tracing::info!("Successfully retrieved {} users", users.len());
    Ok(ApiResponse::page(users, total))
}

/// Get user by ID
pub async fn get_user_by_id(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> AppResult<UserDetailVo> {
    tracing::info!("Getting user by ID: {} for user: {}", id, current_user.username);

    let result = UserService::get_user_by_id(&pool, id).await?;

    tracing::info!("Successfully retrieved user: {}", result.username);
    Ok(ApiResponse::success(result))
}

/// Create user
pub async fn create_user(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Json(dto): Json<CreateUserDto>,
) -> AppResult<UserDetailVo> {
    tracing::info!("Creating user: {} by user: {}", dto.username, current_user.username);

    let result = UserService::create_user(&pool, dto).await?;

    tracing::info!("Successfully created user: {}", result.username);
    Ok(ApiResponse::success(result))
}

/// Update user
pub async fn update_user(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserDto>,
) -> AppResult<UserDetailVo> {
    tracing::info!("Updating user ID: {} by user: {}", id, current_user.username);

    let result = UserService::update_user(&pool, id, dto).await?;

    tracing::info!("Successfully updated user: {}", result.username);
    Ok(ApiResponse::success(result))
}

/// Delete user
pub async fn delete_user(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> AppResult<()> {
    tracing::info!("Deleting user ID: {} by user: {}", id, current_user.username);

    UserService::delete_user(&pool, id).await?;

    tracing::info!("Successfully deleted user ID: {}", id);
    Ok(ApiResponse::success(()))
}

/// Get user status options
pub async fn get_user_status_options(
    current_user: CurrentUser,
) -> AppResult<Vec<UserStatusOptionVo>> {
    tracing::info!("Getting user status options for user: {}", current_user.username);

    let result = UserService::get_user_status_options();

    tracing::info!("Successfully retrieved {} status options", result.len());
    Ok(ApiResponse::success(result))
}

/// Get user options
pub async fn get_user_options(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Query(query): Query<UserOptionsDto>,
) -> AppResult<Vec<UserOptionVo>> {
    tracing::info!("Getting user options for user: {}", current_user.username);

    let result = UserService::get_user_options(&pool, query).await?;

    tracing::info!("Successfully retrieved {} user options", result.len());
    Ok(ApiResponse::success(result))
}
