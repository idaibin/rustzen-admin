use crate::module_routes::{CONTRACT_IPC_TOKEN, build_module_routes};

pub(crate) fn selected_contract_json() -> Result<String, Box<dyn std::error::Error>> {
    let (_, manifest) = build_module_routes(CONTRACT_IPC_TOKEN)?;
    Ok(serde_json::to_string(&serde_json::json!({
        "module": manifest.module,
        "name": manifest.name,
        "apiPrefix": manifest.api_prefix,
        "contractVersion": manifest.contract_version,
        "menus": manifest.menus,
        "routes": manifest.routes,
    }))?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn selected_contract_uses_the_registered_monitor_surface() {
        let value: serde_json::Value =
            serde_json::from_str(&super::selected_contract_json().unwrap()).unwrap();
        assert_eq!(value["module"], "monitor");
        assert_eq!(value["apiPrefix"], "/api/monitor");
        #[cfg(not(feature = "notifications"))]
        #[cfg(not(feature = "notifications"))]
        assert_eq!(value["routes"].as_array().unwrap().len(), 13);
        #[cfg(feature = "notifications")]
        {
            let routes = value["routes"].as_array().unwrap();
            assert_eq!(routes.len(), 14);
            assert!(routes.iter().any(|route| {
                route["path"] == "/notification-delivery"
                    && route["permission"] == "monitor:incident:view"
            }));
        }
        #[cfg(feature = "notifications")]
        assert_eq!(value["routes"].as_array().unwrap().len(), 14);
        assert_eq!(value["menus"].as_array().unwrap().len(), 4);
        assert!(value.get("releaseVersion").is_none());
    }
}
