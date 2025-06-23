use super::model::DictItem;
use super::service::DictService;
use crate::common::api::{ApiResponse, AppResult, DictOptionsQuery};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use sqlx::PgPool;

/// Defines the routes for dictionary items.
pub fn dict_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_dict_list)).route("/options", get(get_dict_options))
}

/// Handles the request to get a list of dictionary items.
async fn get_dict_list(State(pool): State<PgPool>) -> AppResult<Json<ApiResponse<Vec<DictItem>>>> {
    let dict_list = DictService::get_dict_list(&pool).await?;
    Ok(ApiResponse::success(dict_list))
}

/// Handles the request to get dictionary options for dropdowns
///
/// Extracts query parameters and delegates to the service layer for processing.
async fn get_dict_options(
    State(pool): State<PgPool>,
    Query(query): Query<DictOptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::common::api::OptionItem<String>>>>> {
    let options =
        DictService::get_dict_options(&pool, query.dict_type, query.q, query.limit).await?;
    Ok(ApiResponse::success(options))
}
