use figment::{Figment, providers::Env};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEV_RUNTIME_ROOT: &str = ".rustzen-admin";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
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

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Figment::new().merge(Env::prefixed("RUSTZEN_")).extract().expect("Failed to load configuration")
});

impl Config {
    pub fn runtime_root_dir(&self) -> PathBuf {
        PathBuf::from(&self.runtime_root)
    }

    pub fn web_dist_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("web").join("dist")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("data")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("logs")
    }

    pub fn uploads_dir(&self) -> PathBuf {
        self.data_dir().join("uploads")
    }

    pub fn avatars_dir(&self) -> PathBuf {
        self.data_dir().join("avatars")
    }

    pub fn avatars_prefix(&self) -> String {
        format!("{}/avatars", self.files_prefix.trim_end_matches('/'))
    }
}

fn default_log_file_prefix() -> String {
    "server".to_string()
}

fn default_log_retention_days() -> u64 {
    7
}

fn default_app_port() -> u16 {
    8007
}

fn default_app_host() -> String {
    "0.0.0.0".to_string()
}

fn default_db_max_conn() -> u32 {
    4
}

fn default_db_min_conn() -> u32 {
    1
}

fn default_db_conn_timeout() -> u64 {
    10
}

fn default_db_idle_timeout() -> u64 {
    600
}

fn default_jwt_expiration() -> i64 {
    3600
}

fn default_files_prefix() -> String {
    "/resources".to_string()
}

fn default_runtime_root() -> String {
    DEV_RUNTIME_ROOT.to_string()
}

#[cfg(test)]
mod tests {
    use super::{Config, default_runtime_root};
    use std::path::PathBuf;

    #[test]
    fn runtime_root_default_uses_hidden_dev_dir() {
        assert_eq!(default_runtime_root(), ".rustzen-admin");
    }

    #[test]
    fn runtime_root_derives_standard_runtime_paths() {
        let config = Config {
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
}
