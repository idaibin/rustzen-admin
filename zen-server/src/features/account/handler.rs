use super::{
    service::AccountService,
    types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        files::save_avatar,
    },
    features::auth::types::UserInfoResp,
};

use axum::{
    Json,
    extract::{Multipart, State},
};
use rustzen_core::auth::CurrentUser;
use sqlx::PgPool;

/// Update current-account avatar.
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
pub async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    let avatar_url = save_avatar(&mut multipart).await?;

    AccountService::update_avatar(&pool, current_user.user_id, &avatar_url).await?;

    Ok(ApiResponse::success(avatar_url))
}

/// Update current-account profile.
#[tracing::instrument(name = "update_profile", skip(current_user, pool, request))]
pub async fn update_profile(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
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
    State(pool): State<PgPool>,
    Json(request): Json<ChangeAccountPasswordRequest>,
) -> AppResult<()> {
    AccountService::change_password(&pool, current_user.user_id, request).await?;
    Ok(ApiResponse::success(()))
}
