pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::PgPool;
use handler::{create_role, delete_role, get_role_options, list_roles, update_role};
use crate::{
    common::router_ext::RouterExt,
    infra::permission::PermissionsCheck,
};

/// Role management routes with permission examples
pub fn role_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(list_roles),
            PermissionsCheck::Require("system:role:list"),
        )
        .route_with_permission(
            "/",
            post(create_role),
            PermissionsCheck::Require("system:role:create"),
        )
        .route_with_permission(
            "/{id}",
            put(update_role),
            PermissionsCheck::Require("system:role:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_role),
            PermissionsCheck::Require("system:role:delete"),
        )
        .route_with_permission(
            "/options",
            get(get_role_options),
            PermissionsCheck::Require("system:role:options"),
        )
}
