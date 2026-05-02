use super::{
    repo::AccountRepository,
    types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
};
use crate::{
    common::error::ServiceError,
    features::auth::{service::AuthService, types::UserInfoResp},
    infra::password::PasswordUtils,
};

use sqlx::PgPool;

/// Account service for current-user profile operations.
pub struct AccountService;

impl AccountService {
    pub async fn update_avatar(
        pool: &PgPool,
        user_id: i64,
        avatar_url: &str,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating avatar for user_id: {}", user_id);
        AccountRepository::update_avatar(pool, user_id, avatar_url).await?;
        tracing::info!("Avatar updated successfully for user_id: {}", user_id);
        Ok(())
    }

    pub async fn update_profile(
        pool: &PgPool,
        user_id: i64,
        request: UpdateAccountProfileRequest,
    ) -> Result<UserInfoResp, ServiceError> {
        tracing::info!("Updating account profile for user_id: {}", user_id);
        if AccountRepository::email_exists_for_other_user(pool, user_id, &request.email).await? {
            return Err(ServiceError::EmailConflict);
        }

        AccountRepository::update_profile(pool, user_id, &request).await?;
        AuthService::get_login_info(pool, user_id).await
    }

    pub async fn change_password(
        pool: &PgPool,
        user_id: i64,
        request: ChangeAccountPasswordRequest,
    ) -> Result<(), ServiceError> {
        tracing::info!("Changing account password for user_id: {}", user_id);
        let current = AccountRepository::find_password_hash_by_id(pool, user_id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;
        let password_hash = Self::build_password_hash(
            &request.current_password,
            &current.password_hash,
            &request.new_password,
            &request.confirm_password,
        )?;

        AccountRepository::update_password(pool, user_id, &password_hash).await
    }

    pub fn build_password_hash(
        current_password: &str,
        current_hash: &str,
        new_password: &str,
        confirm_password: &str,
    ) -> Result<String, ServiceError> {
        if !PasswordUtils::verify_password(current_password, current_hash) {
            return Err(ServiceError::InvalidCurrentPassword);
        }
        if new_password != confirm_password {
            return Err(ServiceError::PasswordConfirmationMismatch);
        }
        PasswordUtils::hash_password(new_password)
    }
}
