use super::{
    repo::AuthRepository,
    types::{LoginCredentialsRow, LoginResp, UserInfoResp, UserStatus},
};
use crate::{
    common::error::ServiceError,
    infra::{
        jwt::{self},
        password::PasswordUtils,
        permission::PermissionService,
    },
};

use sqlx::PgPool;
use tracing;

/// Authentication service for login/register operations
pub struct AuthService;

impl AuthService {
    /// Login with username/password
    pub async fn login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginResp, ServiceError> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt received for username: {}", username);

        // 1. verify login credentials
        let user = Self::verify_login(pool, username, password).await.map_err(|e| {
            tracing::warn!("Login verification failed for username={}: {:?}", username, e);
            e
        })?;
        let verification_time = start.elapsed();
        tracing::debug!(
            "User verification completed in {:?} for user_id={}",
            verification_time,
            user.id
        );

        // 2. generate token
        let token = jwt::generate_token(user.id, username, user.is_system).map_err(|e| {
            tracing::error!("Failed to generate token for user_id={}: {:?}", user.id, e);
            ServiceError::TokenCreationFailed
        })?;

        tracing::debug!("JWT token generated successfully for user_id={}", user.id);

        // 3. cache user permissions
        Self::cache_user_permissions(pool, user.id, user.is_system).await.map_err(|e| {
            tracing::error!(
                "Failed to cache permissions during login for user_id={}: {:?}",
                user.id,
                e
            );
            e
        })?;

        // 4. update last login time
        let user_id_clone = user.id;
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            tracing::debug!(user_id = user_id_clone, "Updating last login in background");
            if let Err(error) = AuthRepository::update_last_login(&pool_clone, user_id_clone).await
            {
                tracing::debug!(
                    user_id = user_id_clone,
                    error = ?error,
                    "Failed to update last login in background"
                );
            } else {
                tracing::debug!(user_id = user_id_clone, "Updated last login in background");
            }
        });

        // 5. get user info
        let user_info = Self::get_login_info(pool, user.id).await?;

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            username,
            user.id,
            total_time
        );

        // 6. return login vo
        Ok(LoginResp { token, user_info })
    }

    /// Get detailed user info with roles, menus, and permissions
    pub async fn get_login_info(pool: &PgPool, user_id: i64) -> Result<UserInfoResp, ServiceError> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        // Get user basic info
        let user = AuthRepository::find_user_by_id(pool, user_id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;
        let super::types::AuthUserRow { id, username, real_name, email, avatar_url, is_system } =
            user;

        tracing::debug!("User basic info retrieved for user_id={}, username={}", user_id, username);

        let permissions = Self::load_permissions(pool, user_id, is_system).await?;

        // Refresh user permissions cache
        Self::refresh_user_permissions_cache(user_id, &permissions).await?;

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}",
            user_id,
            username
        );

        Ok(UserInfoResp { id, username, real_name, email, avatar_url, is_system, permissions })
    }

    /// Verify login credentials
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsRow, ServiceError> {
        tracing::info!("Starting login verification for username: {}", username);

        // 1. get login credentials
        let user = AuthRepository::get_login_credentials(pool, username)
            .await?
            .ok_or(ServiceError::InvalidCredentials)?;

        tracing::debug!(
            "User found for username={}, user_id={}, status={}",
            username,
            user.id,
            user.status
        );

        // 2. check if user is enabled
        let status = UserStatus::try_from(user.status)?;
        status.check_status()?;

        // 3. verify password
        if !PasswordUtils::verify_password(password, &user.password_hash) {
            tracing::warn!(
                "Invalid login attempt: password verification failed for username={}, user_id={}",
                username,
                user.id
            );
            return Err(ServiceError::InvalidCredentials);
        }

        tracing::info!(
            "Login verification successful for username={}, user_id={}",
            username,
            user.id
        );
        Ok(user)
    }

    /// Cache user permissions
    pub async fn cache_user_permissions(
        pool: &PgPool,
        user_id: i64,
        is_system: bool,
    ) -> Result<(), ServiceError> {
        tracing::debug!("Starting to cache user permissions for user_id: {}", user_id);

        let permissions = Self::load_permissions(pool, user_id, is_system).await?;
        Self::cache_permissions_with_values(user_id, &permissions)
    }

    /// Refresh user permissions cache
    pub async fn refresh_user_permissions_cache(
        user_id: i64,
        permissions: &[String],
    ) -> Result<(), ServiceError> {
        tracing::debug!("Refreshing permissions cache for user_id: {}", user_id);

        Self::cache_permissions_with_values(user_id, permissions)
    }

    pub async fn update_avatar(
        pool: &PgPool,
        user_id: i64,
        avatar_url: &str,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating avatar for user_id: {}", user_id);
        AuthRepository::update_avatar(pool, user_id, avatar_url).await?;
        tracing::info!("Avatar updated successfully for user_id: {}", user_id);
        Ok(())
    }

    async fn load_permissions(
        pool: &PgPool,
        user_id: i64,
        is_system: bool,
    ) -> Result<Vec<String>, ServiceError> {
        if is_system {
            AuthRepository::get_all_permissions(pool).await
        } else {
            AuthRepository::get_user_permissions(pool, user_id).await
        }
    }

    fn cache_permissions_with_values(
        user_id: i64,
        permissions: &[String],
    ) -> Result<(), ServiceError> {
        PermissionService::cache_user_permissions(user_id, permissions);
        tracing::info!(
            "Successfully refreshed {} permissions cache for user_id={}",
            permissions.len(),
            user_id
        );
        Ok(())
    }
}
