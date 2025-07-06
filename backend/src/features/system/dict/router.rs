use super::dto::{CreateDictDto, DictQueryDto, UpdateDictDto};
use super::service::DictService;
use super::vo::DictDetailVo;
use crate::common::{
    api::{ApiResponse, AppResult, DictOptionsQuery},
    router_ext::RouterExt,
};
use crate::features::auth::permission::PermissionsCheck;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, patch, post, put},
};
use sqlx::PgPool;

/// Defines the routes for dictionary item management operations.
///
/// This router provides full CRUD access to system dictionary data which is typically used for:
/// - Dropdown/select options in forms
/// - Standardized values across the application
/// - Configuration data that rarely changes
///
/// All routes require appropriate permissions to access dictionary data.
///
/// # Routes
/// - `GET /` - List all dictionary items (requires: system:dict:list)
/// - `GET /{id}` - Get dictionary item by ID (requires: system:dict:get)
/// - `POST /` - Create new dictionary item (requires: system:dict:create)
/// - `PUT /{id}` - Update dictionary item (requires: system:dict:update)
/// - `DELETE /{id}` - Delete dictionary item (requires: system:dict:delete)
/// - `PATCH /{id}/status` - Update dictionary item status (requires: system:dict:update)
/// - `GET /options` - Get dictionary options for dropdowns (requires: system:dict:options)
/// - `GET /types` - Get all dictionary types (requires: system:dict:list)
/// - `GET /type/{type}` - Get dictionary items by type (requires: system:dict:list)
pub fn dict_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_dict_list),
            PermissionsCheck::Single("system:dict:list"),
        )
        .route_with_permission(
            "/",
            post(create_dict),
            PermissionsCheck::Single("system:dict:create"),
        )
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Single("system:dict:options"),
        )
        .route_with_permission(
            "/types",
            get(get_dict_types),
            PermissionsCheck::Single("system:dict:list"),
        )
        .route_with_permission(
            "/type/{type}",
            get(get_dict_by_type),
            PermissionsCheck::Single("system:dict:list"),
        )
        .route_with_permission(
            "/{id}",
            get(get_dict_by_id),
            PermissionsCheck::Single("system:dict:get"),
        )
        .route_with_permission(
            "/{id}",
            put(update_dict),
            PermissionsCheck::Single("system:dict:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_dict),
            PermissionsCheck::Single("system:dict:delete"),
        )
        .route_with_permission(
            "/{id}/status",
            patch(update_dict_status),
            PermissionsCheck::Single("system:dict:update"),
        )
}

/// Retrieves a complete list of dictionary items with optional filtering.
///
/// This endpoint returns dictionary items in the system, with support for filtering
/// by type and search terms.
///
/// # Query Parameters
/// - `dict_type`: Filter by dictionary type (optional)
/// - `q`: Search term to filter by label or value (optional)
/// - `limit`: Maximum number of results (optional)
///
/// # Response
/// - Array of dictionary items with their keys, values, and metadata
async fn get_dict_list(
    State(pool): State<PgPool>,
    Query(params): Query<DictQueryDto>,
) -> AppResult<Json<ApiResponse<Vec<DictDetailVo>>>> {
    tracing::info!("Dictionary list request received with params: {:?}", params);

    let query_params = if params.dict_type.is_some() || params.q.is_some() || params.limit.is_some()
    {
        Some(params)
    } else {
        None
    };

    let (dict_list, total) = DictService::get_dict_list(&pool, query_params).await?;

    tracing::info!("Dictionary list retrieved successfully: count={}", dict_list.len());

    Ok(ApiResponse::page(dict_list, total))
}

/// Retrieves a single dictionary item by ID.
///
/// # Path Parameters
/// - `id`: Dictionary item ID
///
/// # Response
/// - Dictionary item details
async fn get_dict_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<DictDetailVo>>> {
    tracing::info!("Get dictionary item by ID: {}", id);

    let dict = DictService::get_dict_by_id(&pool, id).await?;

    tracing::info!("Dictionary item retrieved: id={}, type={}", dict.id, dict.dict_type);

    Ok(ApiResponse::success(dict))
}

