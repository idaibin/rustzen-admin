pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use handler::{
    create_dict, delete_dict, get_dict_by_type, get_dict_options, list_dicts, update_dict,
    update_dict_status,
};
use rustzen_core::{
    capability::manage_dict,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn dict_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_dicts), PermissionsCheck::Require(manage_dict::LIST))
        .route_with_permission(
            "/",
            post(create_dict),
            PermissionsCheck::Require(manage_dict::CREATE),
        )
        // More specific routes first to avoid ambiguous matching with `/{id}`.
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Require(manage_dict::OPTIONS),
        )
        .route_with_permission(
            "/type/{type}",
            get(get_dict_by_type),
            PermissionsCheck::Require(manage_dict::OPTIONS),
        )
        .route_with_permission(
            "/{id}/status",
            patch(update_dict_status),
            PermissionsCheck::Require(manage_dict::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            put(update_dict),
            PermissionsCheck::Require(manage_dict::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_dict),
            PermissionsCheck::Require(manage_dict::DELETE),
        )
}
