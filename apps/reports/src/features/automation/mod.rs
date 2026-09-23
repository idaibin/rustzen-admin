mod browser;
#[cfg(feature = "notifications")]
mod delivery;
mod flows;
mod handler;
pub(crate) mod repo;
mod retry;
mod scheduler;
mod service;
mod systems;
mod types;
#[cfg(test)]
mod types_tests;
mod validation;

use rustzen_auth::capability::reports;
use rustzen_ipc::{ManifestError, ModuleRouter, Require};

use crate::app::AppState;

pub use scheduler::{initialize, spawn};

pub fn routes(router: ModuleRouter<AppState>) -> Result<ModuleRouter<AppState>, ManifestError> {
    let router = router
        .get_with_permission("/systems", handler::systems, Require(reports::SYSTEM_VIEW))?
        .post_with_permission("/systems", handler::create_system, Require(reports::SYSTEM_MANAGE))?
        .put_with_permission(
            "/systems/{id}",
            handler::update_system,
            Require(reports::SYSTEM_MANAGE),
        )?
        .delete_with_permission(
            "/systems/{id}",
            handler::delete_system,
            Require(reports::SYSTEM_MANAGE),
        )?
        .get_with_permission("/flows", handler::flows, Require(reports::FLOW_VIEW))?
        .post_with_permission("/flows", handler::create_flow, Require(reports::FLOW_MANAGE))?
        .put_with_permission("/flows/{id}", handler::update_flow, Require(reports::FLOW_MANAGE))?
        .delete_with_permission("/flows/{id}", handler::delete_flow, Require(reports::FLOW_MANAGE))?
        .get_with_permission("/schedules", handler::schedules, Require(reports::SCHEDULE_VIEW))?
        .get_with_permission(
            "/flow-options",
            handler::flow_options,
            Require(reports::SCHEDULE_VIEW),
        )?
        .post_with_permission(
            "/schedules",
            handler::create_schedule,
            Require(reports::SCHEDULE_MANAGE),
        )?
        .get_with_permission("/schedules/{id}", handler::schedule, Require(reports::SCHEDULE_VIEW))?
        .put_with_permission(
            "/schedules/{id}",
            handler::update_schedule,
            Require(reports::SCHEDULE_MANAGE),
        )?
        .delete_with_permission(
            "/schedules/{id}",
            handler::delete_schedule,
            Require(reports::SCHEDULE_MANAGE),
        )?
        .get_with_permission("/settings", handler::settings, Require(reports::SCHEDULE_VIEW))?
        .get_with_permission("/runs", handler::runs, Require(reports::RUN_VIEW))?
        .post_with_permission("/runs", handler::create_run, Require(reports::RUN_MANAGE))?
        .get_with_permission("/runs/{id}", handler::run, Require(reports::RUN_VIEW))?
        .post_with_permission(
            "/runs/{id}/cancel",
            handler::cancel_run,
            Require(reports::RUN_MANAGE),
        )?
        .post_with_permission("/runs/{id}/retry", handler::retry_run, Require(reports::RUN_MANAGE))?
        .get_with_permission("/runs/{id}/steps", handler::run_steps, Require(reports::RUN_VIEW))?
        .get_with_permission(
            "/runs/{id}/artifacts",
            handler::run_artifacts,
            Require(reports::RUN_VIEW),
        )?
        .get_with_permission(
            "/runs/{run_id}/artifacts/{artifact_id}",
            handler::artifact,
            Require(reports::RUN_VIEW),
        )?
        .get_with_permission(
            "/runs/{id}/live-frame",
            handler::live_frame,
            Require(reports::RUN_VIEW),
        )?;
    #[cfg(feature = "notifications")]
    let router = router.get_with_permission(
        "/notification-delivery",
        delivery::status,
        Require(reports::RUN_VIEW),
    )?;
    Ok(router)
}
