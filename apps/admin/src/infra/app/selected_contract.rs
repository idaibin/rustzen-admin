use super::*;

#[cfg(feature = "monitor-distribution")]
pub(crate) fn selected_contract_json() -> Result<String, serde_json::Error> {
    let mut routes = documented_all_contracts()
        .into_iter()
        .map(|contract| {
            let access = match contract.access {
                crate::infra::contract::RegisteredAccess::Public => {
                    serde_json::json!({"kind":"public"})
                }
                crate::infra::contract::RegisteredAccess::Authenticated => {
                    serde_json::json!({"kind":"authenticated"})
                }
                crate::infra::contract::RegisteredAccess::Require(mut capabilities) => {
                    capabilities.sort();
                    serde_json::json!({"kind":"require","capabilities":capabilities})
                }
                crate::infra::contract::RegisteredAccess::Any(mut capabilities) => {
                    capabilities.sort();
                    serde_json::json!({"kind":"any","capabilities":capabilities})
                }
                crate::infra::contract::RegisteredAccess::All(mut capabilities) => {
                    capabilities.sort();
                    serde_json::json!({"kind":"all","capabilities":capabilities})
                }
            };
            serde_json::json!({
                "method": contract.method.as_str(),
                "path": contract.path,
                "operation": contract.operation.operation_id(),
                "access": access,
            })
        })
        .collect::<Vec<_>>();
    routes.sort_by(|left, right| {
        left["method"]
            .as_str()
            .cmp(&right["method"].as_str())
            .then(left["path"].as_str().cmp(&right["path"].as_str()))
    });
    serde_json::to_string(&serde_json::json!({"version": 1, "routes": routes}))
}

#[cfg(all(test, feature = "monitor-distribution"))]
mod tests {
    #[test]
    fn selected_contract_uses_the_registered_minimal_admin_surface() {
        let value: serde_json::Value =
            serde_json::from_str(&super::selected_contract_json().unwrap()).unwrap();
        let routes = value["routes"].as_array().unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(routes.len(), 20);
        assert!(routes.iter().any(|route| route["path"] == "/api/auth/login"));
        assert!(routes.iter().any(|route| route["path"] == "/api/system/modules/navigation"));
        assert!(!routes.iter().any(|route| route["path"] == "/api/reports"));
    }
}
