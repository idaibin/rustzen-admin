use std::path::PathBuf;

use serde::Deserialize;

use crate::shared::{
    ConfigError, DatabaseConfig, RuntimeConfig, default_ipc_token, ensure_optional_non_empty,
    ensure_production_secret, ensure_required_non_empty, load, local,
};

const DEFAULT_ADMIN_HOST: &str = "0.0.0.0";
const DEFAULT_ADMIN_PORT: u16 = 9801;
const DEFAULT_INTERNAL_HOST: &str = "127.0.0.1";
const DEFAULT_MONITOR_PORT: u16 = 9802;
#[cfg(feature = "admin")]
const DEFAULT_INSIGHTS_PORT: u16 = 9803;
#[cfg(feature = "admin")]
const DEFAULT_REPORTS_PORT: u16 = 9804;
const DEFAULT_ADMIN_SQLITE_PATH: &str = "./data/db/admin.db";
#[cfg(feature = "admin")]
const DEFAULT_MONITOR_SQLITE_PATH: &str = "./data/db/monitor.db";
#[cfg(feature = "admin")]
const DEFAULT_INSIGHTS_SQLITE_PATH: &str = "./data/db/insights.db";
#[cfg(feature = "admin")]
const DEFAULT_REPORTS_SQLITE_PATH: &str = "./data/reports/db/reports.db";
const DEFAULT_JWT_EXPIRATION: i64 = 7200;
#[cfg(feature = "admin")]
const DEFAULT_TASK_RUN_TIMEOUT_SECONDS: u64 = 1800;
const DEFAULT_DEV_JWT_SECRET: &str = "rustzen-dev-jwt-secret-change-in-production";
const RELEASE_JWT_SECRET_PLACEHOLDER: &str = "rustzen-admin-release-{version}";
const RELEASE_JWT_SECRET_PREFIX: &str = "rustzen-admin-release-";
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_MESSAGE_LIMIT: u64 = 100_000;
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_ROW_LIMIT: u64 = 1_000_000;
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_CHARGED_BYTES_LIMIT: u64 = 512 * 1024 * 1024;
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_FREE_SPACE_RESERVE_BYTES: u64 = 128 * 1024 * 1024;
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_INGRESS_PORT: u16 = 9811;
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_EVENT_KEY_ID: &str = "local-v1";
#[cfg(feature = "notifications")]
const DEFAULT_NOTIFICATION_EVENT_KEY: &str = "rustzen-local-notification-key-change-me";
#[cfg(feature = "reports-notifications")]
const DEFAULT_REPORTS_NOTIFICATION_EVENT_KEY_ID: &str = "reports-local-v1";
#[cfg(feature = "reports-notifications")]
const DEFAULT_REPORTS_NOTIFICATION_EVENT_KEY: &str =
    "rustzen-local-reports-notification-key-change-me";

#[derive(Debug, Clone, Deserialize)]
pub struct AdminConfig {
    #[serde(flatten)]
    pub runtime: RuntimeConfig,
    #[serde(flatten)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub admin_host: Option<String>,
    #[serde(default)]
    pub admin_port: Option<u16>,
    #[serde(default)]
    pub internal_host: Option<String>,
    #[serde(default)]
    pub monitor_port: Option<u16>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub insights_port: Option<u16>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub reports_port: Option<u16>,
    #[serde(default)]
    pub admin_sqlite_path: Option<String>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub monitor_sqlite_path: Option<String>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub insights_sqlite_path: Option<String>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub reports_sqlite_path: Option<String>,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default)]
    pub jwt_expiration: Option<i64>,
    #[serde(default = "default_ipc_token")]
    pub ipc_token: String,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_message_limit: Option<u64>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_recipient_limit: Option<u64>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_receipt_limit: Option<u64>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_charged_bytes_limit: Option<u64>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_free_space_reserve_bytes: Option<u64>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_wal_pressure_frames: Option<u32>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_wal_pressure_observations: Option<u32>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_ingress_port: Option<u16>,
    #[cfg(feature = "notifications")]
    #[serde(default = "default_notification_event_key_id")]
    pub notification_event_key_id: String,
    #[cfg(feature = "notifications")]
    #[serde(default = "default_notification_event_key")]
    pub notification_event_key: String,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_previous_event_key_id: Option<String>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_previous_event_key: Option<String>,
    #[cfg(feature = "notifications")]
    #[serde(default)]
    pub notification_previous_event_key_expires_at: Option<i64>,
    #[cfg(feature = "reports-notifications")]
    #[serde(default = "default_reports_notification_event_key_id")]
    pub reports_notification_event_key_id: String,
    #[cfg(feature = "reports-notifications")]
    #[serde(default = "default_reports_notification_event_key")]
    pub reports_notification_event_key: String,
    #[cfg(feature = "reports-notifications")]
    #[serde(default)]
    pub reports_notification_previous_event_key_id: Option<String>,
    #[cfg(feature = "reports-notifications")]
    #[serde(default)]
    pub reports_notification_previous_event_key: Option<String>,
    #[cfg(feature = "reports-notifications")]
    #[serde(default)]
    pub reports_notification_previous_event_key_expires_at: Option<i64>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub task_run_timeout_seconds: Option<u64>,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub deploy_signature_required: bool,
    #[cfg(feature = "admin")]
    #[serde(default)]
    pub deploy_verify_key: Option<String>,
}

