use super::model::{
    CreateRoleRequest, RoleListResponse, RoleQueryParams, RoleResponse, UpdateRoleRequest,
};
use super::service::RoleService;
use crate::common::api::ApiResponse;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// 角色路由
pub fn role_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_role_list))
        .route("/", post(create_role))
        .route("/{id}", get(get_role_by_id))
        .route("/{id}", put(update_role))
        .route("/{id}", delete(delete_role))
        .route("/{id}/menus", get(get_role_menus))
        .route("/{id}/menus", put(set_role_menus))
}

/// 获取角色列表
async fn get_role_list(
    State(pool): State<PgPool>,
    Query(params): Query<RoleQueryParams>,
) -> Json<ApiResponse<RoleListResponse>> {
    match RoleService::get_role_list(&pool, params).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取角色列表失败: {}", e);
            ApiResponse::fail(500, "获取角色列表失败".to_string())
        }
    }
}

/// 根据 ID 获取角色
async fn get_role_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<RoleResponse>> {
    match RoleService::get_role_by_id(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取角色失败: {}", e);
            ApiResponse::fail(500, "获取角色失败".to_string())
        }
    }
}

/// 创建角色
async fn create_role(
    State(pool): State<PgPool>,
    Json(request): Json<CreateRoleRequest>,
) -> Json<ApiResponse<RoleResponse>> {
    match RoleService::create_role(&pool, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("创建角色失败: {}", e);
            ApiResponse::fail(500, "创建角色失败".to_string())
        }
    }
}

/// 更新角色
async fn update_role(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateRoleRequest>,
) -> Json<ApiResponse<RoleResponse>> {
    match RoleService::update_role(&pool, id, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("更新角色失败: {}", e);
            ApiResponse::fail(500, "更新角色失败".to_string())
        }
    }
}

/// 删除角色
async fn delete_role(State(pool): State<PgPool>, Path(id): Path<i64>) -> Json<ApiResponse<()>> {
    match RoleService::delete_role(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("删除角色失败: {}", e);
            ApiResponse::fail(500, "删除角色失败".to_string())
        }
    }
}

/// 获取角色菜单权限
async fn get_role_menus(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<Vec<i64>>> {
    match RoleService::get_role_menus(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取角色菜单权限失败: {}", e);
            ApiResponse::fail(500, "获取角色菜单权限失败".to_string())
        }
    }
}

/// 设置角色菜单权限
async fn set_role_menus(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(menu_ids): Json<Vec<i64>>,
) -> Json<ApiResponse<()>> {
    match RoleService::set_role_menus(&pool, id, menu_ids).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("设置角色菜单权限失败: {}", e);
            ApiResponse::fail(500, "设置角色菜单权限失败".to_string())
        }
    }
}
