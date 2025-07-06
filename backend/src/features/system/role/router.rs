use super::model::{CreateRoleRequest, RoleQueryParams, RoleResponse, UpdateRoleRequest};
use super::service::RoleService;
use crate::common::{
    api::{ApiResponse, AppResult, OptionItem, OptionsQuery},
    router_ext::RouterExt,
};
use crate::features::auth::permission::PermissionsCheck;
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
            PermissionsCheck::Single("system:role:list"),
        )
        .route_with_permission(
            "/",
            post(create_role),
            PermissionsCheck::Single("system:role:create"),
        )
        .route_with_permission(
            "/options",
            get(get_role_options),
            PermissionsCheck::Single("system:role:options"),
        )
        .route_with_permission(
            "/{id}",
            get(get_role_by_id),
            PermissionsCheck::Single("system:role:get"),
        )
        .route_with_permission(
            "/{id}",
            put(update_role),
            PermissionsCheck::Single("system:role:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_role),
            PermissionsCheck::Single("system:role:delete"),
        )
    // .route_with_permission("/{id}/menus", get(get_role_menus), "system:role:menus:get")
    // .route_with_permission("/{id}/menus", put(set_role_menus), "system:role:menus:set")
}

/// Get paginated role list with filtering
/// Query params: current, page_size, name, status
async fn get_role_list(
    State(pool): State<PgPool>,
    Query(params): Query<RoleQueryParams>,
) -> AppResult<Json<ApiResponse<Vec<RoleResponse>>>> {
    tracing::info!(
        "Role list request: page={}, size={}, name={:?}, status={:?}",
        params.current.unwrap_or(1),
        params.page_size.unwrap_or(10),
        params.role_name,
        params.status
    );

    let (role_list, total) = RoleService::get_role_list(&pool, params).await?;

    tracing::info!("Role list retrieved: total={}, returned={}", total, role_list.len());

    Ok(ApiResponse::page(role_list, total))
}

/// Get role by ID with permissions
async fn get_role_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    tracing::info!("Get role by ID: {}", id);

    let role = RoleService::get_role_by_id(&pool, id).await?;

    tracing::info!(
        "Role retrieved: id={}, name={}, menus={}",
        role.id,
        role.role_name,
        role.menu_ids.len()
    );

    Ok(ApiResponse::success(role))
}

/// Create new role
/// Body: name, description, status, permissions
async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    tracing::info!("Create role: name={}, menus={}", request.role_name, request.menu_ids.len());

    let new_role = RoleService::create_role(&pool, request).await?;

    tracing::info!(
        "Role created: id={}, name={}, menus={}",
        new_role.id,
        new_role.role_name,
        new_role.menu_ids.len()
    );

    Ok(ApiResponse::success(new_role))
}

/// Update role information
/// Body: name, description, status, permissions (all optional)
async fn update_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    tracing::info!(
        "Update role {}: name={:?}, menus={}",
        id,
        request.role_name,
        request.menu_ids.as_ref().map_or(0, |m| m.len())
    );

    let updated_role = RoleService::update_role(&pool, id, request).await?;

    tracing::info!(
        "Role updated: id={}, name={}, menus={}",
        updated_role.id,
        updated_role.role_name,
        updated_role.menu_ids.len()
    );

    Ok(ApiResponse::success(updated_role))
}

/// Delete role with dependency validation
async fn delete_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Delete role: {}", id);

    RoleService::delete_role(&pool, id).await?;

    tracing::info!("Role deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Get role options for dropdowns
/// Query params: q (search), limit, status, exclude_id
async fn get_role_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<OptionItem<i64>>>>> {
    tracing::debug!("Role options: q={:?}, limit={:?}", query.q, query.limit);

    let options = RoleService::get_role_options(&pool, Query(query)).await?;

    tracing::debug!("Role options returned: {}", options.len());

    Ok(ApiResponse::success(options))
}
