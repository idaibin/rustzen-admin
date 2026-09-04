pub mod gateway;
pub mod handler;
pub mod registry;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::get;
#[cfg(feature = "full")]
use axum::routing::put;
#[cfg(feature = "full")]
use rustzen_auth::capability::{dashboard, system_module};

#[cfg(feature = "full")]
use self::handler::{dashboard, list, update};
use self::{handler::navigation, service::ModuleControlState};

pub fn control_routes() -> ContractRouter<ModuleControlState> {
    let router = ContractRouter::new()
        .get(
            "/api/system/modules/navigation",
            OperationDescriptor::GetModuleNavigation,
            AccessPolicy::Authenticated,
            get(navigation),
        )
        .expect("static module control contract");
    #[cfg(feature = "full")]
    let router = router
        .get(
            "/api/system/modules",
            OperationDescriptor::ListModules,
            AccessPolicy::Require(system_module::LIST),
            get(list),
        )
        .expect("static module control contract")
        .put(
            "/api/system/modules/{module}/enabled",
            OperationDescriptor::UpdateModuleEnabled,
            AccessPolicy::Require(system_module::UPDATE),
            put(update),
        )
        .expect("static module control contract")
        .get(
            "/api/dashboard/modules",
            OperationDescriptor::GetDashboardModules,
            AccessPolicy::Require(dashboard::VIEW),
            get(dashboard),
        )
        .expect("static module control contract");
    router
}
