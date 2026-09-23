use std::{path::PathBuf, time::Duration};

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
use figment::{Figment, providers::Env};
use rustzen_runtime::{DEFAULT_FILES_PREFIX, DEFAULT_RUNTIME_ROOT, RuntimeLayout};
use serde::Deserialize;
#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
use serde::de::DeserializeOwned;

pub(crate) const DEFAULT_DB_MAX_CONN: u32 = 8;
pub(crate) const DEFAULT_DB_MIN_CONN: u32 = 1;
pub(crate) const DEFAULT_DB_CONN_TIMEOUT: u64 = 10;
pub(crate) const DEFAULT_DB_IDLE_TIMEOUT: u64 = 600;
#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "reports"
))]
pub(crate) const DEFAULT_IPC_TOKEN: &str = "rustzen-dev-ipc-token-change-in-production";
#[cfg(any(feature = "monitor-controller", feature = "monitor-agent"))]
pub(crate) const DEFAULT_MONITOR_AGENT_TOKEN: &str =
    "rustzen-dev-monitor-agent-token-change-in-production";
#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) const RELEASE_SECRET_PLACEHOLDER: &str = "replace-me";

const DEFAULT_ENVIRONMENT: &str = "development";
const DEFAULT_TIMEZONE: &str = "UTC";

#[derive(Debug)]
pub enum ConfigError {
    Dotenv(dotenvy::Error),
    Extract(Box<figment::Error>),
    Empty(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dotenv(error) => write!(formatter, "failed to load .env: {error}"),
            Self::Extract(error) => write!(formatter, "failed to load configuration: {error}"),
            Self::Empty(name) => write!(formatter, "{name} must not be empty"),
            Self::Invalid(name) => write!(formatter, "{name} is invalid"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dotenv(error) => Some(error),
            Self::Extract(error) => Some(error.as_ref()),
            Self::Empty(_) | Self::Invalid(_) => None,
        }
    }
}

impl From<figment::Error> for ConfigError {
    fn from(error: figment::Error) -> Self {
        Self::Extract(Box::new(error))
    }
}

pub fn load_dotenv_if_present() -> Result<(), ConfigError> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ConfigError::Dotenv(error)),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    #[serde(rename = "env", default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_runtime_root")]
    pub runtime_root: String,
    #[serde(default)]
    pub timezone: Option<String>,
}

impl RuntimeConfig {
    pub fn layout(&self) -> RuntimeLayout {
        RuntimeLayout::new(&self.runtime_root, DEFAULT_FILES_PREFIX)
    }

    pub fn runtime_root_dir(&self) -> PathBuf {
        self.layout().runtime_root_dir()
    }

    pub fn web_dist_dir(&self) -> PathBuf {
        self.layout().web_dist_dir()
    }

    pub fn data_dir(&self) -> PathBuf {
        self.layout().data_dir()
    }

    pub fn log_dir(&self) -> PathBuf {
        self.layout().log_dir()
    }

    pub fn uploads_dir(&self) -> PathBuf {
        self.layout().uploads_dir()
    }

    pub fn avatars_dir(&self) -> PathBuf {
        self.layout().avatars_dir()
    }

    pub fn files_prefix(&self) -> &'static str {
        DEFAULT_FILES_PREFIX
    }

    pub fn avatars_prefix(&self) -> String {
        self.layout().avatars_prefix()
    }

    pub fn timezone(&self) -> &str {
        self.timezone.as_deref().unwrap_or(DEFAULT_TIMEZONE)
    }

    pub fn resolve_path(&self, value: &str) -> PathBuf {
        self.layout().resolve_runtime_path(value)
    }

    #[cfg(any(
        feature = "admin",
        feature = "admin-monitor",
        feature = "admin-insights",
        feature = "insights",
        feature = "monitor-controller",
        feature = "monitor-agent",
        feature = "reports"
    ))]
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        ensure_required_non_empty("RUSTZEN_ENV", &self.environment)?;
        if !matches!(
            self.environment.trim().to_ascii_lowercase().as_str(),
            "development" | "dev" | "production" | "prod"
        ) {
            return Err(ConfigError::Invalid("RUSTZEN_ENV"));
        }
        ensure_required_non_empty("RUSTZEN_RUNTIME_ROOT", &self.runtime_root)?;
        ensure_optional_non_empty("RUSTZEN_TIMEZONE", self.timezone.as_deref())
    }

    pub fn requires_production_secrets(&self) -> bool {
        matches!(self.environment.trim().to_ascii_lowercase().as_str(), "production" | "prod")
            || self.runtime_root.trim() == "."
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub db_max_conn: Option<u32>,
    #[serde(default)]
    pub db_min_conn: Option<u32>,
    #[serde(default)]
    pub db_conn_timeout: Option<u64>,
    #[serde(default)]
    pub db_idle_timeout: Option<u64>,
}

impl DatabaseConfig {
    pub fn max_connections(&self) -> u32 {
        self.db_max_conn.unwrap_or(DEFAULT_DB_MAX_CONN)
    }

