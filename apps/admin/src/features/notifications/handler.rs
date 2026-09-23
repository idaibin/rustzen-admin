use super::realtime::RealtimeHub;
use super::{
    service::NotificationService,
    types::{InboxListQuery, ReadAllRequest},
};
use crate::{
    common::api::{ApiResponse, AppResult},
    common::error::ServiceError,
    infra::config::CONFIG,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State, rejection::QueryRejection},
};
use rustzen_auth::auth::CurrentUser;
use sqlx::SqlitePool;

pub async fn list(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    query: Result<Query<InboxListQuery>, QueryRejection>,
) -> AppResult<super::types::InboxListResponse> {
    let Query(query) = query
        .map_err(|_| ServiceError::InvalidOperation("Invalid inbox query parameters".into()))?;
    Ok(ApiResponse::success(
        NotificationService::list(&pool, current_user.user_id, query, CONFIG.jwt_secret.as_bytes())
            .await?,
    ))
}

pub async fn unread_count(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
) -> AppResult<super::types::UnreadCountResponse> {
    Ok(ApiResponse::success(NotificationService::unread_count(&pool, current_user.user_id).await?))
}

pub async fn detail(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> AppResult<super::types::NotificationItem> {
    Ok(ApiResponse::success(NotificationService::detail(&pool, current_user.user_id, &id).await?))
}

pub async fn mark_read(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    realtime: Option<Extension<RealtimeHub>>,
) -> AppResult<super::types::ReadResponse> {
    let (response, changed) =
        NotificationService::mark_read_for_http(&pool, current_user.user_id, &id).await?;
    if changed && let Some(Extension(realtime)) = realtime {
        realtime.publish(current_user.user_id, response.revision);
    }
    Ok(ApiResponse::success(response))
}

pub async fn mark_all_read(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    realtime: Option<Extension<RealtimeHub>>,
    Json(request): Json<ReadAllRequest>,
) -> AppResult<super::types::ReadAllResponse> {
    let response = NotificationService::mark_all_read(
        &pool,
        current_user.user_id,
        request,
        CONFIG.jwt_secret.as_bytes(),
    )
    .await?;
    if response.changed > 0
        && let Some(Extension(realtime)) = realtime
    {
        realtime.publish(current_user.user_id, response.revision);
    }
    Ok(ApiResponse::success(response))
}
