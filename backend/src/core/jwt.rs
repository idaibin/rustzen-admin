use once_cell::sync::Lazy;
use std::env;

pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
}

pub static JWT_CONFIG: Lazy<JwtConfig> = Lazy::new(|| JwtConfig {
    secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
    expiration: env::var("JWT_EXPIRATION")
        .unwrap_or_else(|_| "7200".to_string())
        .parse::<i64>()
        .expect("JWT_EXPIRATION must be a number"),
});
