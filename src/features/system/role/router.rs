use super::{
    dto::{CreateRoleDto, RoleQueryDto, UpdateRoleDto},
    service::RoleService,
    vo::RoleItemVo,
};
use crate::{
    common::{
        api::{ApiResponse, AppResult, OptionItem, OptionsQuery},
        router_ext::RouterExt,
    },
    core::permission::PermissionsCheck,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// Role management routes with permission examples
pub fn role_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_role_list),
            PermissionsCheck::Any(vec!["system:*", "system:role:*", "system:role:list"]),
        )
        .route_with_permission(
            "/",
            post(create_role),
            PermissionsCheck::Any(vec!["system:*", "system:role:*", "system:role:create"]),
        )
        .route_with_permission(
            "/{id}",
            put(update_role),
            PermissionsCheck::Any(vec!["system:*", "system:role:*", "system:role:update"]),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_role),
            PermissionsCheck::Any(vec!["system:*", "system:role:*", "system:role:delete"]),
        )
        .route_with_permission(
            "/options",
            get(get_role_options),
            PermissionsCheck::Any(vec!["system:*", "system:role:*", "system:role:options"]),
        )
}

/// Get paginated role list with filtering
async fn get_role_list(
    State(pool): State<PgPool>,
    Query(query): Query<RoleQueryDto>,
) -> AppResult<Vec<RoleItemVo>> {
    tracing::info!("Role list request: query={:?}", query);

    let (role_list, total) = RoleService::get_role_list(&pool, query).await?;

    tracing::info!("Role list retrieved: total={}, returned={}", total, role_list.len());

    Ok(ApiResponse::page(role_list, total))
}

/// Create new role
async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleDto>,
) -> AppResult<()> {
    tracing::info!("Create role: name={}, menus={}", request.name, request.menu_ids.len());

    RoleService::create_role(&pool, request).await?;

    tracing::info!("Role created");

    Ok(ApiResponse::success(()))
}

/// Update role information
async fn update_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRoleDto>,
) -> AppResult<()> {
    tracing::info!("Update role {}: name={:?}, menus={:?}", id, request.name, request.menu_ids);

    RoleService::update_role(&pool, id, request).await?;

    tracing::info!("Role updated: id={}", id);

    Ok(ApiResponse::success(()))
}

/// Delete role with dependency validation
async fn delete_role(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Delete role: {}", id);

    RoleService::delete_role(&pool, id).await?;

    tracing::info!("Role deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Get role options for dropdowns
async fn get_role_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<OptionItem<i64>>> {
    tracing::debug!("Role options: q={:?}, limit={:?}", query.q, query.limit);

    let options = RoleService::get_role_options(&pool, query).await?;

    tracing::debug!("Role options returned: {}", options.len());

    Ok(ApiResponse::success(options))
}
