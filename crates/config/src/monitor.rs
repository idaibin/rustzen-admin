use std::path::PathBuf;

use serde::Deserialize;

use crate::shared::{
    ConfigError, RuntimeConfig, default_monitor_agent_token, ensure_production_secret, load, local,
};
#[cfg(feature = "monitor-controller")]
use crate::shared::{DatabaseConfig, default_ipc_token, ensure_optional_non_empty};
#[cfg(feature = "monitor-agent")]
use crate::shared::{ensure_http_url, ensure_required_non_empty};

#[cfg(feature = "monitor-agent")]
const DEFAULT_ADMIN_PORT: u16 = 9801;
#[cfg(feature = "monitor-controller")]
const DEFAULT_INTERNAL_HOST: &str = "127.0.0.1";
#[cfg(feature = "monitor-controller")]
const DEFAULT_MONITOR_PORT: u16 = 9802;
#[cfg(feature = "monitor-controller")]
const DEFAULT_MONITOR_SQLITE_PATH: &str = "./data/db/monitor.db";
#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
const DEFAULT_NOTIFICATION_INGRESS_URL: &str =
    "http://127.0.0.1:9811/internal/v1/notification-events";
#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
const DEFAULT_NOTIFICATION_EVENT_KEY_ID: &str = "local-v1";
#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
const DEFAULT_NOTIFICATION_EVENT_KEY: &str = "rustzen-local-notification-key-change-me";

#[cfg(feature = "monitor-controller")]
#[derive(Debug, Clone, Deserialize)]
pub struct MonitorControllerConfig {
    #[serde(flatten)]
    pub runtime: RuntimeConfig,
    #[serde(flatten)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub internal_host: Option<String>,
    #[serde(default)]
    pub monitor_port: Option<u16>,
    #[serde(default)]
    pub monitor_sqlite_path: Option<String>,
    #[serde(default = "default_ipc_token")]
    pub ipc_token: String,
    #[serde(default = "default_monitor_agent_token")]
    pub monitor_agent_token: String,
    #[cfg(feature = "notifications")]
    #[serde(default = "default_notification_ingress_url")]
    pub notification_ingress_url: String,
    #[cfg(feature = "notifications")]
    #[serde(default = "default_notification_event_key_id")]
    pub notification_event_key_id: String,
    #[cfg(feature = "notifications")]
    #[serde(default = "default_notification_event_key")]
    pub notification_event_key: String,
}

#[cfg(feature = "monitor-controller")]
impl MonitorControllerConfig {
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

    pub fn monitor_port(&self) -> u16 {
        self.monitor_port.unwrap_or(DEFAULT_MONITOR_PORT)
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.internal_host(), self.monitor_port())
    }

    pub fn database_path(&self) -> PathBuf {
        self.runtime.resolve_path(
            self.monitor_sqlite_path.as_deref().unwrap_or(DEFAULT_MONITOR_SQLITE_PATH),
        )
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime.log_dir()
    }

    pub fn timezone(&self) -> &str {
        self.runtime.timezone()
    }

    #[cfg(feature = "notifications")]
    pub fn notification_transport(&self) -> (&str, &str, &str) {
        (
            &self.notification_ingress_url,
            &self.notification_event_key_id,
            &self.notification_event_key,
        )
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.runtime.validate()?;
        ensure_optional_non_empty("RUSTZEN_INTERNAL_HOST", self.internal_host.as_deref())?;
        ensure_optional_non_empty(
            "RUSTZEN_MONITOR_SQLITE_PATH",
            self.monitor_sqlite_path.as_deref(),
        )?;
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_IPC_TOKEN",
            &self.ipc_token,
            crate::shared::DEFAULT_IPC_TOKEN,
        )?;
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_MONITOR_AGENT_TOKEN",
            &self.monitor_agent_token,
            crate::shared::DEFAULT_MONITOR_AGENT_TOKEN,
        )?;
        if self.runtime.requires_production_secrets()
            && !crate::shared::valid_monitor_agent_token(&self.monitor_agent_token)
        {
            return Err(ConfigError::Invalid("RUSTZEN_MONITOR_AGENT_TOKEN"));
        }
        #[cfg(feature = "notifications")]
        {
            if !valid_notification_ingress_url(&self.notification_ingress_url)
                || !rustzen_ipc::valid_notification_key_id(&self.notification_event_key_id)
                || self.notification_event_key.len() < 32
                || self.notification_event_key == self.ipc_token
                || self.notification_event_key == self.monitor_agent_token
            {
                return Err(ConfigError::Invalid("RUSTZEN_NOTIFICATION_EVENT_KEY"));
            }
            ensure_production_secret(
                &self.runtime,
                "RUSTZEN_NOTIFICATION_EVENT_KEY",
                &self.notification_event_key,
                DEFAULT_NOTIFICATION_EVENT_KEY,
            )?;
        }
        Ok(())
    }
}

