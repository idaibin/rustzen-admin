pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;

use handler::{get_login_info, login, logout};

pub fn public_auth_routes() -> Router<SqlitePool> {
    Router::new().route("/login", post(login))
}

pub fn protected_auth_routes() -> Router<SqlitePool> {
    Router::new().route("/me", get(get_login_info)).route("/logout", get(logout))
}
