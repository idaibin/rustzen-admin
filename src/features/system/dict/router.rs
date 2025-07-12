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
pub fn dict_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_dict_list),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:list"]),
        )
        .route_with_permission(
            "/",
            post(create_dict),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:create"]),
        )
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:options"]),
        )
        .route_with_permission(
            "/types",
            get(get_dict_types),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:list"]),
        )
        .route_with_permission(
            "/type/{type}",
            get(get_dict_by_type),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:list"]),
        )
        .route_with_permission(
            "/{id}",
            get(get_dict_by_id),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:get"]),
        )
        .route_with_permission(
            "/{id}",
            put(update_dict),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:update"]),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_dict),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:delete"]),
        )
        .route_with_permission(
            "/{id}/status",
            patch(update_dict_status),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:update"]),
        )
}

/// Retrieves a complete list of dictionary items with optional filtering.
async fn get_dict_list(
    State(pool): State<PgPool>,
    Query(query): Query<DictQueryDto>,
) -> AppResult<Vec<DictDetailVo>> {
    tracing::info!("Dictionary list request received with params: {:?}", query);

    let (dict_list, total) = DictService::get_dict_list(&pool, query).await?;

    tracing::info!("Dictionary list retrieved successfully: count={}", dict_list.len());

    Ok(ApiResponse::page(dict_list, total))
}

/// Retrieves a single dictionary item by ID.
async fn get_dict_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<DictDetailVo> {
    tracing::info!("Get dictionary item by ID: {}", id);

    let dict = DictService::get_dict_by_id(&pool, id).await?;

    tracing::info!("Dictionary item retrieved: id={}, type={}", dict.id, dict.dict_type);

    Ok(ApiResponse::success(dict))
}

/// Creates a new dictionary item.
async fn create_dict(
    State(pool): State<PgPool>,
    Json(request): Json<CreateDictDto>,
) -> AppResult<DictDetailVo> {
    tracing::info!("Create dictionary item: type={}, label={}", request.dict_type, request.label);

    let new_dict = DictService::create_dict(&pool, request).await?;

    tracing::info!("Dictionary item created: id={}, type={}", new_dict.id, new_dict.dict_type);

    Ok(ApiResponse::success(new_dict))
}

/// Updates an existing dictionary item.
async fn update_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictDto>,
) -> AppResult<DictDetailVo> {
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
async fn delete_dict(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Delete dictionary item: {}", id);

    DictService::delete_dict(&pool, id).await?;

    tracing::info!("Dictionary item deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Updates the status of a dictionary item.
#[derive(serde::Deserialize)]
struct UpdateStatusRequest {
    status: i16,
}

async fn update_dict_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateStatusRequest>,
) -> AppResult<()> {
    tracing::info!("Update dictionary item {} status to: {}", id, request.status);

    DictService::update_dict_status(&pool, id, request.status).await?;

    tracing::info!("Dictionary item {} status updated to {}", id, request.status);

    Ok(ApiResponse::success(()))
}

/// Retrieves dictionary options for dropdown/select components.
async fn get_dict_options(
    State(pool): State<PgPool>,
    Query(query): Query<DictOptionsQuery>,
) -> AppResult<Vec<crate::common::api::OptionItem<String>>> {
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
async fn get_dict_types(State(pool): State<PgPool>) -> AppResult<Vec<String>> {
    tracing::info!("Dictionary types request received");

    let types = DictService::get_dict_types(&pool).await?;

    tracing::info!("Dictionary types retrieved successfully: count={}", types.len());

    Ok(ApiResponse::success(types))
}

/// Retrieves dictionary items by type.
async fn get_dict_by_type(
    State(pool): State<PgPool>,
    Path(dict_type): Path<String>,
) -> AppResult<Vec<DictDetailVo>> {
    tracing::info!("Dictionary items by type request: {}", dict_type);

    let dicts = DictService::get_dict_by_type(&pool, &dict_type).await?;

    tracing::info!("Dictionary items by type retrieved: type={}, count={}", dict_type, dicts.len());

    Ok(ApiResponse::success(dicts))
}
