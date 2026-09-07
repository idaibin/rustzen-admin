use super::{
    service::RoleService,
    types::{CreateRoleRequest, RoleItemResp, RoleOptionResp, RoleQuery, UpdateRolePayload},
};
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rustzen_auth::auth::{AuthClaims, CurrentUser};
use sqlx::SqlitePool;

/// Get paginated role list with filtering
pub async fn list_roles(
    State(pool): State<SqlitePool>,
    Query(query): Query<RoleQuery>,
) -> AppResult<Vec<RoleItemResp>> {
    let (role_list, total) = RoleService::list_roles(&pool, query).await?;
    Ok(ApiResponse::page(role_list, total))
}

/// Create new role
pub async fn create_role(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Json(request): Json<CreateRoleRequest>,
) -> AppResult<()> {
    RoleService::create_role(&pool, current_user.user_id, request, &claims).await?;
    Ok(ApiResponse::success(()))
}

/// Update role information
pub async fn update_role(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRolePayload>,
) -> AppResult<()> {
    RoleService::update_role(&pool, id, current_user.user_id, request, &claims).await?;
    Ok(ApiResponse::success(()))
}

/// Delete role with dependency validation
pub async fn delete_role(
    current_user: CurrentUser,
    Extension(claims): Extension<AuthClaims>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> AppResult<()> {
    RoleService::delete_role(&pool, id, current_user.user_id, &claims).await?;
    Ok(ApiResponse::success(()))
}

/// Get role options for dropdowns
pub async fn get_role_options(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<RoleOptionResp>> {
    Ok(ApiResponse::success(
        RoleService::get_role_options(&pool, current_user.user_id, query).await?,
    ))
}
