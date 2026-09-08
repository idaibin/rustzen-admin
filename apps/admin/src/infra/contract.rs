//! Bounded, code-first route contract registration for Admin routes.
//!
//! This is deliberately local to Admin: one typed registration owns both the
//! Axum route and the metadata consumed by selected contracts and OpenAPI.

mod operation;
mod router;

pub use operation::{
    AccessPolicy, ContractError, OperationDescriptor, RegisteredAccess, RouteContract,
};
pub use router::ContractRouter;

#[cfg(test)]
use router::{join_path, normalize_access};

#[cfg(test)]
mod tests;
