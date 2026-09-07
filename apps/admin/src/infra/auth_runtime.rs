use crate::{features::auth::session::SessionRepository, infra::config::CONFIG};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use rustzen_auth::{
    auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec},
    error::CoreError,
};

static JWT_CODEC: Lazy<JwtCodec> = Lazy::new(|| {
    JwtCodec::with_issuer_audience(
        CONFIG.jwt_secret.clone(),
        CONFIG.jwt_expiration(),
        format!("urn:rustzen:admin:{}:{}", CONFIG.admin_host(), CONFIG.admin_port()),
        "rustzen-entry",
    )
});

pub fn jwt_codec() -> JwtCodec {
    JWT_CODEC.clone()
}

#[derive(Debug, Clone)]
pub struct ServerAuthContextLoader {
    pool: sqlx::SqlitePool,
}

#[async_trait]
impl AuthContextLoader for ServerAuthContextLoader {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
        SessionRepository::load_authoritative_user(
            &self.pool,
            claims,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(|_| CoreError::InvalidToken)
    }
}

impl ServerAuthContextLoader {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}
