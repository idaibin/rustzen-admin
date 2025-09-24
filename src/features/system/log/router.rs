use super::{dto::LogQueryDto, service::LogService, vo::LogItemVo};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        router_ext::RouterExt,
    },
    core::permission::PermissionsCheck,
};

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use sqlx::PgPool;

/// Defines the routes for log management
pub fn log_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_log_list),
            PermissionsCheck::Any(vec!["system:*", "system:log:*", "system:log:list"]),
        )
        .route_with_permission(
            "/export",
            get(export_log_list),
            PermissionsCheck::Any(vec!["system:*", "system:log:*", "system:log:export"]),
        )
}

/// Handles the request to get a paginated list of logs
pub async fn get_log_list(
    State(pool): State<PgPool>,
    Query(query): Query<LogQueryDto>,
) -> AppResult<Vec<LogItemVo>> {
    let (logs, total) = LogService::get_log_list(&pool, query).await?;
    Ok(ApiResponse::page(logs, total))
}

pub async fn export_log_list(
    State(pool): State<PgPool>,
    Query(query): Query<LogQueryDto>,
) -> Result<Response, (StatusCode, String)> {
    let content = LogService::get_all_log_csv(&pool, query).await.unwrap();

    let filename = format!("log_{}.csv", get_timestamp());
    let disposition = format!("attachment; filename={}", filename);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_str(&disposition).unwrap());
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(content.len()));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));

    Ok((headers, content).into_response())
}

/// get timestamp
fn get_timestamp() -> String {
    let now = Utc::now().timestamp_millis();
    format!("{}", now)
}
