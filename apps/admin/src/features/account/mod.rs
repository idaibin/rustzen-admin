pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::put;
#[cfg(feature = "core")]
use axum::{extract::DefaultBodyLimit, routing::post};
use sqlx::SqlitePool;

#[cfg(feature = "core")]
use handler::update_avatar;
use handler::{change_password, update_profile};

#[cfg(feature = "core")]
const AVATAR_MULTIPART_MAX_SIZE: usize = 1024 * 1024 + 64 * 1024;

pub fn account_routes() -> ContractRouter<SqlitePool> {
    let routes = ContractRouter::new();
    #[cfg(feature = "core")]
    let routes = routes
        .post(
            "/avatar",
            OperationDescriptor::UpdateAccountAvatar,
            AccessPolicy::Authenticated,
            post(update_avatar).layer(DefaultBodyLimit::max(AVATAR_MULTIPART_MAX_SIZE)),
        )
        .expect("static account contract");
    routes
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
