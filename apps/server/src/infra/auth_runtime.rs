use crate::{
    common::error::ServiceError,
    features::auth::{repo::AuthRepository, types::AuthUserRow},
    infra::{config::CONFIG, permission::PermissionService},
};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use rustzen_core::{
    auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec},
    error::CoreError,
};

static JWT_CODEC: Lazy<JwtCodec> =
    Lazy::new(|| JwtCodec::new(CONFIG.jwt_secret.clone(), CONFIG.jwt_expiration));

pub fn jwt_codec() -> JwtCodec {
    JWT_CODEC.clone()
}

#[derive(Debug, Clone)]
pub struct ServerAuthContextLoader {
    pool: SqlitePool,
}

#[async_trait]
impl AuthContextLoader for ServerAuthContextLoader {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
        if let Ok(current_user) = PermissionService::load_current_user(claims.user_id, &claims.username) {
            return Ok(current_user);
        }

        Self::load_from_db(&self.pool, claims)
            .await
            .map_err(|error| match error {
                ServiceError::InvalidToken => CoreError::InvalidToken,
                _ => CoreError::MissingAuthContext,
            })
    }
}

impl ServerAuthContextLoader {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn load_from_db(
        pool: &SqlitePool,
        claims: &AuthClaims,
    ) -> Result<CurrentUser, ServiceError> {
        let user = AuthRepository::find_user_by_id(pool, claims.user_id).await?;
        let AuthUserRow { id, username, is_system, .. } = user.ok_or(ServiceError::InvalidToken)?;

        let permissions = if is_system {
            AuthRepository::get_all_permissions(pool).await?
        } else {
            AuthRepository::get_user_permissions(pool, claims.user_id).await?
        };
        PermissionService::cache_user_permissions(id, &permissions);

        Ok(CurrentUser::new(id, username, permissions, is_system))
    }
}
