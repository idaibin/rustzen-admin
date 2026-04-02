use figment::{
    Figment,
    providers::Env,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub app_port: u16,
    pub app_host: String,
    pub db_max_conn: u32,
    pub db_min_conn: u32,
    pub db_conn_timeout: u64,
    pub db_idle_timeout: u64,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub web_dist: String,
    pub upload_dir: String,
    pub avatar_dir: String,
    pub upload_public_prefix: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config: Config = Figment::new()
        .merge(Env::prefixed("RUSTZEN_"))
        .extract()
        .expect("Failed to load configuration");

    tracing::info!(
        app_host = %config.app_host,
        app_port = config.app_port,
        db_max_conn = config.db_max_conn,
        db_min_conn = config.db_min_conn,
        db_conn_timeout = config.db_conn_timeout,
        db_idle_timeout = config.db_idle_timeout,
        jwt_expiration = config.jwt_expiration,
        web_dist = %config.web_dist,
        upload_dir = %config.upload_dir,
        avatar_dir = %config.avatar_dir,
        upload_public_prefix = %config.upload_public_prefix,
        "Configuration loaded"
    );
    config
});
