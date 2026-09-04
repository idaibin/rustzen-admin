use super::{
    service::AccountService,
    types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
};
use crate::{
    common::api::{ApiResponse, AppResult},
    features::auth::types::UserInfoResp,
};

use axum::{
    Json,
    extract::{Multipart, State},
};
use rustzen_auth::auth::CurrentUser;
use sqlx::SqlitePool;

/// Update current-account avatar.
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
pub async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    Ok(ApiResponse::success(
        AccountService::update_avatar(&pool, current_user.user_id, &mut multipart).await?,
    ))
}

/// Update current-account profile.
#[tracing::instrument(name = "update_profile", skip(current_user, pool, request))]
pub async fn update_profile(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Json(request): Json<UpdateAccountProfileRequest>,
) -> AppResult<UserInfoResp> {
    Ok(ApiResponse::success(
        AccountService::update_profile(&pool, current_user.user_id, request).await?,
    ))
}

/// Change current-account password.
#[tracing::instrument(name = "change_password", skip(current_user, pool, request))]
pub async fn change_password(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Json(request): Json<ChangeAccountPasswordRequest>,
) -> AppResult<()> {
    AccountService::change_password(&pool, current_user.user_id, request).await?;
    Ok(ApiResponse::success(()))
}
