pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use handler::{
    create_user, delete_user, get_user_options, get_user_status_options, list_users, update_user,
    update_user_password, update_user_status,
};
use rustzen_core::{
    capability::system_user,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn user_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_users), PermissionsCheck::Require(system_user::LIST))
        .route_with_permission(
            "/",
            post(create_user),
            PermissionsCheck::Require(system_user::CREATE),
        )
        .route_with_permission(
            "/{id}",
            put(update_user),
            PermissionsCheck::Require(system_user::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_user),
            PermissionsCheck::Require(system_user::DELETE),
        )
        .route_with_permission(
            "/options",
            get(get_user_options),
            PermissionsCheck::Require(system_user::OPTIONS),
        )
        .route_with_permission(
            "/status-options",
            get(get_user_status_options),
            PermissionsCheck::Require(system_user::OPTIONS),
        )
        .route_with_permission(
            "/{id}/password",
            put(update_user_password),
            PermissionsCheck::Require(system_user::RESET_PASSWORD),
        )
        .route_with_permission(
            "/{id}/status",
            put(update_user_status),
            PermissionsCheck::Require(system_user::UPDATE_STATUS),
        )
}
