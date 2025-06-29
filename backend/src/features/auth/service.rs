use super::{
    model::{
        AuthMenuInfoEntity, LoginCredentialsEntity, LoginRequest, LoginResponse, UserInfoResponse,
    },
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
        tracing::info!("User login attempt received");
        tracing::debug!(username = %request.username, "Login details");

        // 1. verify login credentials
        let user = Self::verify_login(pool, &request.username, &request.password).await?;
        tracing::info!("User verification took: {:?}", start.elapsed());

        // 2. generate token and cache permissions
        let token =
            jwt::generate_token(user.id, &user.username, user.is_super_admin).map_err(|e| {
                tracing::error!("Failed to generate token: {:?}", e);
                ServiceError::TokenCreationFailed
            })?;

        // 3. cache user permissions
        Self::cache_user_permissions(pool, user.id)
            .await
            .map_err(|_| ServiceError::CacheUserPermissionsFailed)?;

        // 4. update last login time
        AuthRepository::update_last_login(pool, user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        tracing::info!(user_id = user.id, "Login successful");
        tracing::info!("Total login time: {:?}", start.elapsed());

        Ok(LoginResponse { token, username: user.username.clone(), user_id: user.id })
    }

    /// Get detailed user info with roles and menus
    #[tracing::instrument(name = "get_user_info", skip(pool))]
    pub async fn get_user_info(
        pool: &PgPool,
        user_id: i64,
        username: &str,
    ) -> Result<UserInfoResponse, ServiceError> {
        tracing::info!(user_id, "Fetching user info");
        tracing::debug!(username, "User info details");

        // Get user basic info
        let (user, menus) = try_join!(
            async {
                AuthRepository::get_user_by_id(pool, user_id)
                    .await
                    .map_err(|_| ServiceError::DatabaseQueryFailed)?
                    .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))
            },
            async {
                let menu_entities = AuthRepository::get_user_menus(pool, user_id)
                    .await
                    .map_err(|_| ServiceError::DatabaseQueryFailed)?;
                let menus: Vec<AuthMenuInfoEntity> =
                    menu_entities.into_iter().map(AuthMenuInfoEntity::from).collect();
                Ok::<Vec<AuthMenuInfoEntity>, ServiceError>(menus)
            }
        )?;

        // Convert to response format
        let user_info = UserInfoResponse {
            id: user.id,
            username: user.username.clone(),
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            menus,
        };

        tracing::info!("User info retrieved successfully: {} menus", user_info.menus.len(),);

        Ok(user_info)
    }

    /// Verify login credentials
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsEntity, ServiceError> {
        tracing::info!("Verifying login credentials for user: {}", username);
        tracing::debug!(username, "Login credentials details");

        // 1. get login credentials
        let user = AuthRepository::get_login_credentials(pool, username)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or(ServiceError::InvalidCredentials)?;

        // 2. check if user is enabled
        let status = UserStatus::try_from(user.status)?;
        status.check_status()?;

        // 3. verify password
        if !PasswordUtils::verify_password(&password.to_string(), &user.password_hash) {
            return Err(ServiceError::InvalidCredentials);
        }

        Ok(user)
    }

    /// Cache user permissions
    pub async fn cache_user_permissions(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        tracing::info!("Caching user permissions for user: {}", user_id);
        let permissions: Vec<String> = AuthRepository::get_user_permissions(pool, user_id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        PermissionService::cache_user_permissions(user_id, permissions.clone());
        tracing::info!("Cached permissions: {:?}", permissions);

        Ok(())
    }
}
