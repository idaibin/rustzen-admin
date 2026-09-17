use axum::Extension;

use crate::common::api::{ApiResponse, AppResult};
use crate::features::modules::registry::ModuleRegistry;

use super::{service::SystemStatusService, types::SystemStatusOverview};

pub async fn get_status_overview(
    Extension(registry): Extension<ModuleRegistry>,
) -> AppResult<SystemStatusOverview> {
    Ok(ApiResponse::success(SystemStatusService::overview(&registry).await?))
}
