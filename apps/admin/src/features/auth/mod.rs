pub mod handler;
pub mod repo;
pub mod service;
pub mod session;
pub mod types;

#[cfg(all(test, feature = "full"))]
#[path = "actor_authority_tests.rs"]
mod actor_authority_tests;

#[cfg(test)]
#[path = "session_cleanup_tests.rs"]
mod session_cleanup_tests;

#[cfg(test)]
#[path = "epoch_trigger_tests.rs"]
mod epoch_trigger_tests;

#[cfg(test)]
#[path = "login_concurrency_tests.rs"]
mod login_concurrency_tests;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{get, post};
use sqlx::SqlitePool;

use handler::{get_login_info, login, logout};

pub fn public_auth_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .post("/api/auth/login", OperationDescriptor::Login, AccessPolicy::Public, post(login))
        .expect("static auth contract")
}

/// `GET /api/auth/me` is documented from the same registration that builds its
/// Axum route. JWT authentication is deliberately applied by the outer Admin API
/// layer, so this route only declares `Authenticated` here.
pub fn protected_auth_contract_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/me",
            OperationDescriptor::CurrentAdminUser,
            AccessPolicy::Authenticated,
            get(get_login_info),
        )
        .expect("static auth contract")
        .get("/logout", OperationDescriptor::Logout, AccessPolicy::Authenticated, get(logout))
        .expect("static auth contract")
}
