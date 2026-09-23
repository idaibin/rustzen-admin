use super::{
    repo::AccountRepository,
    types::{ChangeAccountPasswordRequest, UpdateAccountProfileRequest},
};
#[cfg(feature = "full")]
use crate::common::files::{remove_avatar_by_url, save_avatar};
use crate::{
    common::error::ServiceError,
    features::auth::{service::AuthService, session::SessionRepository, types::UserInfoResp},
    infra::{password::PasswordUtils, permission::PermissionService},
};

#[cfg(feature = "full")]
use axum::extract::Multipart;
use rustzen_auth::auth::AuthClaims;
use sqlx::SqlitePool;

/// Account service for current-user profile operations.
pub struct AccountService;

impl AccountService {
    #[cfg(feature = "full")]
    pub async fn update_avatar(
        pool: &SqlitePool,
        user_id: i64,
        multipart: &mut Multipart,
        actor: &AuthClaims,
    ) -> Result<String, ServiceError> {
        tracing::info!("Updating avatar for user_id: {}", user_id);
        let avatar_url = save_avatar(multipart).await?;
        if let Err(error) =
            AccountRepository::update_avatar(pool, user_id, &avatar_url, actor).await
        {
            if let Err(remove_error) = remove_avatar_by_url(&avatar_url).await {
                tracing::warn!(
                    avatar_url,
                    error = %remove_error,
                    "Failed to remove avatar after database update failure"
                );
            }
            return Err(error);
        }
        tracing::info!("Avatar updated successfully for user_id: {}", user_id);
        Ok(avatar_url)
    }

    pub async fn update_profile(
        pool: &SqlitePool,
        user_id: i64,
        request: UpdateAccountProfileRequest,
        actor: &AuthClaims,
    ) -> Result<UserInfoResp, ServiceError> {
        tracing::info!("Updating account profile for user_id: {}", user_id);
        if AccountRepository::email_exists_for_other_user(pool, user_id, &request.email).await? {
            return Err(ServiceError::EmailConflict);
        }

        AccountRepository::update_profile(pool, user_id, &request, actor).await?;
        AuthService::get_login_info(pool, user_id).await
    }

    #[cfg(test)]
    pub async fn change_password(
        pool: &SqlitePool,
        user_id: i64,
        request: ChangeAccountPasswordRequest,
    ) -> Result<(), ServiceError> {
        Self::change_password_inner(pool, user_id, request, None).await
    }

    pub async fn change_password_authorized(
        pool: &SqlitePool,
        user_id: i64,
        request: ChangeAccountPasswordRequest,
        actor: &AuthClaims,
    ) -> Result<(), ServiceError> {
        Self::change_password_inner(pool, user_id, request, Some(actor)).await
    }

    async fn change_password_inner(
        pool: &SqlitePool,
        user_id: i64,
        request: ChangeAccountPasswordRequest,
        actor: Option<&AuthClaims>,
    ) -> Result<(), ServiceError> {
        tracing::info!("Changing account password for user_id: {}", user_id);
        let current = AccountRepository::find_password_hash_by_id(pool, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("User".to_string()))?;
        let password_hash = Self::build_password_hash(
            &request.current_password,
            &current.password_hash,
            &request.new_password,
            &request.confirm_password,
        )?;

        let updated = match actor {
            Some(actor) => {
                AccountRepository::update_password_authorized(
                    pool,
                    user_id,
                    &current.password_hash,
                    &password_hash,
                    actor,
                )
                .await?
            }
            None => {
                AccountRepository::update_password(
                    pool,
                    user_id,
                    &current.password_hash,
                    &password_hash,
                )
                .await?
            }
        };
        if !updated {
            PermissionService::clear_user_cache(user_id);
            return if AccountRepository::find_password_hash_by_id(pool, user_id).await?.is_some() {
                Err(ServiceError::InvalidCurrentPassword)
            } else {
                Err(ServiceError::NotFound("User".to_string()))
            };
        }
        SessionRepository::revoke_all(pool, user_id, chrono::Utc::now().timestamp()).await?;
        PermissionService::clear_user_cache(user_id);
        Ok(())
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

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
