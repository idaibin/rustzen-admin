use super::{
    repo::AuthRepository,
    types::{
        AuthUserRow, LoginAuditCommand, LoginCredentialsRow, LoginResp, UserInfoResp, UserStatus,
    },
};
use crate::{
    common::error::ServiceError,
    features::manage::log::{service::LogService, types::LogWriteCommand},
    infra::{auth_runtime::jwt_codec, password::PasswordUtils, permission::PermissionService},
};

use sqlx::SqlitePool;
use std::time::Instant;

/// Auth service for login and current-user session operations.
pub struct AuthService;

impl AuthService {
    pub async fn login_with_audit(
        pool: &SqlitePool,
        username: &str,
        password: &str,
        audit_command: LoginAuditCommand,
    ) -> Result<LoginResp, ServiceError> {
        let start_time = Instant::now();

        match Self::login(pool, username, password).await {
            Ok(response) => {
                Self::record_login_operation(
                    pool,
                    response.user_info.id,
                    username,
                    "SUCCESS",
                    "User login successful",
                    start_time,
                    &audit_command,
                )
                .await;
                Ok(response)
            }
            Err(err) => {
                let description = err.to_string();
                Self::record_login_operation(
                    pool,
                    0,
                    username,
                    "FAIL",
                    &description,
                    start_time,
                    &audit_command,
                )
                .await;
                Err(err)
            }
        }
    }

    /// Login with username/password
    pub async fn login(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<LoginResp, ServiceError> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt received for username: {}", username);

        let user = Self::verify_login(pool, username, password).await.map_err(|error| {
            tracing::warn!("Login verification failed for username={}: {:?}", username, error);
            error
        })?;
        let verification_time = start.elapsed();
        tracing::debug!(
            "User verification completed in {:?} for user_id={}",
            verification_time,
            user.id
        );

        let token = jwt_codec().encode(user.id, username).map_err(|e| {
            tracing::error!("Failed to generate token for user_id={}: {:?}", user.id, e);
            ServiceError::TokenCreationFailed
        })?;

        tracing::debug!("JWT token generated successfully for user_id={}", user.id);

        Self::cache_user_permissions(pool, user.id).await.map_err(|e| {
            tracing::error!(
                "Failed to cache permissions during login for user_id={}: {:?}",
                user.id,
                e
            );
            e
        })?;

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

        let user_info = Self::get_login_info(pool, user.id).await?;

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            username,
            user.id,
            total_time
        );

        Ok(LoginResp { token, user_info })
    }

    /// Get detailed user info with roles, menus, and permissions
    pub async fn get_login_info(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<UserInfoResp, ServiceError> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        let user = AuthRepository::find_user_by_id(pool, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("User".to_string()))?;
        let AuthUserRow { id, username, real_name, email, avatar_url, is_system } = user;

        tracing::debug!("User basic info retrieved for user_id={}, username={}", user_id, username);

        let permissions = Self::load_permissions(pool, user_id).await?;

        PermissionService::cache_user_permissions(user_id, &permissions);

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}",
            user_id,
            username
        );

        Ok(UserInfoResp { id, username, real_name, email, avatar_url, is_system, permissions })
    }

    pub fn logout(user_id: i64) {
        PermissionService::clear_user_cache(user_id);
    }

    async fn record_login_operation(
        pool: &SqlitePool,
        user_id: i64,
        username: &str,
        status: &str,
        description: &str,
        start_time: Instant,
        audit_command: &LoginAuditCommand,
    ) {
        if let Err(e) = LogService::record_operation(
            pool,
            LogWriteCommand {
                user_id,
                username: username.to_string(),
                action: "AUTH_LOGIN".to_string(),
                description: description.to_string(),
                data: Some(serde_json::json!({})),
                status: status.to_string(),
                duration_ms: start_time.elapsed().as_millis() as i32,
                ip_address: audit_command.ip_address.clone(),
                user_agent: audit_command.user_agent.clone(),
            },
        )
        .await
        {
            tracing::error!("Failed to log login operation: {:?}", e);
        }
    }

    /// Verify login credentials
    pub async fn verify_login(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsRow, ServiceError> {
        tracing::info!("Starting login verification for username: {}", username);

        let user = AuthRepository::get_login_credentials(pool, username)
            .await?
            .ok_or(ServiceError::InvalidCredentials)?;

        tracing::debug!(
            "User found for username={}, user_id={}, status={}",
            username,
            user.id,
            user.status
        );

        let status = UserStatus::try_from(user.status)?;
        status.check_status()?;

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
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<(), ServiceError> {
        tracing::debug!("Starting to cache user permissions for user_id: {}", user_id);

        let permissions = Self::load_permissions(pool, user_id).await?;
        PermissionService::cache_user_permissions(user_id, &permissions);
        tracing::info!(
            "Successfully refreshed {} permissions cache for user_id={}",
            permissions.len(),
            user_id
        );
        Ok(())
    }

    async fn load_permissions(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        AuthRepository::get_user_permissions(pool, user_id).await
    }
}
