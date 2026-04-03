pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use handler::{get_login_info, login, logout, update_avatar};

pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/login", post(login))
}

pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(get_login_info))
        .route("/logout", get(logout))
        .route("/avatar", post(update_avatar))
}
