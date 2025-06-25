use super::model::DictItem;
use super::service::DictService;
use crate::common::{
    api::{ApiResponse, AppResult, DictOptionsQuery},
    router_ext::RouterExt,
};
use crate::features::auth::permission::PermissionsCheck;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use sqlx::PgPool;

/// Defines the routes for dictionary item management operations.
///
/// This router provides access to system dictionary data which is typically used for:
/// - Dropdown/select options in forms
/// - Standardized values across the application
/// - Configuration data that rarely changes
///
/// All routes require appropriate permissions to access dictionary data.
///
/// # Routes
/// - `GET /` - List all dictionary items (requires: system:dict:list)
/// - `GET /options` - Get dictionary options for dropdowns (requires: system:dict:options)
pub fn dict_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_dict_list),
            PermissionsCheck::Single("system:dict:list"),
        )
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Single("system:dict:options"),
        )
}

/// Retrieves a complete list of dictionary items.
///
/// This endpoint returns all dictionary items in the system, typically used
/// for administrative purposes where a complete view of all dictionary data is needed.
/// Dictionary items usually contain key-value pairs that are used throughout
/// the application for standardized values.
///
/// # Response
/// - Array of dictionary items with their keys, values, and metadata
async fn get_dict_list(State(pool): State<PgPool>) -> AppResult<Json<ApiResponse<Vec<DictItem>>>> {
    tracing::info!("Dictionary list request received");

    let dict_list = DictService::get_dict_list(&pool).await?;

    tracing::info!("Dictionary list retrieved successfully: count={}", dict_list.len());

    Ok(ApiResponse::success(dict_list))
}

/// Retrieves dictionary options for dropdown/select components.
///
/// This endpoint provides filtered dictionary data specifically formatted
/// for use in UI components like dropdowns, select boxes, and option lists.
/// It supports filtering and limiting results for better performance and usability.
///
/// # Query Parameters
/// - `dict_type`: Type/category of dictionary items to retrieve (optional)
/// - `q`: Search term to filter dictionary items by label or value (optional)
/// - `limit`: Maximum number of options to return (optional)
///
/// # Response
/// - Array of option objects with `label` and `value` fields suitable for UI components
///
/// # Usage
/// This endpoint is commonly used to populate form controls with predefined options,
/// such as status dropdowns, category selectors, or other standardized value lists.
async fn get_dict_options(
    State(pool): State<PgPool>,
    Query(query): Query<DictOptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::common::api::OptionItem<String>>>>> {
    tracing::debug!(
        "Dictionary options request: dict_type={:?}, q={:?}, limit={:?}",
        query.dict_type,
        query.q,
        query.limit
    );

    let options =
        DictService::get_dict_options(&pool, query.dict_type, query.q, query.limit).await?;

    tracing::debug!("Dictionary options retrieved successfully: count={}", options.len());

    Ok(ApiResponse::success(options))
}
