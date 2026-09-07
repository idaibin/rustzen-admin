use std::path::PathBuf;

use serde::Deserialize;

use crate::shared::{
    ConfigError, DatabaseConfig, RuntimeConfig, default_ipc_token, ensure_optional_non_empty,
    ensure_production_secret, load, local,
};

const DEFAULT_INTERNAL_HOST: &str = "127.0.0.1";
const DEFAULT_REPORTS_PORT: u16 = 9804;
const DEFAULT_REPORTS_SQLITE_PATH: &str = "./data/reports/db/reports.db";
const DEFAULT_CREDENTIAL_KEY: &str = "rustzen-development-credential-key";
#[cfg(feature = "reports-notifications")]
const DEFAULT_NOTIFICATION_INGRESS_URL: &str =
    "http://127.0.0.1:9811/internal/v1/notification-events";
#[cfg(feature = "reports-notifications")]
const DEFAULT_NOTIFICATION_EVENT_KEY_ID: &str = "reports-local-v1";
#[cfg(feature = "reports-notifications")]
const DEFAULT_NOTIFICATION_EVENT_KEY: &str = "rustzen-local-reports-notification-key-change-me";

#[derive(Debug, Clone, Deserialize)]
pub struct ReportsConfig {
    #[serde(flatten)]
    pub runtime: RuntimeConfig,
    #[serde(flatten)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub internal_host: Option<String>,
    #[serde(default)]
    pub reports_port: Option<u16>,
    #[serde(default)]
    pub reports_sqlite_path: Option<String>,
    #[serde(default = "default_ipc_token")]
    pub ipc_token: String,
    #[serde(default = "default_credential_key")]
    pub reports_credential_key: String,
    #[serde(default)]
    pub reports_browser_path: Option<String>,
    #[serde(default = "default_headless")]
    pub reports_headless: bool,
    #[serde(default = "default_max_concurrency")]
    pub reports_max_concurrency: usize,
    #[cfg(feature = "reports-notifications")]
    #[serde(default = "default_notification_ingress_url")]
    pub reports_notification_ingress_url: String,
    #[cfg(feature = "reports-notifications")]
    #[serde(default = "default_notification_event_key_id")]
    pub reports_notification_event_key_id: String,
    #[cfg(feature = "reports-notifications")]
    #[serde(default = "default_notification_event_key")]
    pub reports_notification_event_key: String,
}

impl ReportsConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config: Self = load()?;
        config.validate()?;
        Ok(config)
    }

    pub fn local() -> Result<Self, ConfigError> {
        let config: Self = local()?;
        config.validate()?;
        Ok(config)
    }

    pub fn internal_host(&self) -> &str {
        self.internal_host.as_deref().unwrap_or(DEFAULT_INTERNAL_HOST)
    }

    pub fn reports_port(&self) -> u16 {
        self.reports_port.unwrap_or(DEFAULT_REPORTS_PORT)
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.internal_host(), self.reports_port())
    }

    pub fn database_path(&self) -> PathBuf {
        self.runtime.resolve_path(
            self.reports_sqlite_path.as_deref().unwrap_or(DEFAULT_REPORTS_SQLITE_PATH),
        )
    }

    pub fn data_dir(&self) -> PathBuf {
        self.runtime.data_dir()
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime.log_dir().join("reports")
    }

    pub fn timezone(&self) -> &str {
        self.runtime.timezone()
    }

    pub fn credential_key(&self) -> &str {
        &self.reports_credential_key
    }

    pub fn browser_path(&self) -> Option<&str> {
        self.reports_browser_path.as_deref()
    }

    #[cfg(feature = "reports-notifications")]
    pub fn notification_transport(&self) -> (&str, &str, &str) {
        (
            &self.reports_notification_ingress_url,
            &self.reports_notification_event_key_id,
            &self.reports_notification_event_key,
        )
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.runtime.validate()?;
        ensure_optional_non_empty("RUSTZEN_INTERNAL_HOST", self.internal_host.as_deref())?;
        ensure_optional_non_empty(
            "RUSTZEN_REPORTS_SQLITE_PATH",
            self.reports_sqlite_path.as_deref(),
        )?;
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_IPC_TOKEN",
            &self.ipc_token,
            crate::shared::DEFAULT_IPC_TOKEN,
        )?;
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_REPORTS_CREDENTIAL_KEY",
            &self.reports_credential_key,
            DEFAULT_CREDENTIAL_KEY,
        )?;
        if !(1..=4).contains(&self.reports_max_concurrency) {
            return Err(ConfigError::Invalid("RUSTZEN_REPORTS_MAX_CONCURRENCY"));
        }
        #[cfg(feature = "reports-notifications")]
        {
            if !valid_notification_ingress_url(&self.reports_notification_ingress_url)
                || !rustzen_ipc::valid_notification_key_id(&self.reports_notification_event_key_id)
                || self.reports_notification_event_key.len() < 32
                || self.reports_notification_event_key == self.ipc_token
                || self.reports_notification_event_key == self.reports_credential_key
            {
                return Err(ConfigError::Invalid("RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY"));
            }
            ensure_production_secret(
                &self.runtime,
                "RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY",
                &self.reports_notification_event_key,
                DEFAULT_NOTIFICATION_EVENT_KEY,
            )?;
        }
        Ok(())
    }
}

