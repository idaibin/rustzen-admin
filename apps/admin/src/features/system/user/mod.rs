pub mod handler;
pub mod repo;
mod repo_mutation;
mod repo_security;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{delete, get, post, put};
use handler::{
    create_user, delete_user, get_user_options, get_user_status_options, list_users,
    revoke_user_sessions, update_user, update_user_password, update_user_status,
};
use rustzen_auth::capability::system_user;
use sqlx::SqlitePool;

pub fn user_contract_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListUsers,
            AccessPolicy::Require(system_user::LIST),
            get(list_users),
        )
        .expect("static user contract")
        .post(
            "/",
            OperationDescriptor::CreateAdminUser,
            AccessPolicy::Require(system_user::CREATE),
            post(create_user),
        )
        .expect("static user contract")
        .put(
            "/{id}",
            OperationDescriptor::UpdateUser,
            AccessPolicy::Require(system_user::UPDATE),
            put(update_user),
        )
        .expect("static user contract")
        .delete(
            "/{id}",
            OperationDescriptor::DeleteUser,
            AccessPolicy::Require(system_user::DELETE),
            delete(delete_user),
        )
        .expect("static user contract")
        .get(
            "/options",
            OperationDescriptor::GetUserOptions,
            AccessPolicy::Require(system_user::OPTIONS),
            get(get_user_options),
        )
        .expect("static user contract")
        .get(
            "/status-options",
            OperationDescriptor::GetUserStatusOptions,
            AccessPolicy::Require(system_user::OPTIONS),
            get(get_user_status_options),
        )
        .expect("static user contract")
        .put(
            "/{id}/password",
            OperationDescriptor::UpdateUserPassword,
            AccessPolicy::Require(system_user::RESET_PASSWORD),
            put(update_user_password),
        )
        .expect("static user contract")
        .put(
            "/{id}/status",
            OperationDescriptor::UpdateUserStatus,
            AccessPolicy::Require(system_user::UPDATE_STATUS),
            put(update_user_status),
        )
        .expect("static user contract")
        .post(
            "/{id}/sessions/revoke-all",
            OperationDescriptor::RevokeUserSessions,
            AccessPolicy::Require(system_user::RESET_PASSWORD),
            post(revoke_user_sessions),
        )
        .expect("static user contract")
}
