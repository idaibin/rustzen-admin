use super::{
    service::UserService,
    types::{
        CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload, UpdateUserStatusPayload,
        UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rustzen_auth::auth::{AuthClaims, CurrentUser};
use sqlx::SqlitePool;
use tracing::instrument;

/// Get user list
#[instrument(skip(pool, query))]
pub async fn list_users(
    State(pool): State<SqlitePool>,
    Query(query): Query<UserQuery>,
) -> AppResult<Vec<UserItemResp>> {
    let (users, total) = UserService::list_users(&pool, query).await?;
    Ok(ApiResponse::page(users, total))
}

/// Create user
#[instrument(skip(pool, dto))]
pub async fn create_user(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Json(dto): Json<CreateUserRequest>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(
        UserService::create_user(&pool, current_user.user_id, dto, &claims).await?,
    ))
}

/// Update user
#[instrument(skip(pool, id, dto))]
pub async fn update_user(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPayload>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(
        UserService::update_user(&pool, id, current_user.user_id, dto, &claims).await?,
    ))
}

/// Delete user
#[instrument(skip(pool, id, current_user))]
pub async fn delete_user(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> AppResult<()> {
    UserService::delete_user(&pool, id, current_user.user_id, &claims).await?;
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
    State(pool): State<SqlitePool>,
    Query(query): Query<UserOptionsQuery>,
) -> AppResult<Vec<UserOptionResp>> {
    Ok(ApiResponse::success(UserService::get_user_options(&pool, query).await?))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_password(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserPasswordPayload>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(
        UserService::update_user_password(&pool, id, current_user.user_id, dto, &claims).await?,
    ))
}

#[instrument(skip(pool, id, dto))]
pub async fn update_user_status(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserStatusPayload>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(
        UserService::update_user_status(&pool, id, current_user.user_id, dto, &claims).await?,
    ))
}

#[instrument(skip(pool, id, current_user))]
pub async fn revoke_user_sessions(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(
        UserService::revoke_user_sessions(&pool, id, current_user.user_id, &claims).await?,
    ))
}
