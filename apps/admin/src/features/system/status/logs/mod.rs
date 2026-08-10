pub mod handler;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{get, post};
use handler::{
    backup_module_logs, confirm_module_log_cleanup, list_module_logs, preview_module_log_cleanup,
    tail_module_log,
};
use rustzen_auth::capability::system_module_log;
use sqlx::SqlitePool;

pub fn module_log_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListModuleLogs,
            AccessPolicy::Require(system_module_log::VIEW),
            get(list_module_logs),
        )
        .expect("static module log contract")
        .get(
            "/tail",
            OperationDescriptor::TailModuleLog,
            AccessPolicy::Require(system_module_log::VIEW),
            get(tail_module_log),
        )
        .expect("static module log contract")
        .post(
            "/backup",
            OperationDescriptor::BackupModuleLogs,
            AccessPolicy::Require(system_module_log::BACKUP),
            post(backup_module_logs),
        )
        .expect("static module log contract")
        .post(
            "/cleanup/preview",
            OperationDescriptor::PreviewModuleLogCleanup,
            AccessPolicy::Require(system_module_log::CLEANUP),
            post(preview_module_log_cleanup),
        )
        .expect("static module log contract")
        .post(
            "/cleanup/confirm",
            OperationDescriptor::ConfirmModuleLogCleanup,
            AccessPolicy::Require(system_module_log::CLEANUP),
            post(confirm_module_log_cleanup),
        )
        .expect("static module log contract")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::contract::{OperationDescriptor, RegisteredAccess};
    use rustzen_auth::capability::{RolePolicy, system_module_log};

    #[test]
    fn module_log_routes_remain_owner_only_for_each_action() {
        let (_, contracts) = module_log_routes().into_parts();
        assert_eq!(contracts.len(), 5);
        for contract in contracts {
            let capability = match contract.operation {
                OperationDescriptor::ListModuleLogs | OperationDescriptor::TailModuleLog => {
                    system_module_log::VIEW
                }
                OperationDescriptor::BackupModuleLogs => system_module_log::BACKUP,
                OperationDescriptor::PreviewModuleLogCleanup
                | OperationDescriptor::ConfirmModuleLogCleanup => system_module_log::CLEANUP,
                operation => panic!("unexpected module log operation: {operation:?}"),
            };
            assert_eq!(contract.access, RegisteredAccess::Require(vec![capability.to_owned()]));
            assert!(RolePolicy.is_owner_only_capability(capability));
            assert!(!RolePolicy.role_allows_capability("admin", capability));
            assert!(!RolePolicy.role_allows_capability("viewer", capability));
            assert!(!RolePolicy.role_allows_capability("custom", capability));
        }
    }
}