#[cfg(feature = "reports-notifications")]
fn valid_notification_ingress_url(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    let loopback = match url.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        _ => false,
    };
    url.scheme() == "http"
        && loopback
        && url.port().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/internal/v1/notification-events"
        && url.query().is_none()
        && url.fragment().is_none()
}

#[cfg(feature = "reports-notifications")]
fn default_notification_ingress_url() -> String {
    DEFAULT_NOTIFICATION_INGRESS_URL.into()
}

#[cfg(feature = "reports-notifications")]
fn default_notification_event_key_id() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY_ID.into()
}

#[cfg(feature = "reports-notifications")]
fn default_notification_event_key() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY.into()
}

fn default_credential_key() -> String {
    DEFAULT_CREDENTIAL_KEY.to_string()
}

fn default_headless() -> bool {
    true
}

fn default_max_concurrency() -> usize {
    1
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "reports-notifications")]
    use super::DEFAULT_NOTIFICATION_INGRESS_URL;
    use super::{DEFAULT_CREDENTIAL_KEY, ReportsConfig};

    #[test]
    fn local_reports_config_uses_its_own_endpoint_and_database() {
        let config = ReportsConfig::local().expect("local Reports config");

        assert_eq!(config.bind_address(), "127.0.0.1:9804");
        assert!(config.database_path().ends_with("data/reports/db/reports.db"));
        assert_eq!(config.database.db_idle_timeout, None);
    }

    #[test]
    fn production_reports_requires_ipc_and_credential_secrets() {
        let mut config = ReportsConfig::local().expect("local Reports config");
        config.runtime.environment = "production".to_string();
        config.ipc_token = "production-ipc-secret".to_string();
        config.reports_credential_key = "production-credential-secret".to_string();
        #[cfg(feature = "reports-notifications")]
        {
            config.reports_notification_event_key =
                "production-reports-notification-secret".to_string();
        }
        config.validate().expect("focused production Reports config");
        config.ipc_token = "replace-me".to_string();
        assert!(config.validate().is_err());
        config.ipc_token = "production-ipc-secret".to_string();
        config.reports_credential_key = DEFAULT_CREDENTIAL_KEY.to_string();
        assert!(config.validate().is_err());
    }

    #[cfg(feature = "reports-notifications")]
    #[test]
    fn notification_transport_requires_exact_loopback_and_independent_secret() {
        let mut config = ReportsConfig::local().unwrap();
        for invalid in [
            "https://127.0.0.1:9811/internal/v1/notification-events",
            "http://localhost:9811/internal/v1/notification-events",
            "http://127.0.0.1:9811/internal/v1/notification-events?redirect=1",
            "http://user@127.0.0.1:9811/internal/v1/notification-events",
        ] {
            config.reports_notification_ingress_url = invalid.into();
            assert!(config.validate().is_err(), "accepted {invalid}");
        }
        config.reports_notification_ingress_url = DEFAULT_NOTIFICATION_INGRESS_URL.into();
        config.reports_notification_event_key = config.ipc_token.clone();
        assert!(config.validate().is_err());
        config.reports_notification_event_key = config.reports_credential_key.clone();
        assert!(config.validate().is_err());
        config.reports_notification_event_key_id = "bad\nheader".into();
        assert!(config.validate().is_err());
    }
}
