pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use handler::{create_menu, delete_menu, get_menu_options, list_menus, update_menu};
use rustzen_core::{
    capability::system_menu,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn menu_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_menus), PermissionsCheck::Require(system_menu::LIST))
        .route_with_permission(
            "/",
            post(create_menu),
            PermissionsCheck::Require(system_menu::CREATE),
        )
        .route_with_permission(
            "/{id}",
            put(update_menu),
            PermissionsCheck::Require(system_menu::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_menu),
            PermissionsCheck::Require(system_menu::DELETE),
        )
        .route_with_permission(
            "/options",
            get(get_menu_options),
            PermissionsCheck::Require(system_menu::OPTIONS),
        )
}
