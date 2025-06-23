use super::model::{
    CreateMenuRequest, MenuListResponse, MenuQueryParams, MenuResponse, UpdateMenuRequest,
};
use super::service::MenuService;
use crate::common::api::{ApiResponse, AppResult, OptionsQuery};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// Defines the routes for menu management.
pub fn menu_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_menu_list))
        .route("/", post(create_menu))
        .route("/options", get(get_menu_options))
        .route("/{id}", get(get_menu_by_id))
        .route("/{id}", put(update_menu))
        .route("/{id}", delete(delete_menu))
}

/// Handles the request to get a list of menus as a tree.
async fn get_menu_list(
    State(pool): State<PgPool>,
    Query(params): Query<MenuQueryParams>,
) -> AppResult<Json<ApiResponse<MenuListResponse>>> {
    let response = MenuService::get_menu_list(&pool, params).await?;
    Ok(ApiResponse::success(response))
}

/// Handles the request to get a single menu by its ID.
async fn get_menu_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::get_menu_by_id(&pool, id).await?;
    Ok(ApiResponse::success(response))
}

/// Handles the request to create a new menu.
async fn create_menu(
    State(pool): State<PgPool>,
    Json(request): Json<CreateMenuRequest>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::create_menu(&pool, request).await?;
    Ok(ApiResponse::success(response))
}

/// Handles the request to update an existing menu.
async fn update_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuRequest>,
) -> AppResult<Json<ApiResponse<MenuResponse>>> {
    let response = MenuService::update_menu(&pool, id, request).await?;
    Ok(ApiResponse::success(response))
}

/// Handles the request to delete a menu.
async fn delete_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    MenuService::delete_menu(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Handles the request to get menu options for dropdowns
///
/// Extracts query parameters and delegates to the service layer for processing.
async fn get_menu_options(
    State(pool): State<PgPool>,
    query: Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::common::api::OptionItem<i64>>>>> {
    let options = MenuService::get_menu_options(&pool, query).await?;
    Ok(ApiResponse::success(options))
}
