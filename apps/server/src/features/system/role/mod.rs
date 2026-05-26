pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use handler::{create_role, delete_role, get_role_options, list_roles, update_role};
use rustzen_core::{
    capability::system_role,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn role_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_roles), PermissionsCheck::Require(system_role::LIST))
        .route_with_permission(
            "/",
            post(create_role),
            PermissionsCheck::Require(system_role::CREATE),
        )
        .route_with_permission(
            "/{id}",
            put(update_role),
            PermissionsCheck::Require(system_role::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_role),
            PermissionsCheck::Require(system_role::DELETE),
        )
        .route_with_permission(
            "/options",
            get(get_role_options),
            PermissionsCheck::Require(system_role::OPTIONS),
        )
}
