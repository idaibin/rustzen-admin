mod handler;
mod repo;
mod service;
mod types;

use rustzen_ipc::{ManifestError, ModuleRouter};

use crate::app::AppState;

pub fn register(router: ModuleRouter<AppState>) -> Result<ModuleRouter<AppState>, ManifestError> {
    router
        .get_public("/tracker.js", handler::tracker)?
        .options_public("/track", handler::preflight)?
        .post_public("/track", handler::track)
}

#[cfg(test)]
pub(crate) use service::StorageCapacityChecker;
pub(crate) use service::{IngestionState, spawn_retention};
