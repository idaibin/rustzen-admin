pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{post, put},
};
use sqlx::SqlitePool;

use handler::{change_password, update_avatar, update_profile};

pub fn account_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/avatar", post(update_avatar))
        .route("/profile", put(update_profile))
        .route("/password", put(change_password))
}
