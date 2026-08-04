pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{delete, get, post, put};
use handler::{create_role, delete_role, get_role_options, list_roles, update_role};
use rustzen_auth::capability::system_role;
use sqlx::SqlitePool;

pub fn role_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListRoles,
            AccessPolicy::Require(system_role::LIST),
            get(list_roles),
        )
        .expect("static role contract")
        .post(
            "/",
            OperationDescriptor::CreateRole,
            AccessPolicy::Require(system_role::CREATE),
            post(create_role),
        )
        .expect("static role contract")
        .put(
            "/{id}",
            OperationDescriptor::UpdateRole,
            AccessPolicy::Require(system_role::UPDATE),
            put(update_role),
        )
        .expect("static role contract")
        .delete(
            "/{id}",
            OperationDescriptor::DeleteRole,
            AccessPolicy::Require(system_role::DELETE),
            delete(delete_role),
        )
        .expect("static role contract")
        .get(
            "/options",
            OperationDescriptor::GetRoleOptions,
            AccessPolicy::Require(system_role::OPTIONS),
            get(get_role_options),
        )
        .expect("static role contract")
}