impl AdminConfig {
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

    pub fn admin_host(&self) -> &str {
        self.admin_host.as_deref().unwrap_or(DEFAULT_ADMIN_HOST)
    }

    pub fn admin_port(&self) -> u16 {
        self.admin_port.unwrap_or(DEFAULT_ADMIN_PORT)
    }

    pub fn internal_host(&self) -> &str {
        self.internal_host.as_deref().unwrap_or(DEFAULT_INTERNAL_HOST)
    }

    pub fn monitor_port(&self) -> u16 {
        self.monitor_port.unwrap_or(DEFAULT_MONITOR_PORT)
    }

    #[cfg(feature = "admin")]
    pub fn insights_port(&self) -> u16 {
        self.insights_port.unwrap_or(DEFAULT_INSIGHTS_PORT)
    }

    #[cfg(feature = "admin")]
    pub fn reports_port(&self) -> u16 {
        self.reports_port.unwrap_or(DEFAULT_REPORTS_PORT)
    }

    pub fn jwt_expiration(&self) -> i64 {
        self.jwt_expiration.unwrap_or(DEFAULT_JWT_EXPIRATION)
    }

    #[cfg(feature = "admin")]
    pub fn task_run_timeout_seconds(&self) -> u64 {
        self.task_run_timeout_seconds.unwrap_or(DEFAULT_TASK_RUN_TIMEOUT_SECONDS)
    }

    #[cfg(feature = "admin")]
    pub fn runtime_root_dir(&self) -> PathBuf {
        self.runtime.runtime_root_dir()
    }

    pub fn web_dist_dir(&self) -> PathBuf {
        self.runtime.web_dist_dir()
    }

    #[cfg(feature = "admin")]
    pub fn data_dir(&self) -> PathBuf {
        self.runtime.data_dir()
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime.log_dir()
    }

    #[cfg(feature = "admin")]
    pub fn uploads_dir(&self) -> PathBuf {
        self.runtime.uploads_dir()
    }

    #[cfg(feature = "admin")]
    pub fn avatars_dir(&self) -> PathBuf {
        self.runtime.avatars_dir()
    }

