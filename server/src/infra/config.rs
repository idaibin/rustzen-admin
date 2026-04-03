use figment::{Figment, providers::Env};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    #[serde(default = "default_web_dist")]
    pub web_dist: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_files_prefix")]
    pub files_prefix: String,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default = "default_log_file_prefix")]
    pub log_file_prefix: String,
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u64,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Figment::new().merge(Env::prefixed("RUSTZEN_")).extract().expect("Failed to load configuration")
});

impl Config {
    pub fn uploads_dir(&self) -> PathBuf {
        PathBuf::from(&self.data_dir).join("uploads")
    }

    pub fn avatars_dir(&self) -> PathBuf {
        PathBuf::from(&self.data_dir).join("avatars")
    }

    pub fn avatars_prefix(&self) -> String {
        format!("{}/avatars", self.files_prefix.trim_end_matches('/'))
    }
}

fn default_log_dir() -> String {
    "logs".to_string()
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
    10
}

fn default_db_min_conn() -> u32 {
    1
}

fn default_db_conn_timeout() -> u64 {
    10
}

fn default_db_idle_timeout() -> u64 {
    0
}

fn default_jwt_expiration() -> i64 {
    3600
}

fn default_web_dist() -> String {
    "web/dist".to_string()
}

fn default_data_dir() -> String {
    "data".to_string()
}

fn default_files_prefix() -> String {
    "/resources".to_string()
}
