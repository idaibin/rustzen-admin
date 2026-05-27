//! Shared runtime configuration helpers for sqlite-first runtime startup.

use figment::{Figment, providers::Env};
use once_cell::sync::Lazy;
use rustzen_runtime::{DEFAULT_FILES_PREFIX, DEFAULT_RUNTIME_ROOT, RuntimeLayout};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default storage backend.
const DEFAULT_STORAGE: &str = "sqlite";

/// Default path to the local SQLite database file.
const DEFAULT_SQLITE_PATH: &str = "./data/rustzen.db";

/// Default host for the HTTP service.
const DEFAULT_APP_HOST: &str = "0.0.0.0";

/// Default port for the HTTP service.
const DEFAULT_APP_PORT: u16 = 8007;

/// Default maximum database pool size.
const DEFAULT_DB_MAX_CONN: u32 = 4;

/// Default minimum database pool size.
const DEFAULT_DB_MIN_CONN: u32 = 1;

/// Default database acquire timeout in seconds.
const DEFAULT_DB_CONN_TIMEOUT: u64 = 10;

/// Default database idle timeout in seconds.
const DEFAULT_DB_IDLE_TIMEOUT: u64 = 600;

/// Default JWT lifetime in seconds.
const DEFAULT_JWT_EXPIRATION: i64 = 3600;

/// Development-only fallback JWT secret.
const DEFAULT_DEV_JWT_SECRET: &str = "rustzen-dev-jwt-secret-change-in-production";

/// Default logging file prefix.
const DEFAULT_LOG_FILE_PREFIX: &str = "server";

/// Default log retention days.
const DEFAULT_LOG_RETENTION_DAYS: u64 = 7;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_storage")]
    pub storage: String,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
    #[serde(default = "default_app_port")]
    pub app_port: u16,
    #[serde(default = "default_app_host")]
    pub app_host: String,
    #[serde(default = "default_db_max_conn")]
    pub db_max_conn: u32,
    #[serde(default = "default_db_min_conn")]
    pub db_min_conn: u32,
    #[serde(default = "default_db_conn_timeout")]
    pub db_conn_timeout: u64,
    #[serde(default = "default_db_idle_timeout")]
    pub db_idle_timeout: u64,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration: i64,
    #[serde(default = "default_runtime_root")]
    pub runtime_root: String,
    #[serde(default = "default_files_prefix")]
    pub files_prefix: String,
    #[serde(default = "default_log_file_prefix")]
    pub log_file_prefix: String,
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u64,
}

/// Global process configuration loaded from `RUSTZEN_*` env.
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config: Config = Figment::new().merge(Env::prefixed("RUSTZEN_")).extract().expect("Failed to load configuration");
    ensure_production_jwt_secret(&config);
    config
});

impl Config {
    pub fn runtime_layout(&self) -> RuntimeLayout {
        RuntimeLayout::new(&self.runtime_root, &self.files_prefix)
    }

    pub fn runtime_root_dir(&self) -> PathBuf {
        self.runtime_layout().runtime_root_dir()
    }

    pub fn web_dist_dir(&self) -> PathBuf {
        self.runtime_layout().web_dist_dir()
    }

    pub fn data_dir(&self) -> PathBuf {
        self.runtime_layout().data_dir()
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime_layout().log_dir()
    }

    pub fn uploads_dir(&self) -> PathBuf {
        self.runtime_layout().uploads_dir()
    }

    pub fn avatars_dir(&self) -> PathBuf {
        self.runtime_layout().avatars_dir()
    }

    pub fn avatars_prefix(&self) -> String {
        self.runtime_layout().avatars_prefix()
    }

    pub fn sqlite_database_path(&self) -> PathBuf {
        self.runtime_layout().resolve_runtime_path(&self.sqlite_path)
    }
}

fn default_storage() -> String {
    DEFAULT_STORAGE.to_string()
}

fn default_sqlite_path() -> String {
    DEFAULT_SQLITE_PATH.to_string()
}

fn default_log_file_prefix() -> String {
    DEFAULT_LOG_FILE_PREFIX.to_string()
}

fn default_log_retention_days() -> u64 {
    DEFAULT_LOG_RETENTION_DAYS
}

fn default_app_port() -> u16 {
    DEFAULT_APP_PORT
}

