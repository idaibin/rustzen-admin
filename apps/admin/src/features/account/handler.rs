use super::{
    service::AccountService,
    types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
};
use crate::{
    common::api::{ApiResponse, AppResult},
    features::auth::types::UserInfoResp,
};

#[cfg(feature = "full")]
use axum::extract::Multipart;
use axum::{Extension, Json, extract::State};
use rustzen_auth::auth::{AuthClaims, CurrentUser};
use sqlx::SqlitePool;

/// Update current-account avatar.
#[cfg(feature = "full")]
#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
pub async fn update_avatar(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    Ok(ApiResponse::success(
        AccountService::update_avatar(&pool, current_user.user_id, &mut multipart, &claims).await?,
    ))
}

/// Update current-account profile.
#[tracing::instrument(name = "update_profile", skip(current_user, pool, request))]
pub async fn update_profile(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Json(request): Json<UpdateAccountProfileRequest>,
) -> AppResult<UserInfoResp> {
    Ok(ApiResponse::success(
        AccountService::update_profile(&pool, current_user.user_id, request, &claims).await?,
    ))
}

/// Change current-account password.
#[tracing::instrument(name = "change_password", skip(current_user, pool, request))]
pub async fn change_password(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Json(request): Json<ChangeAccountPasswordRequest>,
) -> AppResult<()> {
    AccountService::change_password_authorized(&pool, current_user.user_id, request, &claims)
        .await?;
    Ok(ApiResponse::success(()))
}
