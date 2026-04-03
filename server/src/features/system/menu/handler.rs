use super::{
    service::MenuService,
    types::{CreateMenuRequest, MenuItemResp, MenuQuery, UpdateMenuPayload},
};
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;

/// Get menu list with optional filtering
/// Query params: title, status
/// Need show all menu, not pagination
pub async fn list_menus(
    State(pool): State<PgPool>,
    Query(params): Query<MenuQuery>,
) -> AppResult<Vec<MenuItemResp>> {
    let (menu_list, total) = MenuService::list_menus(&pool, params).await?;
    Ok(ApiResponse::page(menu_list, total))
}

/// Create new menu
/// Body: name, path, parent_id, icon, sort_order, status
pub async fn create_menu(
    State(pool): State<PgPool>,
    Json(request): Json<CreateMenuRequest>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(MenuService::create_menu(&pool, request).await?))
}

/// Update menu
/// Body: name, path, parent_id, icon, sort_order, status (all optional)
pub async fn update_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuPayload>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(MenuService::update_menu(&pool, id, request).await?))
}

/// Disable menu
pub async fn delete_menu(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    MenuService::delete_menu(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Get menu options for dropdowns
pub async fn get_menu_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<crate::common::api::OptionItem<i64>>> {
    Ok(ApiResponse::success(MenuService::get_menu_options(&pool, query).await?))
}
