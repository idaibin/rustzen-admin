use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigContract {
    pub version: u8,
    pub owner: &'static str,
    pub consumer: &'static str,
    pub fields: Vec<ConfigField>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigField {
    pub key: &'static str,
    pub value_type: &'static str,
    pub required: bool,
    pub default_class: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<&'static str>,
}

#[cfg(any(feature = "admin-monitor", feature = "monitor-controller", feature = "monitor-agent"))]
fn field(
    key: &'static str,
    value_type: &'static str,
    required: bool,
    default_class: &'static str,
) -> ConfigField {
    ConfigField { key, value_type, required, default_class, secret_ref: None }
}
#[cfg(any(feature = "admin-monitor", feature = "monitor-controller", feature = "monitor-agent"))]
fn secret(key: &'static str, secret_ref: &'static str) -> ConfigField {
    ConfigField {
        key,
        value_type: "secret",
        required: true,
        default_class: "development-only",
        secret_ref: Some(secret_ref),
    }
}
#[cfg(any(feature = "admin-monitor", feature = "monitor-controller", feature = "monitor-agent"))]
fn runtime_fields() -> Vec<ConfigField> {
    vec![
        field("RUSTZEN_ENV", "environment", false, "built-in"),
        field("RUSTZEN_RUNTIME_ROOT", "path", false, "built-in"),
        field("RUSTZEN_TIMEZONE", "timezone", false, "built-in"),
    ]
}
#[cfg(any(feature = "admin-monitor", feature = "monitor-controller"))]
fn database_fields() -> Vec<ConfigField> {
    vec![
        field("RUSTZEN_DB_CONN_TIMEOUT", "seconds", false, "built-in"),
        field("RUSTZEN_DB_IDLE_TIMEOUT", "seconds", false, "built-in"),
        field("RUSTZEN_DB_MAX_CONN", "integer", false, "built-in"),
        field("RUSTZEN_DB_MIN_CONN", "integer", false, "built-in"),
    ]
}
#[cfg(any(feature = "admin-monitor", feature = "monitor-controller", feature = "monitor-agent"))]
fn contract(
    owner: &'static str,
    consumer: &'static str,
    mut fields: Vec<ConfigField>,
) -> ConfigContract {
    fields.sort_by_key(|item| item.key);
    ConfigContract { version: 1, owner, consumer, fields }
}

#[cfg(feature = "admin-monitor")]
pub fn admin_monitor_contract() -> ConfigContract {
    let mut fields = runtime_fields();
    fields.extend(database_fields());
    fields.extend([
        field("RUSTZEN_ADMIN_HOST", "host", false, "built-in"),
        field("RUSTZEN_ADMIN_PORT", "port", false, "built-in"),
        field("RUSTZEN_ADMIN_SQLITE_PATH", "path", false, "built-in"),
        field("RUSTZEN_INTERNAL_HOST", "host", false, "built-in"),
        field("RUSTZEN_JWT_EXPIRATION", "seconds", false, "built-in"),
        secret("RUSTZEN_JWT_SECRET", "auth.jwt"),
        field("RUSTZEN_MONITOR_PORT", "port", false, "built-in"),
        secret("RUSTZEN_IPC_TOKEN", "monitor.ipc"),
    ]);
    contract("access", "rz-admin", fields)
}

#[cfg(feature = "monitor-controller")]
pub fn monitor_controller_contract() -> ConfigContract {
    let mut fields = runtime_fields();
    fields.extend(database_fields());
    fields.extend([
        field("RUSTZEN_INTERNAL_HOST", "host", false, "built-in"),
        field("RUSTZEN_MONITOR_PORT", "port", false, "built-in"),
        field("RUSTZEN_MONITOR_SQLITE_PATH", "path", false, "built-in"),
        secret("RUSTZEN_IPC_TOKEN", "monitor.ipc"),
        secret("RUSTZEN_MONITOR_AGENT_TOKEN", "monitor.agent"),
    ]);
    contract("monitor", "rz-monitor", fields)
}

#[cfg(feature = "monitor-agent")]
pub fn monitor_agent_contract() -> ConfigContract {
    let mut fields = runtime_fields();
    fields.extend([
        field("RUSTZEN_ADMIN_PORT", "port", false, "local-fallback"),
        secret("RUSTZEN_MONITOR_AGENT_TOKEN", "monitor.agent"),
        field("RUSTZEN_MONITOR_CONTROLLER_URL", "url", false, "local-fallback"),
        field("RUSTZEN_MONITOR_NODE_ID", "identifier", true, "none"),
    ]);
    contract("monitor-agent", "rz-monitor-agent", fields)
}

#[cfg(all(
    test,
    any(feature = "admin-monitor", feature = "monitor-controller", feature = "monitor-agent")
))]
mod tests {
    fn keys(contract: &super::ConfigContract) -> Vec<&str> {
        contract.fields.iter().map(|field| field.key).collect()
    }
    fn secret_refs(contract: &super::ConfigContract) -> Vec<(&str, &str)> {
        contract
            .fields
            .iter()
            .filter_map(|field| field.secret_ref.map(|reference| (field.key, reference)))
            .collect()
    }
    fn assert_complete_metadata(contract: &super::ConfigContract) {
        for field in &contract.fields {
            let value_type = match field.key {
                "RUSTZEN_ADMIN_HOST" | "RUSTZEN_INTERNAL_HOST" => "host",
                "RUSTZEN_ADMIN_PORT" | "RUSTZEN_MONITOR_PORT" => "port",
                "RUSTZEN_ADMIN_SQLITE_PATH"
                | "RUSTZEN_MONITOR_SQLITE_PATH"
                | "RUSTZEN_RUNTIME_ROOT" => "path",
                "RUSTZEN_DB_CONN_TIMEOUT"
                | "RUSTZEN_DB_IDLE_TIMEOUT"
                | "RUSTZEN_JWT_EXPIRATION" => "seconds",
                "RUSTZEN_DB_MAX_CONN" | "RUSTZEN_DB_MIN_CONN" => "integer",
                "RUSTZEN_ENV" => "environment",
                "RUSTZEN_TIMEZONE" => "timezone",
                "RUSTZEN_MONITOR_CONTROLLER_URL" => "url",
                "RUSTZEN_MONITOR_NODE_ID" => "identifier",
                "RUSTZEN_IPC_TOKEN" | "RUSTZEN_JWT_SECRET" | "RUSTZEN_MONITOR_AGENT_TOKEN" => {
                    "secret"
                }
                key => panic!("unreviewed config descriptor key: {key}"),
            };
            let (required, default_class, secret_ref) = match field.key {
                "RUSTZEN_IPC_TOKEN" => (true, "development-only", Some("monitor.ipc")),
                "RUSTZEN_JWT_SECRET" => (true, "development-only", Some("auth.jwt")),
                "RUSTZEN_MONITOR_AGENT_TOKEN" => (true, "development-only", Some("monitor.agent")),
                "RUSTZEN_MONITOR_NODE_ID" => (true, "none", None),
                "RUSTZEN_MONITOR_CONTROLLER_URL" => (false, "local-fallback", None),
                "RUSTZEN_ADMIN_PORT" if contract.owner == "monitor-agent" => {
                    (false, "local-fallback", None)
                }
                _ => (false, "built-in", None),
            };
            assert_eq!(
                (field.value_type, field.required, field.default_class, field.secret_ref),
                (value_type, required, default_class, secret_ref),
                "metadata mismatch for {}",
                field.key
            );
        }
    }

