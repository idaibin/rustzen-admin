use axum::extract::State;
#[cfg(feature = "full")]
use axum::{Json, extract::Path};
use rustzen_auth::auth::CurrentUser;

#[cfg(feature = "full")]
use super::types::{ModuleHealthResponse, ModuleStatusResponse, UpdateModuleRequest};
use super::{
    service::{ModuleControlState, ModuleService},
    types::RuntimeMenuResponse,
};
use crate::common::api::{ApiResponse, AppResult};

#[cfg(feature = "full")]
pub async fn list(State(state): State<ModuleControlState>) -> AppResult<Vec<ModuleStatusResponse>> {
    Ok(ApiResponse::success(ModuleService::statuses(&state)))
}

pub async fn navigation(
    user: CurrentUser,
    State(state): State<ModuleControlState>,
) -> AppResult<Vec<RuntimeMenuResponse>> {
    Ok(ApiResponse::success(ModuleService::navigation(&state, &user).await?))
}

#[cfg(feature = "full")]
pub async fn dashboard(
    State(state): State<ModuleControlState>,
) -> AppResult<Vec<ModuleHealthResponse>> {
    Ok(ApiResponse::success(ModuleService::dashboard_health(&state)))
}

#[cfg(feature = "full")]
pub async fn update(
    State(state): State<ModuleControlState>,
    Path(module): Path<String>,
    Json(request): Json<UpdateModuleRequest>,
) -> AppResult<Vec<ModuleStatusResponse>> {
    Ok(ApiResponse::success(ModuleService::set_enabled(&state, &module, request.enabled).await?))
}
