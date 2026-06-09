pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    extract::DefaultBodyLimit,
    Router,
    routing::{delete, get, post, put},
};
use handler::{
    cleanup_expired, delete_version, deploy_version, expire_version, get_deployment,
    list_deployments, upload_deployment,
};
use rustzen_core::{
    capability::manage_deploy,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

use service::DeployService;

pub fn deploy_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission(
            "/list",
            get(list_deployments),
            PermissionsCheck::Require(manage_deploy::LIST),
        )
        .route_with_permission(
            "/upload",
            post(upload_deployment).layer(DefaultBodyLimit::max(DeployService::upload_body_limit())),
            PermissionsCheck::Require(manage_deploy::CREATE),
        )
        .route_with_permission(
            "/cleanup",
            post(cleanup_expired),
            PermissionsCheck::Require(manage_deploy::DELETE),
        )
        .route_with_permission(
            "/{id}",
            get(get_deployment),
            PermissionsCheck::Require(manage_deploy::LIST),
        )
        .route_with_permission(
            "/{id}/expire",
            put(expire_version),
            PermissionsCheck::Require(manage_deploy::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_version),
            PermissionsCheck::Require(manage_deploy::DELETE),
        )
        .route_with_permission(
            "/{id}/deploy",
            post(deploy_version),
            PermissionsCheck::Require(manage_deploy::RUN),
        )
}
