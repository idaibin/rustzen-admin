use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query},
};

use crate::common::api::{ApiResponse, AppResult};

use super::{
    service::DeployService,
    types::{
        DeployComponent, DeployVersionRequest, DeploymentItem, ExpireVersionRequest,
        ListDeploymentsQuery,
    },
};

pub async fn list_deployments(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Query(query): Query<ListDeploymentsQuery>,
) -> AppResult<Vec<DeploymentItem>> {
    Ok(Json(deploy_service.list(query).await?))
}

pub async fn upload_deployment(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    multipart: Multipart,
) -> AppResult<DeploymentItem> {
    Ok(ApiResponse::success(deploy_service.upload(multipart).await?))
}

pub async fn get_deployment(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
) -> AppResult<DeploymentItem> {
    Ok(ApiResponse::success(deploy_service.find_by_id(id).await?))
}

pub async fn deploy_version(
    Extension(deploy_service): Extension<Arc<DeployService>>,
    Path(id): Path<i64>,
    Json(request): Json<DeployVersionRequest>,
) -> AppResult<bool> {
    Ok(ApiResponse::success(deploy_service.deploy(id, request).await?))
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
    Ok(ApiResponse::success(
        deploy_service.cleanup_expired(query.component).await?,
    ))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupDeploymentsQuery {
    pub component: Option<DeployComponent>,
}
