use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query},
    response::IntoResponse,
};

use crate::common::api::{ApiResponse, AppResult};
use rustzen_auth::auth::CurrentUser;

use super::{
    service::DeployService,
    types::{CleanupDeploymentsQuery, DeploymentItem, ExpireVersionRequest, ListDeploymentsQuery},
};

pub async fn list_deployments(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Query(query): Query<ListDeploymentsQuery>,
) -> AppResult<Vec<DeploymentItem>> {
    let (items, total) = deploy_service.list(query).await?;
    Ok(ApiResponse::page(items, total))
}

pub async fn upload_deployment(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    multipart: Multipart,
) -> Result<axum::response::Response, crate::common::error::AppError> {
    match deploy_service.upload(multipart).await {
        Ok(item) => Ok(ApiResponse::success(item).into_response()),
        Err(crate::common::error::ServiceError::PayloadTooLarge) => {
            Ok((axum::http::StatusCode::PAYLOAD_TOO_LARGE, "Request payload is too large")
                .into_response())
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn get_deployment(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
) -> AppResult<DeploymentItem> {
    Ok(ApiResponse::success(deploy_service.find_by_id(id).await?))
}

pub async fn deploy_version(
    current_user: CurrentUser,
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(deploy_service.deploy(id, current_user.username).await?))
}

pub async fn expire_version(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
    Json(request): Json<ExpireVersionRequest>,
) -> AppResult<DeploymentItem> {
    Ok(ApiResponse::success(deploy_service.expire(id, request).await?))
}

pub async fn delete_version(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
) -> AppResult<DeploymentItem> {
    Ok(ApiResponse::success(deploy_service.delete(id).await?))
}

pub async fn cleanup_expired(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Query(query): Query<CleanupDeploymentsQuery>,
) -> AppResult<usize> {
    Ok(ApiResponse::success(deploy_service.cleanup_expired(query.component).await?))
}
