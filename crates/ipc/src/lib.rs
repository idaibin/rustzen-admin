//! Internal contracts shared by the four Rustzen backend applications.

mod delegation;
mod extract;
mod health;
mod manifest;
mod response;
mod router;

pub use delegation::{
    CONTRACT_VERSION, DelegatedAccess, DelegatedContext, DelegationError, DelegationSigner,
    DelegationVerifier, IPC_ACCESS_HEADER, IPC_CONTRACT_VERSION_HEADER, IPC_MODULE_HEADER,
    IPC_REQUEST_ID_HEADER, IPC_SIGNATURE_HEADER, IPC_TIMESTAMP_HEADER, IPC_USER_ID_HEADER,
};
pub use extract::{ModuleInputRejection, ModuleJson, ModuleJsonRejection, ModuleQuery};
pub use health::HealthResponse;
pub use manifest::{
    AccessMode, ManifestError, MenuDefinition, ModuleDefinition, ModuleManifest, ModuleMetadata,
    RouteManifest,
};
pub use response::{ApiResponse, Page, Pagination, PaginationError};
pub use router::{ModuleRouter, Require};
