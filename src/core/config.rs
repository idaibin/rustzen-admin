use figment::{
    Figment,
    providers::{Env, Serialized},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// application port
    pub app_port: u16,
    /// application host
    pub app_host: String,
    /// database URL
    pub db_url: String,
    /// database maximum connection count
    pub db_max_conn: u32,
    /// database minimum connection count
    pub db_min_conn: u32,
    /// database connection timeout
    pub db_conn_timeout: u64,
    /// database idle connection timeout
    pub db_idle_timeout: u64,
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time    
    pub jwt_expiration: i64,
    /// whether to enable web embedding
    pub web_embed_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_port: 8000,
            app_host: "0.0.0.0".into(),
            db_url: "sqlite://rustzen.db".into(),
            db_max_conn: 10,
            db_min_conn: 1,
            db_conn_timeout: 10,
            db_idle_timeout: 0,
            jwt_secret: "rustzen-admin-secret-key".into(),
            jwt_expiration: 60 * 60, // 1 hour
            web_embed_enabled: false,
        }
    }
}
// 全局单例
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("RUSTZEN_"))
        .extract()
        .expect("Failed to load configuration");

    tracing::info!("CONFIG: {:?}", config);
    config
});