    pub fn min_connections(&self) -> u32 {
        self.db_min_conn.unwrap_or(DEFAULT_DB_MIN_CONN)
    }

    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.db_conn_timeout.unwrap_or(DEFAULT_DB_CONN_TIMEOUT))
    }

    pub fn idle_timeout(&self) -> Option<Duration> {
        Some(Duration::from_secs(self.db_idle_timeout.unwrap_or(DEFAULT_DB_IDLE_TIMEOUT)))
    }
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) fn load<T: DeserializeOwned>() -> Result<T, ConfigError> {
    extract(Figment::new().merge(Env::prefixed("RUSTZEN_")))
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) fn local<T: DeserializeOwned>() -> Result<T, ConfigError> {
    extract(Figment::new())
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
fn extract<T: DeserializeOwned>(figment: Figment) -> Result<T, ConfigError> {
    figment.extract().map_err(ConfigError::from)
}

pub(crate) fn default_environment() -> String {
    DEFAULT_ENVIRONMENT.to_string()
}

pub(crate) fn default_runtime_root() -> String {
    DEFAULT_RUNTIME_ROOT.to_string()
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "reports"
))]
pub(crate) fn default_ipc_token() -> String {
    DEFAULT_IPC_TOKEN.to_string()
}

#[cfg(any(feature = "monitor-controller", feature = "monitor-agent"))]
pub(crate) fn default_monitor_agent_token() -> String {
    DEFAULT_MONITOR_AGENT_TOKEN.to_string()
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) fn ensure_required_non_empty(
    name: &'static str,
    value: &str,
) -> Result<(), ConfigError> {
    if value.trim().is_empty() { Err(ConfigError::Empty(name)) } else { Ok(()) }
}

/// Tokens placed in a systemd EnvironmentFile must be unquoted single-line
/// values. Keep the alphabet deliberately narrow so the same validation is
/// applied by installer and Agent runtime.
pub fn valid_monitor_agent_token(value: &str) -> bool {
    value.len() >= 16
        && !matches!(value, "placeholder" | "changeme" | "default" | "replace-me")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'~'))
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) fn ensure_optional_non_empty(
    name: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    match value {
        Some(value) => ensure_required_non_empty(name, value),
        None => Ok(()),
    }
}

#[cfg(any(
    feature = "admin",
    feature = "admin-monitor",
    feature = "admin-insights",
    feature = "insights",
    feature = "monitor-controller",
    feature = "monitor-agent",
    feature = "reports"
))]
pub(crate) fn ensure_production_secret(
    runtime: &RuntimeConfig,
    name: &'static str,
    value: &str,
    development_default: &str,
) -> Result<(), ConfigError> {
    ensure_required_non_empty(name, value)?;
    if runtime.requires_production_secrets()
        && (value == development_default || value == RELEASE_SECRET_PLACEHOLDER)
    {
        return Err(ConfigError::Invalid(name));
    }
    Ok(())
}

#[cfg(feature = "monitor-agent")]
pub(crate) fn ensure_http_url(name: &'static str, value: Option<&str>) -> Result<(), ConfigError> {
    ensure_optional_non_empty(name, value)?;
    if let Some(value) = value {
        canonical_monitor_endpoint(value).map_err(|_| ConfigError::Invalid(name))?;
    }
    Ok(())
}

/// Canonical controller endpoint used by both the installer and the Agent.
/// Public endpoints require HTTPS; loopback endpoints may use HTTP for local development.
#[cfg(feature = "monitor-agent")]
pub fn canonical_monitor_endpoint(value: &str) -> Result<String, ConfigError> {
    let mut parsed = url::Url::parse(value.trim())
        .map_err(|_| ConfigError::Invalid("RUSTZEN_MONITOR_CONTROLLER_URL"))?;
    let local = matches!(parsed.host_str(), Some("127.0.0.1") | Some("::1"));
    if parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || (!local && parsed.scheme() != "https")
        || (local && !matches!(parsed.scheme(), "http" | "https"))
    {
        return Err(ConfigError::Invalid("RUSTZEN_MONITOR_CONTROLLER_URL"));
    }
    if parsed.path() == "/" {
        parsed.set_path("");
    }
    Ok(parsed.to_string().trim_end_matches('/').to_owned())
}

#[cfg(test)]
mod tests {
    use figment::{Figment, providers::Serialized};
    use serde::Serialize;

    use super::DatabaseConfig;

    #[derive(Serialize)]
    struct NumericOverrides {
        db_idle_timeout: u64,
    }

    #[derive(Serialize)]
    struct InvalidNumericOverride<'a> {
        db_idle_timeout: &'a str,
    }

    #[test]
    fn absent_numeric_overrides_remain_none_and_use_code_defaults() {
        let config: DatabaseConfig = Figment::new().extract().expect("database defaults");

        assert_eq!(config.db_max_conn, None);
        assert_eq!(config.db_min_conn, None);
        assert_eq!(config.db_conn_timeout, None);
        assert_eq!(config.db_idle_timeout, None);
        assert_eq!(config.max_connections(), 8);
        assert_eq!(config.idle_timeout(), Some(std::time::Duration::from_secs(600)));
    }

    #[test]
    fn explicit_zero_numeric_override_is_not_treated_as_absent() {
        let config: DatabaseConfig = Figment::new()
            .merge(Serialized::defaults(NumericOverrides { db_idle_timeout: 0 }))
            .extract()
            .expect("explicit zero");

        assert_eq!(config.db_idle_timeout, Some(0));
        assert_eq!(config.idle_timeout(), Some(std::time::Duration::ZERO));
    }

    #[test]
    fn empty_or_malformed_numeric_override_is_an_error() {
        for value in ["", "not-a-number"] {
            let result = Figment::new()
                .merge(Serialized::defaults(InvalidNumericOverride { db_idle_timeout: value }))
                .extract::<DatabaseConfig>();
            assert!(result.is_err(), "accepted invalid numeric value {value:?}");
        }
    }
}
