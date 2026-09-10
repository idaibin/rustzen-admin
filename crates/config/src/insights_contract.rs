use super::contract::{ConfigContract, ConfigField};

pub fn insights_contract() -> ConfigContract {
    let mut fields = vec![
        field("RUSTZEN_ENV", "environment", false, "built-in"),
        field("RUSTZEN_RUNTIME_ROOT", "path", false, "built-in"),
        field("RUSTZEN_TIMEZONE", "timezone", false, "built-in"),
        field("RUSTZEN_DB_CONN_TIMEOUT", "seconds", false, "built-in"),
        field("RUSTZEN_DB_IDLE_TIMEOUT", "seconds", false, "built-in"),
        field("RUSTZEN_DB_MAX_CONN", "integer", false, "built-in"),
        field("RUSTZEN_DB_MIN_CONN", "integer", false, "built-in"),
        field("RUSTZEN_INTERNAL_HOST", "host", false, "built-in"),
        secret("RUSTZEN_IPC_TOKEN", "insights.ipc"),
        field("RUSTZEN_INSIGHTS_PORT", "port", false, "built-in"),
        field("RUSTZEN_INSIGHTS_SQLITE_PATH", "path", false, "built-in"),
    ];
    fields.sort_by_key(|item| item.key);
    ConfigContract { version: 1, owner: "insights", consumer: "rz-insights", fields }
}

fn field(
    key: &'static str,
    value_type: &'static str,
    required: bool,
    default_class: &'static str,
) -> ConfigField {
    ConfigField { key, value_type, required, default_class, secret_ref: None }
}

fn secret(key: &'static str, secret_ref: &'static str) -> ConfigField {
    ConfigField {
        key,
        value_type: "secret",
        required: true,
        default_class: "development-only",
        secret_ref: Some(secret_ref),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_matches_insights_config_without_foreign_secrets() {
        let contract = super::insights_contract();
        assert_eq!((contract.owner, contract.consumer), ("insights", "rz-insights"));
        assert_eq!(contract.fields.len(), 11);
        assert_eq!(
            contract.fields.iter().map(|field| field.key).collect::<Vec<_>>(),
            [
                "RUSTZEN_DB_CONN_TIMEOUT",
                "RUSTZEN_DB_IDLE_TIMEOUT",
                "RUSTZEN_DB_MAX_CONN",
                "RUSTZEN_DB_MIN_CONN",
                "RUSTZEN_ENV",
                "RUSTZEN_INSIGHTS_PORT",
                "RUSTZEN_INSIGHTS_SQLITE_PATH",
                "RUSTZEN_INTERNAL_HOST",
                "RUSTZEN_IPC_TOKEN",
                "RUSTZEN_RUNTIME_ROOT",
                "RUSTZEN_TIMEZONE",
            ]
        );
        assert_eq!(contract.fields[8].secret_ref, Some("insights.ipc"));
    }
}
