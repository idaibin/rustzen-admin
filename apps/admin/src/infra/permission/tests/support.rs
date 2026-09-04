use axum::http::Method;
use rustzen_ipc::{MenuDefinition, ModuleManifest, RouteManifest};

pub(super) fn monitor_manifest(
    capabilities: &[&str],
    menu_permission: &str,
    title: &str,
    path: &str,
    icon: &str,
    sort_order: i32,
) -> ModuleManifest {
    ModuleManifest {
        module: "monitor".to_string(),
        name: "Monitor".to_string(),
        api_prefix: "/api/monitor".to_string(),
        contract_version: rustzen_ipc::CONTRACT_VERSION,
        release_version: env!("CARGO_PKG_VERSION").to_string(),
        menus: vec![MenuDefinition {
            code: "monitor".to_string(),
            title: title.to_string(),
            path: path.to_string(),
            icon: icon.to_string(),
            sort_order,
            permission: menu_permission.to_string(),
        }],
        routes: capabilities
            .iter()
            .map(|capability| {
                RouteManifest::protected(
                    Method::GET,
                    format!("/{}", capability.replace(':', "-")),
                    *capability,
                )
            })
            .collect(),
    }
}
