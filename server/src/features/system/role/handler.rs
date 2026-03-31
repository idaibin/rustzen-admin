use super::{
    service::RoleService,
    types::{CreateRoleRequest, RoleItemResp, RoleQuery, UpdateRolePayload},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult, OptionItem, OptionsQuery},
    },
};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;

/// Get paginated role list with filtering
pub async fn list_roles(
    State(pool): State<PgPool>,
    Query(query): Query<RoleQuery>,
) -> AppResult<Vec<RoleItemResp>> {
    tracing::info!("Role list request: query={:?}", query);

    let (role_list, total) = RoleService::list_roles(&pool, query).await?;

    tracing::info!("Role list retrieved: total={}, returned={}", total, role_list.len());

    Ok(ApiResponse::page(role_list, total))
}

/// Create new role
pub async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleRequest>,
) -> AppResult<()> {
    tracing::info!("Create role: name={}, menus={}", request.name, request.menu_ids.len());

    RoleService::create_role(&pool, request).await?;

    tracing::info!("Role created");

    Ok(ApiResponse::success(()))
}

/// Update role information
pub async fn update_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRolePayload>,
) -> AppResult<()> {
    tracing::info!("Update role {}: name={:?}, menus={:?}", id, request.name, request.menu_ids);

    RoleService::update_role(&pool, id, request).await?;

    tracing::info!("Role updated: id={}", id);

    Ok(ApiResponse::success(()))
}

/// Delete role with dependency validation
pub async fn delete_role(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Delete role: {}", id);

    RoleService::delete_role(&pool, id).await?;

    tracing::info!("Role deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Get role options for dropdowns
pub async fn get_role_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<OptionItem<i64>>> {
    tracing::debug!("Role options: q={:?}, limit={:?}", query.q, query.limit);

    let options = RoleService::get_role_options(&pool, query).await?;

    tracing::debug!("Role options returned: {}", options.len());

    Ok(ApiResponse::success(options))
}
