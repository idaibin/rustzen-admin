mod handler;
mod repo;
pub(crate) mod service;
pub(crate) mod types;

use rustzen_auth::capability::monitor;
use rustzen_ipc::{ManifestError, ModuleRouter, Require};

use crate::app::AppState;

pub use handler::spawn_evaluator;

pub(crate) fn routes(
    router: ModuleRouter<AppState>,
) -> Result<ModuleRouter<AppState>, ManifestError> {
    router
        .get_with_permission("/incidents", handler::list, Require(monitor::INCIDENT_VIEW))?
        .get_with_permission("/incidents/{id}", handler::get, Require(monitor::INCIDENT_VIEW))
}