/// Creates a new dictionary item.
///
/// # Request Body
/// - `dict_type`: Dictionary type/category
/// - `label`: Display label
/// - `value`: Actual value
/// - `is_default`: Whether this is the default item for the type (optional)
///
/// # Response
/// - Created dictionary item details
async fn create_dict(
    State(pool): State<PgPool>,
    Json(request): Json<CreateDictDto>,
) -> AppResult<Json<ApiResponse<DictDetailVo>>> {
    tracing::info!("Create dictionary item: type={}, label={}", request.dict_type, request.label);

    let new_dict = DictService::create_dict(&pool, request).await?;

    tracing::info!("Dictionary item created: id={}, type={}", new_dict.id, new_dict.dict_type);

    Ok(ApiResponse::success(new_dict))
}

/// Updates an existing dictionary item.
///
/// # Path Parameters
/// - `id`: Dictionary item ID to update
///
/// # Request Body
/// - `dict_type`: New dictionary type/category (optional)
/// - `label`: New display label (optional)
/// - `value`: New actual value (optional)
/// - `is_default`: New default flag (optional)
///
/// # Response
/// - Updated dictionary item details
async fn update_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictDto>,
) -> AppResult<Json<ApiResponse<DictDetailVo>>> {
    tracing::info!("Update dictionary item {}: {:?}", id, request);

    let updated_dict = DictService::update_dict(&pool, id, request).await?;

    tracing::info!(
        "Dictionary item updated: id={}, type={}",
        updated_dict.id,
        updated_dict.dict_type
    );

    Ok(ApiResponse::success(updated_dict))
}

/// Deletes a dictionary item by ID.
///
/// # Path Parameters
/// - `id`: Dictionary item ID to delete
///
/// # Response
/// - Success confirmation
async fn delete_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Delete dictionary item: {}", id);

    DictService::delete_dict(&pool, id).await?;

    tracing::info!("Dictionary item deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Updates the status of a dictionary item.
///
/// # Path Parameters
/// - `id`: Dictionary item ID
///
/// # Request Body
/// - `status`: New status (1=active, 2=inactive)
///
/// # Response
/// - Success confirmation
#[derive(serde::Deserialize)]
struct UpdateStatusRequest {
    status: i16,
}

async fn update_dict_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateStatusRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Update dictionary item {} status to: {}", id, request.status);

    DictService::update_dict_status(&pool, id, request.status).await?;

    tracing::info!("Dictionary item {} status updated to {}", id, request.status);

    Ok(ApiResponse::success(()))
}

/// Retrieves dictionary options for dropdown/select components.
///
/// This endpoint provides filtered dictionary data specifically formatted
/// for use in UI components like dropdowns, select boxes, and option lists.
///
/// # Query Parameters
/// - `dict_type`: Type/category of dictionary items to retrieve (optional)
/// - `q`: Search term to filter dictionary items by label or value (optional)
/// - `limit`: Maximum number of options to return (optional)
///
/// # Response
/// - Array of option objects with `label` and `value` fields suitable for UI components
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

/// Retrieves all available dictionary types.
///
/// # Response
/// - Array of unique dictionary type strings
async fn get_dict_types(State(pool): State<PgPool>) -> AppResult<Json<ApiResponse<Vec<String>>>> {
    tracing::info!("Dictionary types request received");

    let types = DictService::get_dict_types(&pool).await?;

    tracing::info!("Dictionary types retrieved successfully: count={}", types.len());

    Ok(ApiResponse::success(types))
}

/// Retrieves dictionary items by type.
///
/// # Path Parameters
/// - `type`: Dictionary type to filter by
///
/// # Response
/// - Array of dictionary items for the specified type
async fn get_dict_by_type(
    State(pool): State<PgPool>,
    Path(dict_type): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<DictDetailVo>>>> {
    tracing::info!("Dictionary items by type request: {}", dict_type);

    let dicts = DictService::get_dict_by_type(&pool, &dict_type).await?;

    tracing::info!("Dictionary items by type retrieved: type={}, count={}", dict_type, dicts.len());

    Ok(ApiResponse::success(dicts))
}
