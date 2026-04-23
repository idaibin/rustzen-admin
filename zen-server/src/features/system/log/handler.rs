use super::{
    service::LogService,
    types::{LogItemResp, LogQuery},
};
use crate::common::api::{ApiResponse, AppResult};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::PgPool;

/// Handles the request to get a paginated list of logs
pub async fn list_logs(
    State(pool): State<PgPool>,
    Query(query): Query<LogQuery>,
) -> AppResult<Vec<LogItemResp>> {
    let (logs, total) = LogService::list_logs(&pool, query).await?;
    Ok(ApiResponse::page(logs, total))
}

pub async fn export_logs(
    State(pool): State<PgPool>,
    Query(query): Query<LogQuery>,
) -> Result<Response, (StatusCode, String)> {
    let content = LogService::export_logs_csv(&pool, query)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let filename = format!("log_{}.csv", get_timestamp());
    let disposition = format!("attachment; filename={}", filename);

    let mut headers = HeaderMap::new();
    let content_disposition = HeaderValue::from_str(&disposition).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "invalid content disposition".to_string())
    })?;
    headers.insert(header::CONTENT_DISPOSITION, content_disposition);
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(content.len()));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));

    Ok((headers, content).into_response())
}

/// get timestamp
fn get_timestamp() -> String {
    Utc::now().timestamp_millis().to_string()
}