    #[test]
    fn descriptors_never_serialize_values_or_secret_literals() {
        #[cfg(feature = "admin-monitor")]
        let contract = super::admin_monitor_contract();
        #[cfg(all(not(feature = "admin-monitor"), feature = "monitor-controller"))]
        let contract = super::monitor_controller_contract();
        #[cfg(all(
            not(feature = "admin-monitor"),
            not(feature = "monitor-controller"),
            feature = "monitor-agent"
        ))]
        let contract = super::monitor_agent_contract();
        let json = serde_json::to_string(&contract).unwrap();
        assert!(!json.contains("\"value\":"));
        assert!(!json.contains("replace-me"));
        assert!(contract.fields.windows(2).all(|pair| pair[0].key < pair[1].key));
        assert_complete_metadata(&contract);
    }

    #[cfg(feature = "admin-monitor")]
    #[test]
    fn monitor_admin_descriptor_is_exact() {
        let contract = super::admin_monitor_contract();
        assert_eq!((contract.owner, contract.consumer), ("access", "rz-admin"));
        assert_eq!(
            keys(&contract),
            [
                "RUSTZEN_ADMIN_HOST",
                "RUSTZEN_ADMIN_PORT",
                "RUSTZEN_ADMIN_SQLITE_PATH",
                "RUSTZEN_DB_CONN_TIMEOUT",
                "RUSTZEN_DB_IDLE_TIMEOUT",
                "RUSTZEN_DB_MAX_CONN",
                "RUSTZEN_DB_MIN_CONN",
                "RUSTZEN_ENV",
                "RUSTZEN_INTERNAL_HOST",
                "RUSTZEN_IPC_TOKEN",
                "RUSTZEN_JWT_EXPIRATION",
                "RUSTZEN_JWT_SECRET",
                "RUSTZEN_MONITOR_PORT",
                "RUSTZEN_RUNTIME_ROOT",
                "RUSTZEN_TIMEZONE"
            ]
        );
        assert_eq!(
            secret_refs(&contract),
            [("RUSTZEN_IPC_TOKEN", "monitor.ipc"), ("RUSTZEN_JWT_SECRET", "auth.jwt")]
        );
    }

    #[cfg(feature = "monitor-controller")]
    #[test]
    fn monitor_controller_descriptor_is_exact() {
        let contract = super::monitor_controller_contract();
        assert_eq!((contract.owner, contract.consumer), ("monitor", "rz-monitor"));
        assert_eq!(
            keys(&contract),
            [
                "RUSTZEN_DB_CONN_TIMEOUT",
                "RUSTZEN_DB_IDLE_TIMEOUT",
                "RUSTZEN_DB_MAX_CONN",
                "RUSTZEN_DB_MIN_CONN",
                "RUSTZEN_ENV",
                "RUSTZEN_INTERNAL_HOST",
                "RUSTZEN_IPC_TOKEN",
                "RUSTZEN_MONITOR_AGENT_TOKEN",
                "RUSTZEN_MONITOR_PORT",
                "RUSTZEN_MONITOR_SQLITE_PATH",
                "RUSTZEN_RUNTIME_ROOT",
                "RUSTZEN_TIMEZONE"
            ]
        );
        assert_eq!(
            secret_refs(&contract),
            [
                ("RUSTZEN_IPC_TOKEN", "monitor.ipc"),
                ("RUSTZEN_MONITOR_AGENT_TOKEN", "monitor.agent")
            ]
        );
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn monitor_agent_descriptor_is_exact() {
        let contract = super::monitor_agent_contract();
        assert_eq!((contract.owner, contract.consumer), ("monitor-agent", "rz-monitor-agent"));
        assert_eq!(
            keys(&contract),
            [
                "RUSTZEN_ADMIN_PORT",
                "RUSTZEN_ENV",
                "RUSTZEN_MONITOR_AGENT_TOKEN",
                "RUSTZEN_MONITOR_CONTROLLER_URL",
                "RUSTZEN_MONITOR_NODE_ID",
                "RUSTZEN_RUNTIME_ROOT",
                "RUSTZEN_TIMEZONE"
            ]
        );
        assert_eq!(secret_refs(&contract), [("RUSTZEN_MONITOR_AGENT_TOKEN", "monitor.agent")]);
    }
}
