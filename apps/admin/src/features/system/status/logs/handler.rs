use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rustzen_auth::auth::CurrentUser;
use sqlx::SqlitePool;

use crate::common::{
    api::{ApiResponse, AppResult},
    error::AppError,
};

use super::{
    service::{BackupArchive, ModuleLogService},
    types::{
        ModuleLogBackupRequest, ModuleLogCleanupConfirmRequest, ModuleLogCleanupPreviewResp,
        ModuleLogCleanupResultResp, ModuleLogFileResp, ModuleLogListQuery, ModuleLogTailQuery,
        ModuleLogTailResp,
    },
};

pub async fn list_module_logs(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Query(query): Query<ModuleLogListQuery>,
) -> AppResult<Vec<ModuleLogFileResp>> {
    let items = ModuleLogService::list(&pool, &user, query).await?;
    Ok(ApiResponse::success(items))
}

pub async fn tail_module_log(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Query(query): Query<ModuleLogTailQuery>,
) -> AppResult<ModuleLogTailResp> {
    let tail = ModuleLogService::tail(&pool, &user, query).await?;
    Ok(ApiResponse::success(tail))
}

pub async fn backup_module_logs(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Json(request): Json<ModuleLogBackupRequest>,
) -> Result<Response, AppError> {
    let archive = ModuleLogService::backup(&pool, &user, request).await?;
    Ok(archive_response(archive))
}

pub async fn preview_module_log_cleanup(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
) -> AppResult<ModuleLogCleanupPreviewResp> {
    let preview = ModuleLogService::preview_cleanup(&pool, &user).await?;
    Ok(ApiResponse::success(preview))
}

pub async fn confirm_module_log_cleanup(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Json(request): Json<ModuleLogCleanupConfirmRequest>,
) -> AppResult<ModuleLogCleanupResultResp> {
    let result = ModuleLogService::confirm_cleanup(&pool, &user, request).await?;
    Ok(ApiResponse::success(result))
}

fn archive_response(archive: BackupArchive) -> Response {
    let mut response = (StatusCode::OK, Body::from(archive.bytes)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/x-tar"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=rustzen-module-logs.tar"),
    );
    headers.insert(
        header::HeaderName::from_static("x-rustzen-archive-sha256"),
        HeaderValue::from_str(&archive.archive_sha256)
            .expect("sha256 digest is always a valid header value"),
    );
    headers.insert(
        header::HeaderName::from_static("x-rustzen-archive-file-count"),
        HeaderValue::from_str(&archive.file_count.to_string())
            .expect("file count is always a valid header value"),
    );
    response
}
