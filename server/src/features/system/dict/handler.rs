use super::{
    service::DictService,
    types::{CreateDictRequest, DictItemResp, DictQuery, UpdateDictPayload, UpdateDictStatusPayload},
};
use crate::common::{
    api::{ApiResponse, AppResult, DictOptionsQuery, OptionItem},
};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;

/// Retrieves a complete list of dictionary items with optional filtering.
pub async fn list_dicts(
    State(pool): State<PgPool>,
    Query(query): Query<DictQuery>,
) -> AppResult<Vec<DictItemResp>> {
    tracing::info!("Dictionary list request received with params: {:?}", query);

    let (dict_list, total) = DictService::list_dicts(&pool, query).await?;

    tracing::info!("Dictionary list retrieved successfully: count={}", dict_list.len());

    Ok(ApiResponse::page(dict_list, total))
}

/// Creates a new dictionary item.
pub async fn create_dict(
    State(pool): State<PgPool>,
    Json(request): Json<CreateDictRequest>,
) -> AppResult<i64> {
    tracing::info!("Create dictionary item: type={}, label={}", request.dict_type, request.label);

    let dict_id = DictService::create_dict(&pool, request).await?;

    tracing::info!("Dictionary item created: id={}", dict_id);

    Ok(ApiResponse::success(dict_id))
}

/// Updates an existing dictionary item.
pub async fn update_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictPayload>,
) -> AppResult<i64> {
    tracing::info!("Update dictionary item {}: {:?}", id, request);

    let dict_id = DictService::update_dict(&pool, id, request).await?;

    tracing::info!("Dictionary item updated: id={}", dict_id);

    Ok(ApiResponse::success(dict_id))
}

/// Deletes a dictionary item by ID.
pub async fn delete_dict(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    tracing::info!("Delete dictionary item: {}", id);

    DictService::delete_dict(&pool, id).await?;

    tracing::info!("Dictionary item deleted: {}", id);

    Ok(ApiResponse::success(()))
}

pub async fn update_dict_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictStatusPayload>,
) -> AppResult<()> {
    tracing::info!("Update dictionary item {} status to: {}", id, request.status);

    DictService::update_dict_status(&pool, id, request.status).await?;

    tracing::info!("Dictionary item {} status updated to {}", id, request.status);

    Ok(ApiResponse::success(()))
}

/// Retrieves dictionary options for dropdown/select components.
pub async fn get_dict_options(
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
pub async fn get_dict_by_type(
    State(pool): State<PgPool>,
    Path(dict_type): Path<String>,
) -> AppResult<Vec<OptionItem<String>>> {
    tracing::info!("Dictionary items by type request: {}", dict_type);

    let dicts = DictService::get_dict_by_type(&pool, &dict_type).await?;

    tracing::info!("Dictionary items by type retrieved: type={}, count={}", dict_type, dicts.len());

    Ok(ApiResponse::success(dicts))
}
