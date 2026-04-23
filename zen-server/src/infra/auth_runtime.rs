use crate::{
    common::error::ServiceError,
    infra::{config::CONFIG, permission::PermissionService},
};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use rustzen_core::{
    auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec},
    error::CoreError,
};

static JWT_CODEC: Lazy<JwtCodec> =
    Lazy::new(|| JwtCodec::new(CONFIG.jwt_secret.clone(), CONFIG.jwt_expiration));

pub fn jwt_codec() -> JwtCodec {
    JWT_CODEC.clone()
}

#[derive(Debug, Clone, Copy)]
pub struct ServerAuthContextLoader;

#[async_trait]
impl AuthContextLoader for ServerAuthContextLoader {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
        PermissionService::load_current_user(claims.user_id, &claims.username).map_err(|error| {
            match error {
                ServiceError::InvalidToken => CoreError::InvalidToken,
                _ => CoreError::MissingAuthContext,
            }
        })
    }
}
