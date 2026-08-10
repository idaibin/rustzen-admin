use axum::{Json, extract::State};

use crate::{
    app::AppState,
    common::api::{ApiResponse, AppResult},
};

use super::{
    service,
    types::{CollectionPolicy, CollectionPolicyUpdate},
};

pub async fn collection_policy(State(state): State<AppState>) -> AppResult<CollectionPolicy> {
    Ok(ApiResponse::success(service::get_collection_policy(&state.pool).await?))
}

pub async fn update_collection_policy(
    State(state): State<AppState>,
    Json(update): Json<CollectionPolicyUpdate>,
) -> AppResult<CollectionPolicy> {
    Ok(ApiResponse::success(
        service::update_collection_policy(&state.pool, &state.ingestion, update).await?,
    ))
}
