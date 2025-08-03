use super::{
    dto::{CreateDictDto, DictQueryDto, UpdateDictDto, UpdateDictStatusDto},
    service::DictService,
    vo::DictItemVo,
};
use crate::{
    common::{
        api::{ApiResponse, AppResult, DictOptionsQuery, OptionItem},
        router_ext::RouterExt,
    },
    core::permission::PermissionsCheck,
};

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
        .route_with_permission(
            "/options",
            get(get_dict_options),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:options"]),
        )
        .route_with_permission(
            "/type/{type}",
            get(get_dict_by_type),
            PermissionsCheck::Any(vec!["system:*", "system:dict:*", "system:dict:options"]),
        )
}

/// Retrieves a complete list of dictionary items with optional filtering.
async fn get_dict_list(
    State(pool): State<PgPool>,
    Query(query): Query<DictQueryDto>,
) -> AppResult<Vec<DictItemVo>> {
    tracing::info!("Dictionary list request received with params: {:?}", query);

    let (dict_list, total) = DictService::get_dict_list(&pool, query).await?;

    tracing::info!("Dictionary list retrieved successfully: count={}", dict_list.len());

    Ok(ApiResponse::page(dict_list, total))
}

/// Creates a new dictionary item.
async fn create_dict(
    State(pool): State<PgPool>,
    Json(request): Json<CreateDictDto>,
) -> AppResult<i64> {
    tracing::info!("Create dictionary item: type={}, label={}", request.dict_type, request.label);

    let dict_id = DictService::create_dict(&pool, request).await?;

    tracing::info!("Dictionary item created: id={}", dict_id);

    Ok(ApiResponse::success(dict_id))
}

/// Updates an existing dictionary item.
async fn update_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictDto>,
) -> AppResult<i64> {
    tracing::info!("Update dictionary item {}: {:?}", id, request);

    let dict_id = DictService::update_dict(&pool, id, request).await?;

    tracing::info!("Dictionary item updated: id={}", dict_id);

    Ok(ApiResponse::success(dict_id))
}

/// Deletes a dictionary item by ID.
async fn delete_dict(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Delete dictionary item: {}", id);

    DictService::delete_dict(&pool, id).await?;

    tracing::info!("Dictionary item deleted: {}", id);

    Ok(ApiResponse::success(()))
}

async fn update_dict_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictStatusDto>,
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
) -> AppResult<Vec<OptionItem<String>>> {
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

/// Retrieves dictionary items by type.
async fn get_dict_by_type(
    State(pool): State<PgPool>,
    Path(dict_type): Path<String>,
) -> AppResult<Vec<OptionItem<String>>> {
    tracing::info!("Dictionary items by type request: {}", dict_type);

    let dicts = DictService::get_dict_by_type(&pool, &dict_type).await?;

    tracing::info!("Dictionary items by type retrieved: type={}, count={}", dict_type, dicts.len());

    Ok(ApiResponse::success(dicts))
}
