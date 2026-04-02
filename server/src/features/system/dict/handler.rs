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
    let (dict_list, total) = DictService::list_dicts(&pool, query).await?;
    Ok(ApiResponse::page(dict_list, total))
}

/// Creates a new dictionary item.
pub async fn create_dict(
    State(pool): State<PgPool>,
    Json(request): Json<CreateDictRequest>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(DictService::create_dict(&pool, request).await?))
}

/// Updates an existing dictionary item.
pub async fn update_dict(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictPayload>,
) -> AppResult<i64> {
    Ok(ApiResponse::success(DictService::update_dict(&pool, id, request).await?))
}

/// Deletes a dictionary item by ID.
pub async fn delete_dict(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    DictService::delete_dict(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn update_dict_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateDictStatusPayload>,
) -> AppResult<()> {
    DictService::update_dict_status(&pool, id, request.status).await?;
    Ok(ApiResponse::success(()))
}

/// Retrieves dictionary options for dropdown/select components.
pub async fn get_dict_options(
    State(pool): State<PgPool>,
    Query(query): Query<DictOptionsQuery>,
) -> AppResult<Vec<OptionItem<String>>> {
    Ok(ApiResponse::success(
        DictService::get_dict_options(&pool, query.dict_type, query.q, query.limit).await?,
    ))
}

/// Retrieves dictionary items by type.
pub async fn get_dict_by_type(
    State(pool): State<PgPool>,
    Path(dict_type): Path<String>,
) -> AppResult<Vec<OptionItem<String>>> {
    Ok(ApiResponse::success(DictService::get_dict_by_type(&pool, &dict_type).await?))
}
