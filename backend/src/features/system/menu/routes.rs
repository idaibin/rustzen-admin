use super::model::{CreateMenuRequest, MenuQueryParams, MenuResponse, UpdateMenuRequest};
use super::service::MenuService;
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};
use crate::common::router_ext::RouterExt;
use crate::features::auth::permission::PermissionsCheck;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// Menu management routes with permission examples
pub fn menu_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_menu_list),
            PermissionsCheck::Single("system:menu:list"),
        )
        .route_with_permission(
            "/",
            post(create_menu),
            PermissionsCheck::Single("system:menu:create"),
        )
        .route_with_permission(
            "/options",
            get(get_menu_options),
            PermissionsCheck::Single("system:menu:options"),
        )
        .route_with_permission(
            "/{id}",
            get(get_menu_by_id),
            PermissionsCheck::Single("system:menu:get"),
        )
        .route_with_permission(
            "/{id}",
            put(update_menu),
            PermissionsCheck::Single("system:menu:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_menu),
            PermissionsCheck::Single("system:menu:delete"),
        )
}

/// Get menu list with optional filtering
/// Query params: title, status
async fn get_menu_list(
    State(pool): State<PgPool>,
    Query(params): Query<MenuQueryParams>,
) -> AppResult<Json<ApiResponse<Vec<MenuResponse>>>> {
    tracing::info!("Menu list request: {:?}", params);

    let (menu_list, total) = MenuService::get_menu_list(&pool, params).await?;

    tracing::info!("Menu list retrieved: total={}, items={}", total, menu_list.len());

    Ok(ApiResponse::page(menu_list, total))
}

/// Get menu by ID
async fn get_menu_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::get_menu_by_id(&pool, id).await?;
    Ok(ApiResponse::success(response))
}

/// Create new menu
/// Body: name, path, parent_id, icon, sort_order, status
async fn create_menu(
    State(pool): State<PgPool>,
    Json(request): Json<CreateMenuRequest>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::create_menu(&pool, request).await?;
    Ok(ApiResponse::success(response))
}

/// Update menu
/// Body: name, path, parent_id, icon, sort_order, status (all optional)
async fn update_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuRequest>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::update_menu(&pool, id, request).await?;
    Ok(ApiResponse::success(response))
}

/// Delete menu (handles child cleanup)
async fn delete_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    MenuService::delete_menu(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Get menu options for dropdowns
/// Query params: q (search), limit, exclude_id
async fn get_menu_options(
    State(pool): State<PgPool>,
    query: Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::common::api::OptionItem<i64>>>>> {
    let options = MenuService::get_menu_options(&pool, query).await?;
    Ok(ApiResponse::success(options))
}
