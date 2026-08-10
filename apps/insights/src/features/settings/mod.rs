mod handler;
mod repo;
pub(crate) mod service;
mod types;

use rustzen_auth::capability::insights;
use rustzen_ipc::{ManifestError, ModuleRouter, Require};

use crate::app::AppState;

pub fn register(router: ModuleRouter<AppState>) -> Result<ModuleRouter<AppState>, ManifestError> {
    router
        .get_with_permission(
            "/collection-policy",
            handler::collection_policy,
            Require(insights::MANAGE),
        )?
        .put_with_permission(
            "/collection-policy",
            handler::update_collection_policy,
            Require(insights::MANAGE),
        )
}
