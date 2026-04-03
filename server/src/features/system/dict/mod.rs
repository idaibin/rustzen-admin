pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::{common::router_ext::RouterExt, infra::permission::PermissionsCheck};
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use handler::{
    create_dict, delete_dict, get_dict_by_type, get_dict_options, list_dicts, update_dict,
    update_dict_status,
};
use sqlx::PgPool;

pub fn dict_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission("/", get(list_dicts), PermissionsCheck::Require("system:dict:list"))
        .route_with_permission(
            "/",
            post(create_dict),
            PermissionsCheck::Require("system:dict:create"),
        )
        // More specific routes first to avoid ambiguous matching with `/{id}`.
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Require("system:dict:options"),
        )
        .route_with_permission(
            "/type/{type}",
            get(get_dict_by_type),
            PermissionsCheck::Require("system:dict:options"),
        )
        .route_with_permission(
            "/{id}/status",
            patch(update_dict_status),
            PermissionsCheck::Require("system:dict:update"),
        )
        .route_with_permission(
            "/{id}",
            put(update_dict),
            PermissionsCheck::Require("system:dict:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_dict),
            PermissionsCheck::Require("system:dict:delete"),
        )
}
