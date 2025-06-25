use super::model::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfo, UserInfoResponse,
};
use crate::{
    common::error::ServiceError,
    core::{
        jwt::{self, Claims},
        password::PasswordUtils,
    },
    features::{
        auth::permission::PermissionService,
        system::{
            menu::{model::MenuResponse, service::MenuService},
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
    ) -> Result<RegisterResponse, ServiceError> {
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
        let user_response = UserService::create_user(pool, &create_request).await?;

        // Generate JWT for new user
        tracing::debug!(user_id = user_response.id, "Generating JWT for new user.");
        let token =
            jwt::generate_token(user_response.id, &user_response.username).map_err(|e| {
                tracing::error!(
                    "Failed to generate token for new user {}: {:?}",
                    user_response.id,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })?;

        let response = RegisterResponse {
            user: UserInfo { id: user_response.id, username: user_response.username },
            token,
        };

        Ok(response)
    }

    /// Login with username/password
    #[tracing::instrument(name = "auth_login", skip(pool, request), fields(username = %request.username))]
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<LoginResponse, ServiceError> {
        tracing::info!("Attempting to log in user.");

        // Verify credentials
        let user = Self::verify_login(pool, &request.username, &request.password).await?;
        tracing::info!(user_id = user.id, "User authenticated successfully.");

        // Generate JWT token
        tracing::debug!(user_id = user.id, "Generating JWT for logged-in user.");
        let token = jwt::generate_token(user.id, &user.username).map_err(|e| {
            tracing::error!("Failed to generate token for user {}: {:?}", user.id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        let user_info = Self::get_user_info(
            pool,
            Claims { user_id: user.id, username: user.username, exp: 0, iat: 0 },
        )
        .await?;

        Ok(LoginResponse { token, user_info })
    }

    /// Get detailed user info with roles and menus
    #[tracing::instrument(name = "get_user_info", skip(pool, claims), fields(user_id = claims.user_id, username = %claims.username))]
    pub async fn get_user_info(
        pool: &PgPool,
        claims: Claims,
    ) -> Result<UserInfoResponse, ServiceError> {
        tracing::info!("Fetching user info for authenticated user.");

        // Get user basic info
        let user = Self::get_authenticated_user(pool, claims.user_id).await?;

        // Parallel query for roles, permissions and menus
        let (roles_result, permissions_result, menus_result) = tokio::join!(
            UserRepository::get_user_role_infos(pool, user.id),
            UserRepository::get_user_permissions(pool, user.id),
            Self::get_user_menus(pool, user.id)
        );

        let roles = roles_result.map_err(|_| ServiceError::DatabaseQueryFailed)?;
        let permissions = permissions_result.map_err(|_| ServiceError::DatabaseQueryFailed)?;
        let menus = menus_result?;

        // Cache user permissions
        tracing::debug!(user_id = user.id, "Caching user permissions.");
        PermissionService::cache_user_permissions(user.id, permissions);

        let user_info = UserInfoResponse {
            id: user.id,
            username: user.username,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            roles,
            menus,
        };

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

    /// Get user accessible menus based on roles
    pub async fn get_user_menus(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<MenuResponse>, ServiceError> {
        let roles = UserRepository::get_user_role_infos(pool, user_id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        if roles.is_empty() {
            return Ok(vec![]);
        }

        let role_ids: Vec<i64> = roles.into_iter().map(|r| r.id).collect();

        MenuService::get_menus_by_role_ids(pool, &role_ids).await
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
