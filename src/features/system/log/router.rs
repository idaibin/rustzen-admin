use crate::{
    common::{
        api::{ApiResponse, AppResult},
        router_ext::RouterExt,
    },
    features::auth::permission::PermissionsCheck,
};
use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use sqlx::PgPool;

use super::model::{LogQueryParams, LogResponse};
use super::service::LogService;

/// Defines the routes for log management
pub fn log_routes() -> Router<PgPool> {
    Router::new().route_with_permission(
        "/",
        get(get_log_list),
        PermissionsCheck::Any(vec!["system:*", "system:log:*", "system:log:list"]),
    )
}

/// Handles the request to get a paginated list of logs
pub async fn get_log_list(
    Query(params): Query<LogQueryParams>,
    State(pool): State<PgPool>,
) -> AppResult<Vec<LogResponse>> {
    let (logs, total) = LogService::get_log_list(&pool, params).await?;
    Ok(ApiResponse::page(logs, total))
}
