pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use handler::{create_menu, delete_menu, get_menu_options, list_menus, update_menu};
use rustzen_core::permission::{PermissionsCheck, RouterExt};
use sqlx::PgPool;

pub fn menu_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission("/", get(list_menus), PermissionsCheck::Require("system:menu:list"))
        .route_with_permission(
            "/",
            post(create_menu),
            PermissionsCheck::Require("system:menu:create"),
        )
        .route_with_permission(
            "/{id}",
            put(update_menu),
            PermissionsCheck::Require("system:menu:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_menu),
            PermissionsCheck::Require("system:menu:delete"),
        )
        .route_with_permission(
            "/options",
            get(get_menu_options),
            PermissionsCheck::Require("system:menu:options"),
        )
}