fn default_app_host() -> String {
    DEFAULT_APP_HOST.to_string()
}

fn default_db_max_conn() -> u32 {
    DEFAULT_DB_MAX_CONN
}

fn default_db_min_conn() -> u32 {
    DEFAULT_DB_MIN_CONN
}

fn default_db_conn_timeout() -> u64 {
    DEFAULT_DB_CONN_TIMEOUT
}

fn default_db_idle_timeout() -> u64 {
    DEFAULT_DB_IDLE_TIMEOUT
}

fn default_jwt_expiration() -> i64 {
    DEFAULT_JWT_EXPIRATION
}

fn default_jwt_secret() -> String {
    DEFAULT_DEV_JWT_SECRET.to_string()
}

fn default_files_prefix() -> String {
    DEFAULT_FILES_PREFIX.to_string()
}

fn default_runtime_root() -> String {
    DEFAULT_RUNTIME_ROOT.to_string()
}

fn ensure_production_jwt_secret(config: &Config) {
    let env = std::env::var("RUSTZEN_ENV").unwrap_or_else(|_| "development".to_string());
    let env = env.to_ascii_lowercase();
    let is_production = env == "production" || env == "prod";
    let uses_dev_default = config.jwt_secret == DEFAULT_DEV_JWT_SECRET;
    let uses_placeholder = config.jwt_secret == "replace-me";
    let is_empty = config.jwt_secret.trim().is_empty();

    assert!(
        !(is_production && (uses_dev_default || uses_placeholder || is_empty)),
        "RUSTZEN_JWT_SECRET must be explicitly set in production and cannot use default or placeholder values"
    );
}

#[cfg(test)]
mod tests {
    use super::{Config, default_runtime_root};
    use std::env;
    use std::path::PathBuf;
    use rustzen_runtime::resolve_path_with_runtime_root;

    #[test]
    fn runtime_root_default_uses_hidden_dev_dir() {
        assert_eq!(default_runtime_root(), ".rustzen-admin");
    }

    #[test]
    fn runtime_root_derives_standard_runtime_paths() {
        let config = Config {
            storage: "sqlite".to_string(),
            sqlite_path: "./data/rustzen.db".to_string(),
            app_port: 8007,
            app_host: "0.0.0.0".to_string(),
            db_max_conn: 4,
            db_min_conn: 1,
            db_conn_timeout: 10,
            db_idle_timeout: 600,
            jwt_secret: "secret".to_string(),
            jwt_expiration: 3600,
            runtime_root: ".rustzen-admin".to_string(),
            files_prefix: "/resources".to_string(),
            log_file_prefix: "server".to_string(),
            log_retention_days: 7,
        };

        assert_eq!(config.web_dist_dir(), PathBuf::from(".rustzen-admin/web/dist"));
        assert_eq!(config.data_dir(), PathBuf::from(".rustzen-admin/data"));
        assert_eq!(config.log_dir(), PathBuf::from(".rustzen-admin/logs"));
        assert_eq!(config.uploads_dir(), PathBuf::from(".rustzen-admin/data/uploads"));
        assert_eq!(config.avatars_dir(), PathBuf::from(".rustzen-admin/data/avatars"));
    }

    #[test]
    fn sqlite_path_is_relative_to_runtime_root_when_not_absolute() {
        let cwd = env::current_dir().expect("cwd");
        let config = Config {
            storage: "sqlite".to_string(),
            sqlite_path: "./data/rustzen.db".to_string(),
            app_port: 8007,
            app_host: "0.0.0.0".to_string(),
            db_max_conn: 4,
            db_min_conn: 1,
            db_conn_timeout: 10,
            db_idle_timeout: 600,
            jwt_secret: "secret".to_string(),
            jwt_expiration: 3600,
            runtime_root: ".rustzen-admin".to_string(),
            files_prefix: "/resources".to_string(),
            log_file_prefix: "server".to_string(),
            log_retention_days: 7,
        };

        let expected = resolve_path_with_runtime_root(".rustzen-admin", "./data/rustzen.db");
        assert_eq!(config.sqlite_database_path(), expected);
        assert!(config.sqlite_database_path().is_absolute());
        assert_eq!(config.sqlite_database_path(), cwd.join(".rustzen-admin/data/rustzen.db"));
    }
}