    #[cfg(feature = "admin")]
    pub fn files_prefix(&self) -> &'static str {
        self.runtime.files_prefix()
    }

    #[cfg(feature = "admin")]
    pub fn avatars_prefix(&self) -> String {
        self.runtime.avatars_prefix()
    }

    pub fn timezone(&self) -> &str {
        self.runtime.timezone()
    }

    pub fn admin_database_path(&self) -> PathBuf {
        self.database_path(self.admin_sqlite_path.as_deref(), DEFAULT_ADMIN_SQLITE_PATH)
    }

    #[cfg(feature = "admin")]
    pub fn monitor_database_path(&self) -> PathBuf {
        self.database_path(self.monitor_sqlite_path.as_deref(), DEFAULT_MONITOR_SQLITE_PATH)
    }

    #[cfg(feature = "admin")]
    pub fn insights_database_path(&self) -> PathBuf {
        self.database_path(self.insights_sqlite_path.as_deref(), DEFAULT_INSIGHTS_SQLITE_PATH)
    }

    #[cfg(feature = "admin")]
    pub fn reports_database_path(&self) -> PathBuf {
        self.database_path(self.reports_sqlite_path.as_deref(), DEFAULT_REPORTS_SQLITE_PATH)
    }

    pub fn monitor_base_url(&self) -> String {
        format!("http://{}:{}", self.internal_host(), self.monitor_port())
    }

    #[cfg(feature = "notifications")]
    pub fn notification_limits(&self) -> (u64, u64, u64, u64) {
        (
            self.notification_message_limit.unwrap_or(DEFAULT_NOTIFICATION_MESSAGE_LIMIT),
            self.notification_recipient_limit.unwrap_or(DEFAULT_NOTIFICATION_ROW_LIMIT),
            self.notification_receipt_limit.unwrap_or(DEFAULT_NOTIFICATION_ROW_LIMIT),
            self.notification_charged_bytes_limit
                .unwrap_or(DEFAULT_NOTIFICATION_CHARGED_BYTES_LIMIT),
        )
    }

    #[cfg(feature = "notifications")]
    pub fn notification_pressure_limits(&self) -> (u64, u32, u32) {
        (
            self.notification_free_space_reserve_bytes
                .unwrap_or(DEFAULT_NOTIFICATION_FREE_SPACE_RESERVE_BYTES),
            self.notification_wal_pressure_frames.unwrap_or(1024),
            self.notification_wal_pressure_observations.unwrap_or(3),
        )
    }

    #[cfg(feature = "notifications")]
    pub fn notification_ingress_address(&self) -> String {
        format!(
            "{}:{}",
            self.internal_host(),
            self.notification_ingress_port.unwrap_or(DEFAULT_NOTIFICATION_INGRESS_PORT)
        )
    }

    #[cfg(feature = "notifications")]
    pub fn notification_event_keys(&self) -> (&str, &str, Option<(&str, &str, i64)>) {
        (
            &self.notification_event_key_id,
            &self.notification_event_key,
            self.notification_previous_event_key_id
                .as_deref()
                .zip(self.notification_previous_event_key.as_deref())
                .zip(self.notification_previous_event_key_expires_at)
                .map(|((id, key), expires)| (id, key, expires)),
        )
    }

    #[cfg(feature = "reports-notifications")]
    pub fn reports_notification_event_keys(&self) -> (&str, &str, Option<(&str, &str, i64)>) {
        (
            &self.reports_notification_event_key_id,
            &self.reports_notification_event_key,
            self.reports_notification_previous_event_key_id
                .as_deref()
                .zip(self.reports_notification_previous_event_key.as_deref())
                .zip(self.reports_notification_previous_event_key_expires_at)
                .map(|((id, key), expires)| (id, key, expires)),
        )
    }

    #[cfg(feature = "admin")]
    pub fn insights_base_url(&self) -> String {
        format!("http://{}:{}", self.internal_host(), self.insights_port())
    }

    #[cfg(feature = "admin")]
    pub fn reports_base_url(&self) -> String {
        format!("http://{}:{}", self.internal_host(), self.reports_port())
    }

    fn database_path(&self, configured: Option<&str>, default: &str) -> PathBuf {
        self.runtime.resolve_path(configured.unwrap_or(default))
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.runtime.validate()?;
        for (name, value) in [
            ("RUSTZEN_ADMIN_HOST", self.admin_host.as_deref()),
            ("RUSTZEN_INTERNAL_HOST", self.internal_host.as_deref()),
            ("RUSTZEN_ADMIN_SQLITE_PATH", self.admin_sqlite_path.as_deref()),
        ] {
            ensure_optional_non_empty(name, value)?;
        }
        #[cfg(feature = "admin")]
        for (name, value) in [
            ("RUSTZEN_MONITOR_SQLITE_PATH", self.monitor_sqlite_path.as_deref()),
            ("RUSTZEN_INSIGHTS_SQLITE_PATH", self.insights_sqlite_path.as_deref()),
            ("RUSTZEN_REPORTS_SQLITE_PATH", self.reports_sqlite_path.as_deref()),
        ] {
            ensure_optional_non_empty(name, value)?;
        }
        ensure_required_non_empty("RUSTZEN_JWT_SECRET", &self.jwt_secret)?;
        #[cfg(feature = "notifications")]
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| {
                    ConfigError::Invalid("RUSTZEN_NOTIFICATION_PREVIOUS_EVENT_KEY_EXPIRES_AT")
                })?
                .as_secs() as i64;
            let (messages, recipients, receipts, bytes) = self.notification_limits();
            let (reserve, frames, observations) = self.notification_pressure_limits();
            if [messages, recipients, receipts, bytes].contains(&0)
                || reserve == 0
                || frames == 0
                || observations == 0
            {
                return Err(ConfigError::Invalid("RUSTZEN_NOTIFICATION_LIMITS"));
            }
            if !rustzen_ipc::valid_notification_key_id(&self.notification_event_key_id)
                || self.notification_event_key.len() < 32
                || self.notification_event_key == self.ipc_token
                || self.notification_event_key == self.jwt_secret
                || [
                    self.notification_previous_event_key_id.is_some(),
                    self.notification_previous_event_key.is_some(),
                    self.notification_previous_event_key_expires_at.is_some(),
                ]
                .windows(2)
                .any(|pair| pair[0] != pair[1])
                || self.notification_previous_event_key.as_deref().is_some_and(|key| key.len() < 32)
                || self.notification_previous_event_key.as_deref().is_some_and(|key| {
                    key == self.notification_event_key
                        || key == self.ipc_token
                        || key == self.jwt_secret
                })
                || self.notification_previous_event_key_id.as_deref()
                    == Some(self.notification_event_key_id.as_str())
                || self
                    .notification_previous_event_key_id
                    .as_deref()
                    .is_some_and(|id| !rustzen_ipc::valid_notification_key_id(id))
                || self
                    .notification_previous_event_key_expires_at
                    .is_some_and(|expires| expires <= now || expires > now + 120)
            {
                return Err(ConfigError::Invalid("RUSTZEN_NOTIFICATION_EVENT_KEY"));
            }
            ensure_production_secret(
                &self.runtime,
                "RUSTZEN_NOTIFICATION_EVENT_KEY",
                &self.notification_event_key,
                DEFAULT_NOTIFICATION_EVENT_KEY,
            )?;
            #[cfg(feature = "reports-notifications")]
            {
                if !rustzen_ipc::valid_notification_key_id(&self.reports_notification_event_key_id)
                    || self.reports_notification_event_key.len() < 32
                    || self.reports_notification_event_key == self.notification_event_key
                    || self.notification_previous_event_key.as_deref()
                        == Some(self.reports_notification_event_key.as_str())
                    || self.reports_notification_event_key == self.ipc_token
                    || self.reports_notification_event_key == self.jwt_secret
                    || [
                        self.reports_notification_previous_event_key_id.is_some(),
                        self.reports_notification_previous_event_key.is_some(),
                        self.reports_notification_previous_event_key_expires_at.is_some(),
                    ]
                    .windows(2)
                    .any(|pair| pair[0] != pair[1])
                    || self
                        .reports_notification_previous_event_key_id
                        .as_deref()
                        .is_some_and(|id| !rustzen_ipc::valid_notification_key_id(id))
                    || self.reports_notification_previous_event_key_id.as_deref()
                        == Some(self.reports_notification_event_key_id.as_str())
                    || self.reports_notification_previous_event_key.as_deref().is_some_and(|key| {
                        key.len() < 32
                            || key == self.reports_notification_event_key
                            || key == self.notification_event_key
                            || self.notification_previous_event_key.as_deref() == Some(key)
                            || key == self.ipc_token
                            || key == self.jwt_secret
                    })
                    || self
                        .reports_notification_previous_event_key_expires_at
                        .is_some_and(|expires| expires <= now || expires > now + 120)
                {
                    return Err(ConfigError::Invalid("RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY"));
                }
                ensure_production_secret(
                    &self.runtime,
                    "RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY",
                    &self.reports_notification_event_key,
                    DEFAULT_REPORTS_NOTIFICATION_EVENT_KEY,
                )?;
            }
        }
        ensure_production_secret(
            &self.runtime,
            "RUSTZEN_IPC_TOKEN",
            &self.ipc_token,
            crate::shared::DEFAULT_IPC_TOKEN,
        )?;
        if self.runtime.requires_production_secrets()
            && (self.jwt_secret == DEFAULT_DEV_JWT_SECRET
                || self.jwt_secret == crate::shared::RELEASE_SECRET_PLACEHOLDER
                || self.jwt_secret == RELEASE_JWT_SECRET_PLACEHOLDER
                || self.jwt_secret.starts_with(RELEASE_JWT_SECRET_PREFIX))
        {
            return Err(ConfigError::Invalid("RUSTZEN_JWT_SECRET"));
        }
        #[cfg(feature = "admin")]
        ensure_optional_non_empty("RUSTZEN_DEPLOY_VERIFY_KEY", self.deploy_verify_key.as_deref())?;
        #[cfg(feature = "admin")]
        if self.deploy_verify_key.as_deref().is_some_and(|value| {
            let value = value.trim();
            value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err(ConfigError::Invalid("RUSTZEN_DEPLOY_VERIFY_KEY"));
        }
        #[cfg(feature = "admin")]
        if self.runtime.requires_production_secrets() && !self.deploy_signature_required {
            return Err(ConfigError::Invalid("RUSTZEN_DEPLOY_SIGNATURE_REQUIRED"));
        }
        #[cfg(feature = "admin")]
        if self.runtime.requires_production_secrets() && self.deploy_verify_key.is_none() {
            return Err(ConfigError::Invalid("RUSTZEN_DEPLOY_VERIFY_KEY"));
        }
        Ok(())
    }
}

