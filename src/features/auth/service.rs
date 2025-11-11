use super::{
    dto::LoginRequest,
    entity::{LoginCredentialsEntity, UserStatus},
    repo::AuthRepository,
    vo::{LoginVo, UserInfoVo},
};
use crate::{
    common::error::ServiceError,
    core::{
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
    pub async fn login(pool: &PgPool, request: LoginRequest) -> Result<LoginVo, ServiceError> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt received for username: {}", request.username);

        // 1. verify login credentials
        let user =
            Self::verify_login(pool, &request.username, &request.password).await.map_err(|e| {
                tracing::warn!(
                    "Login verification failed for username={}: {:?}",
                    request.username,
                    e
                );
                e
            })?;
        let verification_time = start.elapsed();
        tracing::debug!(
            "User verification completed in {:?} for user_id={}",
            verification_time,
            user.id
        );

        // 2. generate token
        let token = jwt::generate_token(user.id, &request.username).map_err(|e| {
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
        let pool_clone = pool.clone();
        let user_id_clone = user.id;
        tokio::spawn(async move {
            let _ = AuthRepository::update_last_login(&pool_clone, user_id_clone).await;
        });

        // 5. get user info
        let user_info = Self::get_login_info(pool, user.id).await?;

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            &request.username,
            user.id,
            total_time
        );

        // 6. return login vo
        Ok(LoginVo { token, user_info })
    }

    /// Get detailed user info with roles, menus, and permissions
    pub async fn get_login_info(pool: &PgPool, user_id: i64) -> Result<UserInfoVo, ServiceError> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        // Get user basic info
        let user = AuthRepository::get_user_by_id(pool, user_id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;

        tracing::debug!(
            "User basic info retrieved for user_id={}, username={}",
            user_id,
            user.username
        );

        // Get menus and permissions in parallel
        let permissions = if user.is_system {
            vec!["*".to_string()]
        } else {
            AuthRepository::get_user_permissions(pool, user_id).await?
        };

        // Refresh user permissions cache
        Self::refresh_user_permissions_cache(user_id, user.is_system, &permissions).await?;

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}",
            user_id,
            user.username
        );

        Ok(UserInfoVo {
            id: user.id,
            username: user.username.clone(),
            real_name: user.real_name,
            email: user.email,
            avatar_url: user.avatar_url,
            is_system: user.is_system,
            permissions,
        })
    }

    /// Verify login credentials
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsEntity, ServiceError> {
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
        if !PasswordUtils::verify_password(&password.to_string(), &user.password_hash) {
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

        if is_system {
            PermissionService::cache_user_permissions(user_id, &["*".to_string()]);
            tracing::info!("Successfully cached * permissions for user_id={}", user_id);
            return Ok(());
        }
        let permissions: Vec<String> = AuthRepository::get_user_permissions(pool, user_id).await?;

        PermissionService::cache_user_permissions(user_id, &permissions);
        tracing::info!(
            "Successfully cached {} permissions for user_id={}: {:?}",
            permissions.len(),
            user_id,
            permissions
        );

        Ok(())
    }

    /// Refresh user permissions cache
    pub async fn refresh_user_permissions_cache(
        user_id: i64,
        is_system: bool,
        permissions: &[String],
    ) -> Result<(), ServiceError> {
        tracing::debug!("Refreshing permissions cache for user_id: {}", user_id);

        if is_system {
            PermissionService::cache_user_permissions(user_id, &["*".to_string()]);
            tracing::info!("Successfully refreshed * permissions cache for user_id={}", user_id);
            return Ok(());
        }

        PermissionService::cache_user_permissions(user_id, permissions);
        tracing::info!(
            "Successfully refreshed {} permissions cache for user_id={}: {:?}",
            permissions.len(),
            user_id,
            permissions
        );
        Ok(())
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
}