#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
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

#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
fn default_notification_ingress_url() -> String {
    DEFAULT_NOTIFICATION_INGRESS_URL.into()
}
#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
fn default_notification_event_key_id() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY_ID.into()
}
#[cfg(all(feature = "monitor-controller", feature = "notifications"))]
fn default_notification_event_key() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY.into()
}

#[cfg(feature = "monitor-agent")]
#[derive(Debug, Clone, Deserialize)]
pub struct MonitorAgentConfig {
    #[serde(flatten)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub admin_port: Option<u16>,
    #[serde(default = "default_monitor_agent_token")]
    pub monitor_agent_token: String,
    #[serde(default)]
    pub monitor_node_id: Option<String>,
    #[serde(default)]
    pub monitor_controller_url: Option<String>,
}

#[cfg(feature = "monitor-agent")]
impl MonitorAgentConfig {
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

    pub fn node_id(&self) -> Result<&str, ConfigError> {
        self.monitor_node_id.as_deref().ok_or(ConfigError::Empty("RUSTZEN_MONITOR_NODE_ID"))
    }

    pub fn admin_port(&self) -> u16 {
        self.admin_port.unwrap_or(DEFAULT_ADMIN_PORT)
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime.log_dir()
    }

    pub fn timezone(&self) -> &str {
        self.runtime.timezone()
    }

