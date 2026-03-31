pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use handler::{get_login_info_handler, login_handler, logout_handler, update_avatar};

/// Public auth routes (no token required)
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/login", post(login_handler))
}

/// Protected auth routes (JWT required)
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(get_login_info_handler))
        .route("/logout", get(logout_handler))
        .route("/avatar", post(update_avatar))
}
