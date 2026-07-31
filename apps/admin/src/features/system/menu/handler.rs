use super::{
    service::MenuService,
    types::{MenuItemResp, MenuOptionResp, MenuQuery, UpdateMenuPayload},
};
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use rustzen_auth::auth::CurrentUser;
use sqlx::SqlitePool;

/// Get menu list with optional filtering
/// Query params: title, status
/// Need show all menu, not pagination
pub async fn list_menus(
    State(pool): State<SqlitePool>,
    Query(params): Query<MenuQuery>,
) -> AppResult<Vec<MenuItemResp>> {
    let (menu_list, total) = MenuService::list_menus(&pool, params).await?;
    Ok(ApiResponse::page(menu_list, total))
}

/// List the active module-owned menu inventory, including disabled presentation rows.
pub async fn list_module_menu_inventory(
    State(pool): State<SqlitePool>,
) -> AppResult<Vec<MenuItemResp>> {
    Ok(ApiResponse::success(MenuService::list_module_menu_inventory(&pool).await?))
}

/// Update module-owned navigation presentation.
/// Body: name, icon, sort_order, status.
pub async fn update_menu(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuPayload>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(MenuService::update_menu(&pool, id, request).await?))
}

/// Disable menu
pub async fn delete_menu(
    current_user: CurrentUser,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> AppResult<()> {
    MenuService::delete_menu(&pool, id, current_user.user_id).await?;
    Ok(ApiResponse::success(()))
}

/// Get menu options for dropdowns
pub async fn get_menu_options(
    State(pool): State<SqlitePool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<MenuOptionResp>> {
    Ok(ApiResponse::success(MenuService::get_menu_options(&pool, query).await?))
}
