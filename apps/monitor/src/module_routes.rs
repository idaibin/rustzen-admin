use axum::Router;
use rustzen_ipc::{DelegationVerifier, ModuleDefinition, ModuleManifest, ModuleRouter};

use crate::{app::AppState, features};

const MODULE_TOML: &str = include_str!("../module.toml");
pub(crate) const CONTRACT_IPC_TOKEN: &str = "monitor-contract-only-token";

pub(crate) fn build_module_routes(
    ipc_token: &str,
) -> Result<(Router<AppState>, ModuleManifest), Box<dyn std::error::Error>> {
    let definition = ModuleDefinition::from_toml(MODULE_TOML)?;
    let module_id = definition.module.id.clone();
    let verifier = DelegationVerifier::new(ipc_token)?;
    let module_router =
        features::monitoring::routes(ModuleRouter::<AppState>::new(module_id, verifier))?;
    Ok(module_router.build(&definition, env!("CARGO_PKG_VERSION"))?)
}
