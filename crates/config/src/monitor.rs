use std::path::PathBuf;

use serde::Deserialize;

use crate::shared::{
    ConfigError, DatabaseConfig, RuntimeConfig, default_ipc_token, default_monitor_agent_token,
    ensure_http_url, ensure_optional_non_empty, ensure_production_secret, load, local,
};

const DEFAULT_ADMIN_PORT: u16 = 9801;
const DEFAULT_INTERNAL_HOST: &str = "127.0.0.1";
const DEFAULT_MONITOR_PORT: u16 = 9802;
const DEFAULT_MONITOR_SQLITE_PATH: &str = "./data/db/monitor.db";

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
}

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
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitorAgentConfig {
    #[serde(flatten)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub admin_port: Option<u16>,
    #[serde(default = "default_monitor_agent_token")]
    pub monitor_agent_token: String,
    #[serde(default)]
    pub monitor_controller_url: Option<String>,
}

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

    pub fn admin_port(&self) -> u16 {
        self.admin_port.unwrap_or(DEFAULT_ADMIN_PORT)
    }

    pub fn heartbeat_endpoint(&self) -> String {
        let base = self
            .monitor_controller_url
            .as_deref()
            .map(str::trim)
            .map(str::to_string)
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", self.admin_port()));
        format!("{}/api/monitor/heartbeat", base.trim_end_matches('/'))
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime.log_dir()
    }

    pub fn timezone(&self) -> &str {
        self.runtime.timezone()
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.runtime.validate()?;
        ensure_http_url("RUSTZEN_MONITOR_CONTROLLER_URL", self.monitor_controller_url.as_deref())?;
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_MONITOR_AGENT_TOKEN",
            &self.monitor_agent_token,
            crate::shared::DEFAULT_MONITOR_AGENT_TOKEN,
        )
    }
}

#[cfg(test)]
mod tests {
    use figment::{Figment, providers::Serialized};
    use serde::Serialize;

    use super::{MonitorAgentConfig, MonitorControllerConfig};

    #[derive(Serialize)]
    struct InvalidControllerOnlySettings<'a> {
        monitor_port: &'a str,
        ipc_token: &'a str,
        monitor_sqlite_path: &'a str,
    }

    #[test]
    fn local_controller_config_parses_only_controller_settings() {
        let config = MonitorControllerConfig::local().expect("local Monitor controller config");

        assert_eq!(config.bind_address(), "127.0.0.1:9802");
        assert!(config.database_path().ends_with("data/db/monitor.db"));
        assert_eq!(config.database.db_idle_timeout, None);
    }

    #[test]
    fn local_agent_config_parses_only_agent_settings() {
        let config = MonitorAgentConfig::local().expect("local Monitor Agent config");

        assert_eq!(config.heartbeat_endpoint(), "http://127.0.0.1:9801/api/monitor/heartbeat");
    }

    #[test]
    fn agent_config_ignores_invalid_controller_only_settings() {
        let config: MonitorAgentConfig = Figment::new()
            .merge(Serialized::defaults(InvalidControllerOnlySettings {
                monitor_port: "not-a-number",
                ipc_token: "",
                monitor_sqlite_path: "",
            }))
            .extract()
            .expect("focused Agent extraction");

        config.validate().expect("focused Agent validation");
    }

    #[test]
    fn production_monitor_modes_validate_only_their_own_secrets() {
        let mut controller =
            MonitorControllerConfig::local().expect("local Monitor controller config");
        controller.runtime.environment = "production".to_string();
        controller.ipc_token = "production-ipc-secret".to_string();
        controller.monitor_agent_token = "production-agent-secret".to_string();
        controller.validate().expect("production controller config");
        controller.monitor_agent_token = "replace-me".to_string();
        assert!(controller.validate().is_err());

        let mut agent = MonitorAgentConfig::local().expect("local Monitor Agent config");
        agent.runtime.environment = "production".to_string();
        agent.monitor_agent_token = "production-agent-secret".to_string();
        agent.validate().expect("production Agent config without IPC or database fields");
        agent.monitor_agent_token = "replace-me".to_string();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn monitor_agent_rejects_empty_or_invalid_remote_controller_urls() {
        let mut agent = MonitorAgentConfig::local().expect("local Monitor Agent config");
        for url in ["", "http://", "ftp://monitor.example"] {
            agent.monitor_controller_url = Some(url.to_string());
            assert!(agent.validate().is_err(), "accepted {url:?}");
        }
        agent.monitor_controller_url = Some("http://monitor.example".to_string());
        agent.validate().expect("valid internal HTTP controller URL");
        agent.monitor_controller_url = Some("https://monitor.example".to_string());
        agent.validate().expect("valid remote controller URL");
        assert_eq!(agent.heartbeat_endpoint(), "https://monitor.example/api/monitor/heartbeat");
    }
}
