use super::{
    dto::{CreateUserDto, UpdateUserDto, UserOptionsDto, UserQueryDto},
    service::UserService,
    vo::{UserDetailVo, UserOptionVo, UserStatusOptionVo},
};
use crate::features::auth::extractor::CurrentUser;
use crate::{
    common::api::{ApiResponse, AppResult},
    features::system::user::vo::UserListVo,
};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use sqlx::PgPool;
use tracing::instrument;

/// Get user list
#[instrument(skip(pool, query, current_user))]
pub async fn get_user_list(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Query(query): Query<UserQueryDto>,
) -> AppResult<Json<ApiResponse<Vec<UserListVo>>>> {
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
) -> AppResult<Json<ApiResponse<UserDetailVo>>> {
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
) -> AppResult<Json<ApiResponse<UserDetailVo>>> {
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
) -> AppResult<Json<ApiResponse<UserDetailVo>>> {
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
) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Deleting user ID: {} by user: {}", id, current_user.username);

    UserService::delete_user(&pool, id).await?;

    tracing::info!("Successfully deleted user ID: {}", id);
    Ok(ApiResponse::success(()))
}

/// Get user status options
pub async fn get_user_status_options(
    current_user: CurrentUser,
) -> AppResult<Json<ApiResponse<Vec<UserStatusOptionVo>>>> {
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
) -> AppResult<Json<ApiResponse<Vec<UserOptionVo>>>> {
    tracing::info!("Getting user options for user: {}", current_user.username);

    let result = UserService::get_user_options(&pool, query).await?;

    tracing::info!("Successfully retrieved {} user options", result.len());
    Ok(ApiResponse::success(result))
}

// /// Enable user
// pub async fn enable_user(
//     State(pool): State<PgPool>,
//     current_user: CurrentUser,
//     Path(id): Path<i64>,
// ) -> AppResult<Json<ApiResponse<()>>> {
//     tracing::info!("Enabling user ID: {} by user: {}", id, current_user.username);

//     UserService::enable_user(&pool, id).await?;

//     tracing::info!("Successfully enabled user ID: {}", id);
//     Ok(ApiResponse::success(()))
// }

// /// Disable user
// pub async fn disable_user(
//     State(pool): State<PgPool>,
//     current_user: CurrentUser,
//     Path(id): Path<i64>,
// ) -> AppResult<Json<ApiResponse<()>>> {
//     tracing::info!("Disabling user ID: {} by user: {}", id, current_user.username);

//     UserService::disable_user(&pool, id).await?;

//     tracing::info!("Successfully disabled user ID: {}", id);
//     Ok(ApiResponse::success(()))
// }

// /// Reset user password
// pub async fn reset_user_password(
//     State(pool): State<PgPool>,
//     current_user: CurrentUser,
//     Path(id): Path<i64>,
// ) -> AppResult<Json<ApiResponse<()>>> {
//     tracing::info!("Resetting password for user ID: {} by user: {}", id, current_user.username);

//     UserService::reset_user_password(&pool, id).await?;

//     tracing::info!("Successfully reset password for user ID: {}", id);
//     Ok(ApiResponse::success(()))
// }
