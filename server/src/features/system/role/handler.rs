use super::{
    service::RoleService,
    types::{CreateRoleRequest, RoleItemResp, RoleQuery, UpdateRolePayload},
};
use crate::common::api::{ApiResponse, AppResult, OptionItem, OptionsQuery};
use crate::infra::extractor::CurrentUser;

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
    let (role_list, total) = RoleService::list_roles(&pool, query).await?;
    Ok(ApiResponse::page(role_list, total))
}

/// Create new role
pub async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleRequest>,
) -> AppResult<()> {
    RoleService::create_role(&pool, request).await?;
    Ok(ApiResponse::success(()))
}

/// Update role information
pub async fn update_role(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRolePayload>,
) -> AppResult<()> {
    RoleService::update_role(&pool, id, current_user.user_id, request).await?;
    Ok(ApiResponse::success(()))
}

/// Delete role with dependency validation
pub async fn delete_role(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<()> {
    RoleService::delete_role(&pool, id, current_user.user_id).await?;
    Ok(ApiResponse::success(()))
}

/// Get role options for dropdowns
pub async fn get_role_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<OptionItem<i64>>> {
    Ok(ApiResponse::success(RoleService::get_role_options(&pool, query).await?))
}
