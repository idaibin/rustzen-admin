#[cfg(feature = "full")]
use super::types::LoginAuditCommand;
use super::{
    repo::AuthRepository,
    types::{AuthUserRow, LoginCredentialsRow, LoginResp, UserInfoResp, UserStatus},
};
#[cfg(feature = "full")]
use crate::features::manage::log::{service::LogService, types::LogWriteCommand};
use crate::{
    common::error::ServiceError,
    infra::{auth_runtime::jwt_codec, password::PasswordUtils, permission::PermissionService},
};

use sqlx::SqlitePool;
#[cfg(feature = "full")]
use std::time::Instant;

/// Auth service for login and current-user session operations.
pub struct AuthService;

impl AuthService {
    #[cfg(feature = "full")]
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
                    "用户登录成功",
                    start_time,
                    &audit_command,
                )
                .await;
                Ok(response)
            }
            Err(err) => {
                let description = login_failure_description(&err);
                Self::record_login_operation(
                    pool,
                    0,
                    username,
                    "FAIL",
                    description,
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

        AuthRepository::update_last_login(pool, user.id).await?;

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

        let permissions = PermissionService::refresh_user_permissions(pool, user_id).await?;

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

    #[cfg(feature = "full")]
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

        let permissions = PermissionService::refresh_user_permissions(pool, user_id).await?;
        tracing::info!(
            "Successfully refreshed {} permissions cache for user_id={}",
            permissions.len(),
            user_id
        );
        Ok(())
    }
}

#[cfg(feature = "full")]
fn login_failure_description(error: &ServiceError) -> &'static str {
    match error {
        ServiceError::InvalidCredentials => "用户名或密码错误",
        ServiceError::UserIsDisabled => "账号已禁用",
        ServiceError::UserIsPending => "账号待审核",
        ServiceError::UserIsLocked => "账号已锁定",
        _ => "登录失败，请稍后重试",
    }
}

#[cfg(all(test, feature = "full"))]
mod login_description_tests {
    use super::login_failure_description;
    use crate::common::error::ServiceError;

    #[test]
    fn login_failures_use_chinese_business_descriptions() {
        assert_eq!(
            login_failure_description(&ServiceError::InvalidCredentials),
            "用户名或密码错误"
        );
        assert_eq!(login_failure_description(&ServiceError::UserIsDisabled), "账号已禁用");
        assert_eq!(login_failure_description(&ServiceError::UserIsPending), "账号待审核");
        assert_eq!(login_failure_description(&ServiceError::UserIsLocked), "账号已锁定");
        assert_eq!(
            login_failure_description(&ServiceError::DatabaseQueryFailed),
            "登录失败，请稍后重试"
        );
    }
}
