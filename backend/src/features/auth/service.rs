use super::model::{LoginRequest, LoginResponse, RegisterRequest, UserInfoResponse};
use crate::{
    common::error::ServiceError,
    core::{
        jwt::{self},
        password::PasswordUtils,
    },
    features::{
        auth::{model::UserInfo, permission::PermissionService},
        system::{
            menu::service::MenuService,
            user::{
                model::{CreateUserRequest, UserEntity, UserStatus},
                repo::UserRepository,
                service::UserService,
            },
        },
    },
};
use sqlx::PgPool;
use tracing;

/// Authentication service for login/register operations
pub struct AuthService;

impl AuthService {
    /// Register new user account
    #[tracing::instrument(name = "auth_register", skip(pool, request), fields(username = %request.username, email = %request.email))]
    pub async fn register(
        pool: &PgPool,
        request: RegisterRequest,
    ) -> Result<UserInfo, ServiceError> {
        tracing::info!("Attempting to register new user.");

        // Create user with default values for registration
        let create_request = CreateUserRequest {
            username: request.username.clone(),
            email: request.email.clone(),
            password: PasswordUtils::hash_password(&request.password)?,
            real_name: None,  // Default for registration
            status: None,     // Default status
            role_ids: vec![], // No roles by default
        };

        // Use unified user creation service
        let user_info = UserService::create_user(pool, &create_request).await?;

        Ok(UserInfo { id: user_info.id, username: user_info.username })
    }

    /// Login with username/password
    #[tracing::instrument(name = "auth_login", skip(pool, request), fields(username = %request.username))]
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<LoginResponse, ServiceError> {
        // INFO: Business operation start
        tracing::info!("User login attempt: {}", request.username);

        // Verify credentials
        let user = Self::verify_login(pool, &request.username, &request.password).await?;
        // INFO: Important business state change
        tracing::info!("User authenticated successfully: {}", request.username);

        // Generate JWT token
        // DEBUG: Technical details
        tracing::debug!("Generating JWT token for user: {}", user.id);
        let token = jwt::generate_token(user.id, &user.username).map_err(|e| {
            tracing::error!("Failed to generate token for user {}: {:?}", user.id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        // INFO: Business operation completed
        tracing::info!("User login completed successfully: {}", request.username);

        // Cache user permissions
        Self::cache_user_permissions(pool, user.id).await?;

        Ok(LoginResponse { token, username: user.username.clone(), user_id: user.id })
    }

    /// Get detailed user info with roles and menus
    #[tracing::instrument(name = "get_user_info", skip(pool), fields(user_id = user_id, username = %username))]
    pub async fn get_user_info(
        pool: &PgPool,
        user_id: i64,
        username: &str,
    ) -> Result<UserInfoResponse, ServiceError> {
        // INFO: Business operation start
        tracing::info!("Fetching user info for user: {}", username);

        // Get user basic info
        let user = Self::get_authenticated_user(pool, user_id).await?;
        // DEBUG: Detailed execution results
        tracing::debug!("Retrieved user basic info: id={}, status={}", user.id, user.status);

        // Get user role IDs
        let role_ids = UserRepository::get_user_role_ids(pool, user.id).await.map_err(|e| {
            tracing::error!("Failed to get user role IDs for user {}: {:?}", user.id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        // DEBUG: Intermediate calculation results
        tracing::debug!("User {} has {} roles: {:?}", username, role_ids.len(), role_ids);

        // Get menus based on user roles
        let menus = if role_ids.is_empty() {
            // WARN: Business state that needs attention
            tracing::warn!("User {} has no roles assigned", username);
            vec![]
        } else {
            MenuService::get_menus_by_role_ids(pool, &role_ids).await.map_err(|e| {
                tracing::error!("Failed to get menus for user {}: {:?}", username, e);
                ServiceError::DatabaseQueryFailed
            })?
        };

        // DEBUG: Intermediate calculation results
        tracing::debug!("User {} has {} accessible menus", username, menus.len());

        // Cache user permissions
        Self::cache_user_permissions(pool, user.id).await?;

        let user_info = UserInfoResponse {
            id: user.id,
            username: user.username.clone(),
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            menus,
        };

        // INFO: Business operation completed, including important statistics
        tracing::info!(
            "User info retrieved successfully: {} roles, {} menus",
            role_ids.len(),
            user_info.menus.len(),
        );

        Ok(user_info)
    }

    /// Verify login credentials
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<UserEntity, ServiceError> {
        let user = UserRepository::find_by_username(pool, username)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or(ServiceError::InvalidCredentials)?;

        // Check user status
        if let Some(status) = UserStatus::from_i16(user.status) {
            if !status.is_active() {
                return Err(ServiceError::UserIsDisabled);
            }
        } else {
            // Invalid status also denied
            return Err(ServiceError::UserIsDisabled);
        }

        if PasswordUtils::verify_password(password, &user.password_hash) {
            UserRepository::update_last_login(pool, user.id)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;
            Ok(user)
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    pub async fn cache_user_permissions(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        // Extract permissions from menus
        let permissions = MenuService::get_permissions_by_menu(pool, user_id).await?;
        // Cache user permissions for performance
        PermissionService::cache_user_permissions(user_id, permissions.clone());
        // DEBUG: Cache operation
        tracing::debug!("Cached permissions: {:?}", permissions);

        Ok(())
    }

    /// Get authenticated user info
    async fn get_authenticated_user(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<UserEntity, ServiceError> {
        UserRepository::find_by_id(pool, user_id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))
    }
}
