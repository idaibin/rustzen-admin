use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query},
};

use crate::common::api::{ApiResponse, AppResult};

use super::{
    service::TaskService,
    types::{TaskItem, TaskRunItem, TaskRunQuery},
};

pub async fn list_tasks(
    Extension(task_service): Extension<Arc<TaskService>>,
) -> AppResult<Vec<TaskItem>> {
    Ok(ApiResponse::success(task_service.list_tasks().await?))
}

pub async fn list_task_runs(
    Extension(task_service): Extension<Arc<TaskService>>,
    Path(task_key): Path<String>,
    Query(query): Query<TaskRunQuery>,
) -> AppResult<Vec<TaskRunItem>> {
    Ok(axum::Json(task_service.list_task_runs(&task_key, query).await?))
}

pub async fn run_task(
    Extension(task_service): Extension<Arc<TaskService>>,
    Path(task_key): Path<String>,
) -> AppResult<TaskRunItem> {
    Ok(ApiResponse::success(task_service.run_task(&task_key).await?))
}
