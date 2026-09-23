mod handler;
mod service;
mod types;

use axum::routing::get;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};

pub use service::InstallationState;

pub fn public_routes() -> ContractRouter<()> {
    ContractRouter::new()
        .get(
            "/__web-binding",
            OperationDescriptor::GetWebBinding,
            AccessPolicy::Public,
            get(handler::web_binding),
        )
        .expect("static Web binding contract")
}

pub fn protected_routes() -> ContractRouter<InstallationState> {
    ContractRouter::new()
        .get(
            "/api/installation",
            OperationDescriptor::GetInstallation,
            AccessPolicy::Authenticated,
            get(handler::installation),
        )
        .expect("static installation contract")
}

#[cfg(test)]
mod tests;