#[cfg(feature = "reports-notifications")]
fn default_reports_notification_event_key_id() -> String {
    DEFAULT_REPORTS_NOTIFICATION_EVENT_KEY_ID.into()
}

#[cfg(feature = "reports-notifications")]
fn default_reports_notification_event_key() -> String {
    DEFAULT_REPORTS_NOTIFICATION_EVENT_KEY.into()
}

#[cfg(all(test, feature = "admin-monitor", not(feature = "admin")))]
mod monitor_distribution_tests {
    use figment::{Figment, providers::Serialized};
    use serde::Serialize;

    use super::AdminConfig;

    #[derive(Serialize)]
    struct FullOnlySettings<'a> {
        insights_port: &'a str,
        reports_port: &'a str,
        monitor_sqlite_path: &'a str,
        task_run_timeout_seconds: &'a str,
        deploy_verify_key: &'a str,
    }

    #[test]
    fn monitor_admin_extracts_only_access_and_monitor_host_settings() {
        let config: AdminConfig = Figment::new()
            .merge(Serialized::defaults(FullOnlySettings {
                insights_port: "not-a-number",
                reports_port: "not-a-number",
                monitor_sqlite_path: "",
                task_run_timeout_seconds: "not-a-number",
                deploy_verify_key: "invalid",
            }))
            .extract()
            .expect("full-only settings are not part of minimal config");
        assert_eq!(config.admin_port(), 9801);
        assert_eq!(config.monitor_base_url(), "http://127.0.0.1:9802");
    }

    #[test]
    fn monitor_admin_requires_production_authentication_secrets() {
        let mut config = AdminConfig::local().expect("local minimal Admin config");
        config.runtime.environment = "production".to_string();
        config.jwt_secret = "production-jwt-secret".to_string();
        config.ipc_token = "production-ipc-secret".to_string();
        #[cfg(feature = "notifications")]
        {
            config.notification_event_key = "production-notification-event-secret".to_string();
        }
        config.validate().expect("hardened minimal production config");

        let mut invalid = config.clone();
        invalid.jwt_secret = "replace-me".to_string();
        assert!(invalid.validate().is_err());
        let mut invalid = config;
        invalid.ipc_token = "replace-me".to_string();
        assert!(invalid.validate().is_err());
    }

    #[cfg(feature = "notifications")]
    #[test]
    fn notification_keys_are_not_reused_across_authentication_domains() {
        let base = AdminConfig::local().unwrap();
        for invalid_id in ["bad\nheader".into(), "bad id".into(), "x".repeat(65)] {
            let mut invalid = base.clone();
            invalid.notification_event_key_id = invalid_id;
            assert!(invalid.validate().is_err());
        }
        for reused in [base.ipc_token.clone(), base.jwt_secret.clone()] {
            let mut invalid = base.clone();
            invalid.notification_event_key = reused;
            assert!(invalid.validate().is_err());
        }
        let mut invalid = base;
        invalid.notification_previous_event_key_id = Some("previous".into());
        invalid.notification_previous_event_key = Some(invalid.notification_event_key.clone());
        invalid.notification_previous_event_key_expires_at = Some(1);
        assert!(invalid.validate().is_err());

        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                as i64;
        let mut rotating = AdminConfig::local().unwrap();
        rotating.notification_previous_event_key_id = Some("previous".into());
        rotating.notification_previous_event_key =
            Some("previous-notification-key-0123456789".into());
        rotating.notification_previous_event_key_expires_at = Some(now + 60);
        rotating.validate().unwrap();
        rotating.notification_previous_event_key_id = Some("bad\nheader".into());
        assert!(rotating.validate().is_err());
        rotating.notification_previous_event_key_id = Some("previous".into());
        rotating.notification_previous_event_key_expires_at = Some(now);
        assert!(rotating.validate().is_err());
        rotating.notification_previous_event_key_expires_at = Some(now + 121);
        assert!(rotating.validate().is_err());
    }
}

