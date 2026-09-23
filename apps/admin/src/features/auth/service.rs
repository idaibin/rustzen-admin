#[cfg(feature = "full")]
use super::types::LoginAuditCommand;
use super::{
    repo::AuthRepository,
    session::SessionRepository,
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

        let user = Self::verify_login(pool, username, password).await?;
        let codec = jwt_codec();
        let sid = uuid::Uuid::new_v4().to_string();
        let issued_at = chrono::Utc::now().timestamp();
        let claims = codec.claims_at(user.id, username, &sid, user.auth_epoch, issued_at);
        let token = codec.encode_claims(&claims).map_err(|error| {
            tracing::error!(%error, "encoding pending login session");
            ServiceError::TokenCreationFailed
        })?;
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.map_err(|error| {
            tracing::error!(%error, "starting login transaction");
            ServiceError::DatabaseQueryFailed
        })?;
        let current = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE id=? AND username=? AND password_hash=? AND auth_epoch=?
                   AND status=1 AND deleted_at IS NULL
             )",
        )
        .bind(user.id)
        .bind(username)
        .bind(&user.password_hash)
        .bind(user.auth_epoch)
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| {
            tracing::error!(%error, "rechecking verified login identity");
            ServiceError::DatabaseQueryFailed
        })?;
        if !current {
            return Err(ServiceError::InvalidCredentials);
        }
        SessionRepository::create_in_transaction(
            &mut tx,
            user.id,
            &sid,
            user.auth_epoch,
            issued_at,
            claims.exp as i64,
        )
        .await?;
        let auth_user = sqlx::query_as::<_, AuthUserRow>(
            "SELECT id,username,real_name,email,avatar_url,is_system FROM users
             WHERE id=? AND status=1 AND deleted_at IS NULL",
        )
        .bind(user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|error| {
            tracing::error!(%error, "loading login user information");
            ServiceError::DatabaseQueryFailed
        })?
        .ok_or(ServiceError::InvalidToken)?;
        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT menu_code FROM user_permissions WHERE user_id=? ORDER BY menu_code",
        )
        .bind(user.id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|error| {
            tracing::error!(%error, "loading login permissions");
            ServiceError::DatabaseQueryFailed
        })?;
        let now = chrono::Utc::now().naive_utc();
        sqlx::query("UPDATE users SET last_login_at=?,updated_at=? WHERE id=?")
            .bind(now)
            .bind(now)
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                tracing::error!(%error, "updating login timestamp");
                ServiceError::DatabaseQueryFailed
            })?;
        tx.commit().await.map_err(|error| {
            tracing::error!(%error, "committing login transaction");
            ServiceError::DatabaseQueryFailed
        })?;
        let user_info = UserInfoResp {
            id: auth_user.id,
            username: auth_user.username,
            real_name: auth_user.real_name,
            email: auth_user.email,
            avatar_url: auth_user.avatar_url,
            is_system: auth_user.is_system,
            permissions,
        };

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            username,
            user.id,
            total_time
        );

        Ok(LoginResp { token, user_info })
    }

    pub async fn verify_login(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsRow, ServiceError> {
        let user = AuthRepository::get_login_credentials(pool, username)
            .await?
            .ok_or(ServiceError::InvalidCredentials)?;
        UserStatus::try_from(user.status)?.check_status()?;
        if !PasswordUtils::verify_password(password, &user.password_hash) {
            return Err(ServiceError::InvalidCredentials);
        }
        Ok(user)
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

        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT menu_code FROM user_permissions WHERE user_id=? ORDER BY menu_code",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, "loading current login permissions");
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}",
            user_id,
            username
        );

        Ok(UserInfoResp { id, username, real_name, email, avatar_url, is_system, permissions })
    }

    pub async fn logout(pool: &SqlitePool, user_id: i64, sid: &str) -> Result<(), ServiceError> {
        SessionRepository::revoke_sid(pool, user_id, sid, chrono::Utc::now().timestamp()).await?;
        PermissionService::clear_user_cache(user_id);
        Ok(())
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
