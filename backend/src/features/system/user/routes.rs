use super::handler;
use crate::common::router_ext::RouterExt;
use crate::features::auth::permission::PermissionsCheck;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

/// User management routes
pub fn user_routes() -> Router<sqlx::PgPool> {
    Router::new()
        // User CRUD operations
        .route_with_permission(
            "/",
            get(handler::get_user_list),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
        .route_with_permission(
            "/",
            post(handler::create_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:create"]),
        )
        .route_with_permission(
            "/{id}",
            get(handler::get_user_by_id),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:detail"]),
        )
        .route_with_permission(
            "/{id}",
            put(handler::update_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:update"]),
        )
        .route_with_permission(
            "/{id}",
            delete(handler::delete_user),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:delete"]),
        )
        // User management operations
        // .route_with_permission(
        //     "/{id}/enable",
        //     patch(handler::enable_user),
        //     PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:update"]),
        // )
        // .route_with_permission(
        //     "/{id}/disable",
        //     patch(handler::disable_user),
        //     PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:update"]),
        // )
        // .route_with_permission(
        //     "/{id}/reset-password",
        //     patch(handler::reset_user_password),
        //     PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:update"]),
        // )
        // Dropdown options
        .route_with_permission(
            "/options",
            get(handler::get_user_options),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
        .route_with_permission(
            "/status-options",
            get(handler::get_user_status_options),
            PermissionsCheck::Any(vec!["system:*", "system:user:*", "system:user:list"]),
        )
}