    pub fn agent_root(&self) -> PathBuf {
        PathBuf::from("/opt/rz")
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.runtime.validate()?;
        let node_id =
            self.monitor_node_id.as_deref().ok_or(ConfigError::Empty("RUSTZEN_MONITOR_NODE_ID"))?;
        ensure_required_non_empty("RUSTZEN_MONITOR_NODE_ID", node_id)?;
        if node_id.len() > 128
            || !node_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(ConfigError::Invalid("RUSTZEN_MONITOR_NODE_ID"));
        }
        ensure_http_url("RUSTZEN_MONITOR_CONTROLLER_URL", self.monitor_controller_url.as_deref())?;
        if self.runtime.requires_production_secrets()
            && self
                .monitor_controller_url
                .as_deref()
                .is_some_and(|url| !url.trim().starts_with("https://"))
        {
            return Err(ConfigError::Invalid("RUSTZEN_MONITOR_CONTROLLER_URL"));
        }
        if self.runtime.requires_production_secrets() && self.monitor_controller_url.is_none() {
            return Err(ConfigError::Empty("RUSTZEN_MONITOR_CONTROLLER_URL"));
        }
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_MONITOR_AGENT_TOKEN",
            &self.monitor_agent_token,
            crate::shared::DEFAULT_MONITOR_AGENT_TOKEN,
        )?;
        if self.runtime.requires_production_secrets()
            && !crate::shared::valid_monitor_agent_token(&self.monitor_agent_token)
        {
            return Err(ConfigError::Invalid("RUSTZEN_MONITOR_AGENT_TOKEN"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "monitor-agent")]
    use figment::{Figment, providers::Serialized};
    #[cfg(feature = "monitor-agent")]
    use serde::Serialize;

    #[cfg(feature = "monitor-agent")]
    use super::MonitorAgentConfig;
    #[cfg(feature = "monitor-controller")]
    use super::MonitorControllerConfig;

    #[cfg(feature = "monitor-agent")]
    #[derive(Serialize)]
    struct InvalidControllerOnlySettings<'a> {
        monitor_port: &'a str,
        ipc_token: &'a str,
        monitor_sqlite_path: &'a str,
    }

    #[cfg(feature = "monitor-agent")]
    #[derive(Serialize)]
    struct AgentSettings<'a> {
        monitor_node_id: &'a str,
    }

    #[cfg(feature = "monitor-agent")]
    fn local_agent() -> MonitorAgentConfig {
        Figment::new()
            .merge(Serialized::defaults(AgentSettings { monitor_node_id: "local-agent" }))
            .extract()
            .expect("local Monitor Agent config")
    }

    #[cfg(feature = "monitor-controller")]
    #[test]
    fn local_controller_config_parses_only_controller_settings() {
        let config = MonitorControllerConfig::local().expect("local Monitor controller config");

        assert_eq!(config.bind_address(), "127.0.0.1:9802");
        assert!(config.database_path().ends_with("data/db/monitor.db"));
        assert_eq!(config.database.db_idle_timeout, None);
    }

    #[cfg(all(feature = "monitor-controller", feature = "notifications"))]
    #[test]
    fn notification_ingress_requires_exact_loopback_origin_and_path() {
        let base = MonitorControllerConfig::local().unwrap();
        for invalid in [
            "https://127.0.0.1:9811/internal/v1/notification-events",
            "http://localhost:9811/internal/v1/notification-events",
            "http://192.0.2.1:9811/internal/v1/notification-events",
            "http://user@127.0.0.1:9811/internal/v1/notification-events",
            "http://127.0.0.1:9811/internal/v1/notification-events?x=1",
            "http://127.0.0.1:9811/internal/v1/notification-events#x",
            "http://127.0.0.1:9811/internal/v1/notification-events/extra",
            "http://127.0.0.1/internal/v1/notification-events",
        ] {
            let mut config = base.clone();
            config.notification_ingress_url = invalid.into();
            assert!(config.validate().is_err(), "accepted {invalid}");
        }
        let mut ipv6 = base;
        ipv6.notification_ingress_url = "http://[::1]:9811/internal/v1/notification-events".into();
        ipv6.validate().unwrap();
    }

    #[cfg(all(feature = "monitor-controller", feature = "notifications"))]
    #[test]
    fn notification_key_is_not_reused_for_ipc_or_agent_authentication() {
        let base = MonitorControllerConfig::local().unwrap();
        for invalid_id in ["bad\nheader", "bad id"] {
            let mut invalid = base.clone();
            invalid.notification_event_key_id = invalid_id.into();
            assert!(invalid.validate().is_err());
        }
        for reused in [base.ipc_token.clone(), base.monitor_agent_token.clone()] {
            let mut invalid = base.clone();
            invalid.notification_event_key = reused;
            assert!(invalid.validate().is_err());
        }
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn local_agent_config_parses_only_agent_settings() {
        let _config = local_agent();
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn agent_config_ignores_invalid_controller_only_settings() {
        let mut config: MonitorAgentConfig = Figment::new()
            .merge(Serialized::defaults(InvalidControllerOnlySettings {
                monitor_port: "not-a-number",
                ipc_token: "",
                monitor_sqlite_path: "",
            }))
            .extract()
            .expect("focused Agent extraction");

        config.monitor_node_id = Some("stable-agent".to_string());
        config.validate().expect("focused Agent validation");
    }

    #[cfg(feature = "monitor-controller")]
    #[test]
    fn production_controller_validates_its_own_secrets() {
        let mut controller =
            MonitorControllerConfig::local().expect("local Monitor controller config");
        controller.runtime.environment = "production".to_string();
        controller.ipc_token = "production-ipc-secret".to_string();
        controller.monitor_agent_token = "production-agent-secret".to_string();
        #[cfg(feature = "notifications")]
        {
            controller.notification_event_key = "production-notification-event-secret".to_string();
        }
        controller.validate().expect("production controller config");
        controller.monitor_agent_token = "replace-me".to_string();
        assert!(controller.validate().is_err());
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn production_agent_validates_its_own_secret() {
        let mut agent = local_agent();
        agent.runtime.environment = "production".to_string();
        agent.monitor_controller_url = Some("https://monitor.example".to_string());
        agent.monitor_agent_token = "production-agent-secret".to_string();
        agent.validate().expect("production Agent config without IPC or database fields");
        agent.monitor_agent_token = "replace-me".to_string();
        assert!(agent.validate().is_err());
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn monitor_agent_rejects_empty_or_invalid_remote_controller_urls() {
        let mut agent = local_agent();
        for url in ["", "http://", "ftp://monitor.example"] {
            agent.monitor_controller_url = Some(url.to_string());
            assert!(agent.validate().is_err(), "accepted {url:?}");
        }
        agent.monitor_controller_url = Some("http://127.0.0.1".to_string());
        agent.validate().expect("valid internal HTTP controller URL");
        agent.runtime.environment = "production".to_string();
        assert!(agent.validate().is_err(), "production rejects IPv4 HTTP");
        agent.monitor_controller_url = Some("http://[::1]".to_string());
        assert!(agent.validate().is_err(), "production rejects IPv6 HTTP");
        agent.runtime.environment = "development".to_string();
        agent.monitor_controller_url = Some("https://monitor.example".to_string());
        agent.validate().expect("valid remote controller URL");
    }

    #[cfg(feature = "monitor-agent")]
    #[test]
    fn agent_node_id_is_explicit_and_never_derived_from_hostname() {
        let mut agent = local_agent();
        assert_eq!(agent.node_id().unwrap(), "local-agent");
        agent.monitor_node_id = Some("stable-node".to_string());
        agent.validate().expect("stable node id");
        agent.monitor_node_id = Some("host name".to_string());
        assert!(agent.validate().is_err());
        agent.monitor_node_id = Some("  ".to_string());
        assert!(agent.validate().is_err());
        agent.monitor_node_id = None;
        assert!(agent.validate().is_err());
    }
}
