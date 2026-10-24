use axum::extract::State;
#[cfg(feature = "core")]
use axum::{Extension, Json, extract::Path};
#[cfg(feature = "core")]
use rustzen_auth::auth::AuthClaims;
use rustzen_auth::auth::CurrentUser;

#[cfg(feature = "core")]
use super::types::{ModuleHealthResponse, ModuleStatusResponse, UpdateModuleRequest};
use super::{
    service::{ModuleControlState, ModuleService},
    types::RuntimeMenuResponse,
};
use crate::common::api::{ApiResponse, AppResult};

#[cfg(feature = "core")]
pub async fn list(State(state): State<ModuleControlState>) -> AppResult<Vec<ModuleStatusResponse>> {
    Ok(ApiResponse::success(ModuleService::statuses(&state)))
}

pub async fn navigation(
    user: CurrentUser,
    State(state): State<ModuleControlState>,
) -> AppResult<Vec<RuntimeMenuResponse>> {
    Ok(ApiResponse::success(ModuleService::navigation(&state, &user).await?))
}

#[cfg(feature = "core")]
pub async fn dashboard(
    State(state): State<ModuleControlState>,
) -> AppResult<Vec<ModuleHealthResponse>> {
    Ok(ApiResponse::success(ModuleService::dashboard_health(&state).await?))
}

#[cfg(feature = "core")]
pub async fn update(
    Extension(actor): Extension<AuthClaims>,
    State(state): State<ModuleControlState>,
    Path(module): Path<String>,
    Json(request): Json<UpdateModuleRequest>,
) -> AppResult<Vec<ModuleStatusResponse>> {
    Ok(ApiResponse::success(
        ModuleService::set_enabled_authorized(&state, &module, request.enabled, &actor).await?,
    ))
}
