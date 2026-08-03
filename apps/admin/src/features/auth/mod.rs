pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;

use handler::{get_login_info, login, logout};

pub fn public_auth_routes() -> Router<SqlitePool> {
    Router::new().route("/login", post(login))
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
        .merge_router(Router::new().route("/logout", get(logout)))
}
