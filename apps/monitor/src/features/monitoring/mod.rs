//! Monitor Controller endpoints are organized by the state they own.
//!
//! `acceptance` serializes incoming agent reports, `queries` serves read APIs,
//! `policy` owns alert policy mutations and threshold evaluation, and
//! `background` owns periodic summaries, liveness scans, and retention.

mod acceptance;
mod background;
mod policy;
mod queries;

#[cfg(test)]
mod tests;

use rustzen_ipc::{ModuleRouter, Require};

use crate::app::AppState;

pub(crate) use background::spawn_background;

#[cfg(test)]
pub(crate) use acceptance::{record_at, submit};

pub fn routes(
    router: ModuleRouter<AppState>,
) -> Result<ModuleRouter<AppState>, rustzen_ipc::ManifestError> {
    router
        .post_public("/agent-reports", acceptance::submit)?
        .get_with_permission(
            "/overview",
            queries::overview,
            Require(rustzen_auth::capability::monitor::OVERVIEW_VIEW),
        )?
        .get_with_permission(
            "/nodes",
            queries::nodes,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .get_with_permission(
            "/nodes/{node_id}",
            queries::node,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .get_with_permission(
            "/nodes/{node_id}/metrics",
            queries::metrics,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .get_with_permission(
            "/nodes/{node_id}/alert-settings",
            policy::node_settings,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .put_with_permission(
            "/nodes/{node_id}/alert-settings",
            policy::update_node_settings,
            Require(rustzen_auth::capability::monitor::MANAGE),
        )?
        .delete_with_permission(
            "/nodes/{node_id}/alert-settings",
            policy::reset_node_settings,
            Require(rustzen_auth::capability::monitor::MANAGE),
        )?
        .get_with_permission(
            "/incidents",
            queries::incidents,
            Require(rustzen_auth::capability::monitor::INCIDENT_VIEW),
        )?
        .get_with_permission(
            "/incidents/{id}",
            queries::incident,
            Require(rustzen_auth::capability::monitor::INCIDENT_VIEW),
        )?
        .get_with_permission(
            "/alert-settings",
            policy::settings,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .put_with_permission(
            "/alert-settings",
            policy::update_settings,
            Require(rustzen_auth::capability::monitor::MANAGE),
        )?
        .get_with_permission(
            "/daily-summaries",
            background::summaries,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )
}
