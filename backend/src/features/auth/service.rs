use super::{
    model::{LoginCredentialsEntity, LoginRequest, LoginResponse, UserInfoResponse},
    permission::PermissionService,
    repo::AuthRepository,
};
use crate::{
    common::error::ServiceError,
    core::{
        jwt::{self},
        password::PasswordUtils,
    },
    features::auth::model::UserStatus,
};
use sqlx::PgPool;
use tokio::try_join;
use tracing;

/// Authentication service for login/register operations
pub struct AuthService;

impl AuthService {
    /// Login with username/password
    #[tracing::instrument(name = "auth_login", skip(pool, request))]
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<LoginResponse, ServiceError> {
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
        let token =
            jwt::generate_token(user.id, &user.username, user.is_super_admin).map_err(|e| {
                tracing::error!(
                    "Failed to generate token for user_id={}, username={}: {:?}",
                    user.id,
                    user.username,
                    e
                );
                ServiceError::TokenCreationFailed
            })?;

        tracing::debug!("JWT token generated successfully for user_id={}", user.id);

        // 3. cache user permissions
        Self::cache_user_permissions(pool, user.id).await.map_err(|e| {
            tracing::error!(
                "Failed to cache permissions during login for user_id={}: {:?}",
                user.id,
                e
            );
            e
        })?;

        // 4. update last login time
        AuthRepository::update_last_login(pool, user.id).await.map_err(|e| {
            tracing::error!("Failed to update last login time for user_id={}: {:?}", user.id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            user.username,
            user.id,
            total_time
        );

        Ok(LoginResponse { token, username: user.username.clone(), user_id: user.id })
    }

    /// Get detailed user info with roles, menus, and permissions
    #[tracing::instrument(name = "get_me_info", skip(pool))]
    pub async fn get_me_info(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<UserInfoResponse, ServiceError> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        // Get user basic info
        let user = AuthRepository::get_user_by_id(pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get user basic info for user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?
            .ok_or_else(|| {
                tracing::warn!("User not found for user_id={}", user_id);
                ServiceError::NotFound("User".to_string())
            })?;

        tracing::debug!(
            "User basic info retrieved for user_id={}, username={}",
            user_id,
            user.username
        );

        // Get menus and permissions in parallel
        let (menus, permissions) = try_join!(
            AuthRepository::get_user_menus(pool, user_id),
            AuthRepository::get_user_permissions(pool, user_id)
        )
        .map_err(|e| {
            tracing::error!(
                "Failed to get user menus/permissions for user_id={}: {:?}",
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        // Convert to response format
        let user_info = UserInfoResponse {
            id: user.id,
            username: user.username.clone(),
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            menus,
            permissions,
        };

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}: {} menus, {} permissions",
            user_id,
            user_info.username,
            user_info.menus.len(),
            user_info.permissions.len()
        );

        Ok(user_info)
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
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get login credentials for username={}: {:?}",
                    username,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })?
            .ok_or_else(|| {
                tracing::warn!("Invalid login attempt: username not found: {}", username);
                ServiceError::InvalidCredentials
            })?;

        tracing::debug!(
            "User found for username={}, user_id={}, status={}",
            username,
            user.id,
            user.status
        );

        // 2. check if user is enabled
        let status = UserStatus::try_from(user.status).map_err(|e| {
            tracing::error!(
                "Invalid user status for user_id={}, status={}: {:?}",
                user.id,
                user.status,
                e
            );
            e
        })?;

        status.check_status().map_err(|e| {
            tracing::warn!(
                "User status check failed for user_id={}, username={}, status={:?}: {:?}",
                user.id,
                username,
                status,
                e
            );
            e
        })?;

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
    pub async fn cache_user_permissions(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Starting to cache user permissions for user_id: {}", user_id);

        let permissions: Vec<String> =
            AuthRepository::get_user_permissions(pool, user_id).await.map_err(|e| {
                tracing::error!(
                    "Failed to get user permissions for caching, user_id={}: {:?}",
                    user_id,
                    e
                );
                ServiceError::CacheUserPermissionsFailed
            })?;

        PermissionService::cache_user_permissions(user_id, permissions.clone());
        tracing::info!(
            "Successfully cached {} permissions for user_id={}: {:?}",
            permissions.len(),
            user_id,
            permissions
        );

        Ok(())
    }
}
