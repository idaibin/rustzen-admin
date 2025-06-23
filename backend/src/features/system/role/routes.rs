use super::model::{
    CreateRoleRequest, RoleListResponse, RoleQueryParams, RoleResponse, UpdateRoleRequest,
};
use super::service::RoleService;
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// Defines the routes for role management
pub fn role_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_role_list))
        .route("/", post(create_role))
        .route("/options", get(get_role_options))
        .route("/{id}", get(get_role_by_id))
        .route("/{id}", put(update_role))
        .route("/{id}", delete(delete_role))
        .route("/{id}/menus", get(get_role_menus))
        .route("/{id}/menus", put(set_role_menus))
}

/// Handles the request to get a paginated list of roles
async fn get_role_list(
    State(pool): State<PgPool>,
    Query(params): Query<RoleQueryParams>,
) -> AppResult<Json<ApiResponse<RoleListResponse>>> {
    let response_data = RoleService::get_role_list(&pool, params).await?;
    Ok(ApiResponse::success(response_data))
}

/// Handles the request to get a single role by its ID
async fn get_role_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    let role = RoleService::get_role_by_id(&pool, id).await?;
    Ok(ApiResponse::success(role))
}

/// Handles the request to create a new role
async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    let new_role = RoleService::create_role(&pool, request).await?;
    Ok(ApiResponse::success(new_role))
}

/// Handles the request to update an existing role
async fn update_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    let updated_role = RoleService::update_role(&pool, id, request).await?;
    Ok(ApiResponse::success(updated_role))
}

/// Handles the request to delete a role
async fn delete_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    RoleService::delete_role(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Handles the request to get role menu permissions
async fn get_role_menus(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<Vec<i64>>>> {
    let menu_ids = RoleService::get_role_menus(&pool, id).await?;
    Ok(ApiResponse::success(menu_ids))
}

/// Handles the request to set role menu permissions
async fn set_role_menus(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(menu_ids): Json<Vec<i64>>,
) -> AppResult<Json<ApiResponse<()>>> {
    RoleService::set_role_menus(&pool, id, menu_ids).await?;
    Ok(ApiResponse::success(()))
}

/// Handles the request to get role options for dropdowns
///
/// Extracts query parameters and delegates to the service layer for processing.
async fn get_role_options(
    State(pool): State<PgPool>,
    query: Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::common::api::OptionItem<i64>>>>> {
    let options = RoleService::get_role_options(&pool, query).await?;
    Ok(ApiResponse::success(options))
}