fn default_jwt_secret() -> String {
    DEFAULT_DEV_JWT_SECRET.to_string()
}

#[cfg(feature = "notifications")]
fn default_notification_event_key_id() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY_ID.into()
}

#[cfg(feature = "notifications")]
fn default_notification_event_key() -> String {
    DEFAULT_NOTIFICATION_EVENT_KEY.into()
}

#[cfg(all(test, feature = "admin"))]
mod tests {
    use super::AdminConfig;

    #[test]
    fn local_admin_config_uses_safe_code_defaults() {
        let config = AdminConfig::local().expect("local Admin config");

        assert_eq!(config.admin_host(), "0.0.0.0");
        assert_eq!(config.admin_port(), 9801);
        assert_eq!(config.internal_host(), "127.0.0.1");
        assert_eq!(config.monitor_base_url(), "http://127.0.0.1:9802");
        assert_eq!(config.database.db_idle_timeout, None);
        assert_eq!(config.runtime.runtime_root, ".rustzen-admin");
    }

    #[test]
    fn production_admin_rejects_every_placeholder_and_disabled_verification() {
        let mut hardened = AdminConfig::local().expect("local Admin config");
        hardened.runtime.environment = "production".to_string();
        hardened.jwt_secret = "production-jwt-secret".to_string();
        hardened.ipc_token = "production-ipc-secret".to_string();
        #[cfg(feature = "notifications")]
        {
            hardened.notification_event_key = "production-notification-event-secret".to_string();
        }
        #[cfg(feature = "reports-notifications")]
        {
            hardened.reports_notification_event_key =
                "production-reports-notification-secret".to_string();
        }
        hardened.deploy_signature_required = true;
        hardened.deploy_verify_key = Some("ab".repeat(32));
        hardened.validate().expect("hardened production config");

        let mut invalid = hardened.clone();
        invalid.jwt_secret = "replace-me".to_string();
        assert!(invalid.validate().is_err());
        let mut invalid = hardened.clone();
        invalid.ipc_token = "replace-me".to_string();
        assert!(invalid.validate().is_err());
        let mut invalid = hardened.clone();
        invalid.deploy_signature_required = false;
        assert!(invalid.validate().is_err());
        let mut invalid = hardened;
        invalid.deploy_verify_key = Some("replace-me".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn explicit_empty_admin_override_and_unknown_environment_are_errors() {
        let mut config = AdminConfig::local().expect("local Admin config");
        config.admin_host = Some("  ".to_string());
        assert!(config.validate().is_err());

        let mut config = AdminConfig::local().expect("local Admin config");
        config.runtime.environment = "staging".to_string();
        assert!(config.validate().is_err());
    }

    #[cfg(feature = "notifications")]
    #[test]
    fn full_admin_rejects_invalid_notification_key_ids() {
        let base = AdminConfig::local().unwrap();
        for id in ["bad\nheader".into(), "bad id".into(), "x".repeat(65)] {
            let mut invalid = base.clone();
            invalid.notification_event_key_id = id;
            assert!(invalid.validate().is_err());
        }
        #[cfg(feature = "reports-notifications")]
        {
            let mut invalid = base.clone();
            invalid.reports_notification_event_key_id = "bad\nheader".into();
            assert!(invalid.validate().is_err());
            let mut invalid = base.clone();
            invalid.reports_notification_event_key = invalid.notification_event_key.clone();
            assert!(invalid.validate().is_err());
        }
    }
}
