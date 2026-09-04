pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::{
    extract::DefaultBodyLimit,
    routing::{post, put},
};
use sqlx::SqlitePool;

use handler::{change_password, update_avatar, update_profile};

const AVATAR_MULTIPART_MAX_SIZE: usize = 1024 * 1024 + 64 * 1024;

pub fn account_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .post(
            "/avatar",
            OperationDescriptor::UpdateAccountAvatar,
            AccessPolicy::Authenticated,
            post(update_avatar).layer(DefaultBodyLimit::max(AVATAR_MULTIPART_MAX_SIZE)),
        )
        .expect("static account contract")
        .put(
            "/profile",
            OperationDescriptor::UpdateAccountProfile,
            AccessPolicy::Authenticated,
            put(update_profile),
        )
        .expect("static account contract")
        .put(
            "/password",
            OperationDescriptor::ChangeAccountPassword,
            AccessPolicy::Authenticated,
            put(change_password),
        )
        .expect("static account contract")
}
