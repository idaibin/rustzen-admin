pub mod gateway;
pub mod handler;
pub mod registry;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{get, put};
use rustzen_auth::capability::{dashboard, system_module};

use self::{
    handler::{dashboard, list, navigation, update},
    service::ModuleControlState,
};

pub fn control_routes() -> ContractRouter<ModuleControlState> {
    ContractRouter::new()
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
            "/api/system/modules/navigation",
            OperationDescriptor::GetModuleNavigation,
            AccessPolicy::Authenticated,
            get(navigation),
        )
        .expect("static module control contract")
        .get(
            "/api/dashboard/modules",
            OperationDescriptor::GetDashboardModules,
            AccessPolicy::Require(dashboard::VIEW),
            get(dashboard),
        )
        .expect("static module control contract")
}
