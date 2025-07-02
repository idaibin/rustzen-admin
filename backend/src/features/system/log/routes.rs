use crate::common::api::{ApiResponse, AppResult};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sqlx::PgPool;

use super::model::{CreateLogRequest, LogQueryParams, LogResponse};
use super::service::LogService;

/// Defines the routes for log management
pub fn log_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_log_list).post(create_log)).route("/{id}", get(get_log_by_id))
}

/// Handles the request to get a paginated list of logs
pub async fn get_log_list(
    Query(params): Query<LogQueryParams>,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<Vec<LogResponse>>>> {
    let (logs, total) = LogService::get_log_list(&pool, params).await?;
    Ok(ApiResponse::page(logs, total))
}

/// Handles the request to get a specific log entry by ID
async fn get_log_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<ApiResponse<LogResponse>>> {
    let log = LogService::get_log_by_id(&pool, id).await?;
    Ok(ApiResponse::success(log))
}

/// Handles the request to create a new log entry
async fn create_log(
    State(pool): State<PgPool>,
    Json(request): Json<CreateLogRequest>,
) -> AppResult<Json<ApiResponse<LogResponse>>> {
    let log = LogService::create_log(
        &pool,
        request.level,
        request.message,
        request.user_id,
        request.ip_address,
    )
    .await?;

    Ok(ApiResponse::success(log))
}
