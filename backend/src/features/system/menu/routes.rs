use super::model::{
    CreateMenuRequest, MenuListResponse, MenuQueryParams, MenuResponse, UpdateMenuRequest,
};
use super::service::MenuService;
use crate::common::api::ApiResponse;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// 菜单路由
pub fn menu_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_menu_list))
        .route("/", post(create_menu))
        .route("/{id}", get(get_menu_by_id))
        .route("/{id}", put(update_menu))
        .route("/{id}", delete(delete_menu))
}

/// 获取菜单列表（树形结构）
async fn get_menu_list(
    State(pool): State<PgPool>,
    Query(params): Query<MenuQueryParams>,
) -> Json<ApiResponse<MenuListResponse>> {
    match MenuService::get_menu_list(&pool, params).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取菜单列表失败: {}", e);
            ApiResponse::fail(500, "获取菜单列表失败".to_string())
        }
    }
}

/// 根据 ID 获取菜单
async fn get_menu_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<MenuResponse>> {
    match MenuService::get_menu_by_id(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取菜单失败: {}", e);
            ApiResponse::fail(500, "获取菜单失败".to_string())
        }
    }
}

/// 创建菜单
async fn create_menu(
    State(pool): State<PgPool>,
    Json(request): Json<CreateMenuRequest>,
) -> Json<ApiResponse<MenuResponse>> {
    match MenuService::create_menu(&pool, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("创建菜单失败: {}", e);
            ApiResponse::fail(500, "创建菜单失败".to_string())
        }
    }
}

/// 更新菜单
async fn update_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuRequest>,
) -> Json<ApiResponse<MenuResponse>> {
    match MenuService::update_menu(&pool, id, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("更新菜单失败: {}", e);
            ApiResponse::fail(500, "更新菜单失败".to_string())
        }
    }
}

/// 删除菜单
async fn delete_menu(State(pool): State<PgPool>, Path(id): Path<i64>) -> Json<ApiResponse<()>> {
    match MenuService::delete_menu(&pool, id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("删除菜单失败: {}", e);
            ApiResponse::fail(500, "删除菜单失败".to_string())
        }
    }
}
