mod bundle;
pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
};
use handler::{
    cleanup_expired, delete_version, deploy_version, expire_version, get_deployment,
    list_deployments, upload_deployment,
};
use rustzen_auth::capability::manage_deploy;
use sqlx::SqlitePool;

use service::DeployService;

pub fn deploy_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/list",
            OperationDescriptor::ListDeployments,
            AccessPolicy::Require(manage_deploy::LIST),
            get(list_deployments),
        )
        .expect("static deploy contract")
        .post(
            "/upload",
            OperationDescriptor::UploadDeployment,
            AccessPolicy::Require(manage_deploy::CREATE),
            post(upload_deployment)
                .layer(DefaultBodyLimit::max(DeployService::upload_body_limit())),
        )
        .expect("static deploy contract")
        .post(
            "/cleanup",
            OperationDescriptor::CleanupDeployments,
            AccessPolicy::Require(manage_deploy::DELETE),
            post(cleanup_expired),
        )
        .expect("static deploy contract")
        .get(
            "/{id}",
            OperationDescriptor::GetDeployment,
            AccessPolicy::Require(manage_deploy::LIST),
            get(get_deployment),
        )
        .expect("static deploy contract")
        .put(
            "/{id}/expire",
            OperationDescriptor::ExpireDeployment,
            AccessPolicy::Require(manage_deploy::UPDATE),
            put(expire_version),
        )
        .expect("static deploy contract")
        .delete(
            "/{id}",
            OperationDescriptor::DeleteDeployment,
            AccessPolicy::Require(manage_deploy::DELETE),
            delete(delete_version),
        )
        .expect("static deploy contract")
        .post(
            "/{id}/deploy",
            OperationDescriptor::DeployVersion,
            AccessPolicy::Require(manage_deploy::RUN),
            post(deploy_version),
        )
        .expect("static deploy contract")
}
