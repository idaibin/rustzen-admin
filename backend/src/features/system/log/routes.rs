use crate::common::api::{ApiResponse, AppResult};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

use super::service::LogService;

/// Query parameters for log list API
#[derive(Debug, Deserialize)]
pub struct LogListQuery {
    /// Search term for filtering log messages
    pub q: Option<String>,
    /// Page number (1-based)
    pub page: Option<i64>,
    /// Number of records per page
    pub page_size: Option<i64>,
}

/// Request body for creating new log entries
#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    /// Log level (INFO, WARN, ERROR, DEBUG)
    pub level: String,
    /// Log message content
    pub message: String,
    /// Optional user ID associated with the log
    pub user_id: Option<i32>,
    /// Optional IP address
    pub ip_address: Option<String>,
}

/// Response for paginated log list
#[derive(Debug, Serialize)]
pub struct PaginatedLogsResponse {
    /// List of log entries
    pub items: Vec<Value>,
    /// Total number of matching records
    pub total: i64,
    /// Current page number
    pub page: i64,
    /// Number of records per page
    pub page_size: i64,
}

/// Defines the routes for log management
pub fn log_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_log_list).post(create_log)).route("/:id", get(get_log_by_id))
}

/// Handles the request to get a paginated list of logs
async fn get_log_list(
    State(pool): State<PgPool>,
    Query(query): Query<LogListQuery>,
) -> AppResult<Json<ApiResponse<PaginatedLogsResponse>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (logs, total) =
        LogService::get_log_list(&pool, query.q, Some(page), Some(page_size)).await?;

    let response = PaginatedLogsResponse { items: logs, total, page, page_size };

    Ok(ApiResponse::success(response))
}

/// Handles the request to get a specific log entry by ID
async fn get_log_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<ApiResponse<Value>>> {
    let log = LogService::get_log_by_id(&pool, id).await?;
    Ok(ApiResponse::success(log))
}

/// Handles the request to create a new log entry
async fn create_log(
    State(pool): State<PgPool>,
    Json(request): Json<CreateLogRequest>,
) -> AppResult<Json<ApiResponse<Value>>> {
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
