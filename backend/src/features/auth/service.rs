use super::model::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfo, UserInfoResponse,
};
use crate::{
    common::error::ServiceError,
    core::{
        jwt::{self, Claims},
        password::PasswordUtils,
    },
    features::system::{
        menu::{model::MenuResponse, service::MenuService},
        user::{
            model::{CreateUserRequest, UserEntity, UserStatus},
            repo::UserRepository,
            service::UserService,
        },
    },
};
use sqlx::PgPool;
use tracing;

/// A service for authentication-related operations.
pub struct AuthService;

impl AuthService {
    /// Handles user registration.
    ///
    /// This function validates the registration request, creates a new user,
    /// and if successful, generates a JWT for the new user.
    ///
    /// # Errors
    ///
    /// - `ServiceError::UsernameConflict` if the username is already taken.
    /// - `ServiceError::EmailConflict` if the email is already taken.
    /// - `ServiceError::DatabaseQueryFailed` for other internal errors.
    #[tracing::instrument(name = "auth_register", skip(pool, request), fields(username = %request.username, email = %request.email))]
    pub async fn register(
        pool: &PgPool,
        request: RegisterRequest,
    ) -> Result<RegisterResponse, ServiceError> {
        tracing::info!("Attempting to register new user.");

        // 组装统一的用户创建请求（注册场景，添加默认值）
        let create_request = CreateUserRequest {
            username: request.username.clone(),
            email: request.email.clone(),
            password: PasswordUtils::hash_password(&request.password)?,
            real_name: None,  // 注册时默认为None
            status: None,     // 注册时默认为正常状态
            role_ids: vec![], // 注册时默认无角色
        };

        // 使用统一的用户创建服务
        let user_response = UserService::create_user(pool, &create_request).await?;

        // Generate a JWT for the new user.
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

    /// Handles user login.
    ///
    /// Verifies user credentials and returns a JWT upon success.
    ///
    /// # Errors
    ///
    /// - `ServiceError::InvalidCredentials` if the login attempt fails.
    /// - `ServiceError::DatabaseQueryFailed` for other internal errors.
    #[tracing::instrument(name = "auth_login", skip(pool, request), fields(username = %request.username))]
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<LoginResponse, ServiceError> {
        tracing::info!("Attempting to log in user.");
        let user = Self::verify_login(pool, &request.username, &request.password).await?;
        tracing::info!(user_id = user.id, "User authenticated successfully.");

        tracing::debug!(user_id = user.id, "Generating JWT for logged-in user.");
        let token = jwt::generate_token(user.id, &user.username).map_err(|e| {
            tracing::error!("Failed to generate token for user {}: {:?}", user.id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(LoginResponse { token })
    }

    /// Retrieves detailed information for the currently authenticated user.
    ///
    /// # Errors
    ///
    /// - `ServiceError::NotFound` if the user from the token does not exist.
    /// - `ServiceError::InvalidCredentials` as a safeguard if token data mismatches.
    /// - `ServiceError::DatabaseQueryFailed` for other internal errors.
    #[tracing::instrument(name = "get_user_info", skip(pool, claims), fields(user_id = claims.user_id, username = %claims.username))]
    pub async fn get_user_info(
        pool: &PgPool,
        claims: Claims,
    ) -> Result<UserInfoResponse, ServiceError> {
        tracing::info!("Fetching user info for authenticated user.");
        let user = Self::get_authenticated_user(pool, claims.user_id).await?;

        // This should not happen if the token is valid, but as a safeguard.
        if user.username != claims.username {
            tracing::warn!(
                token_user_id = claims.user_id,
                token_username = %claims.username,
                db_username = %user.username,
                "Token-database username mismatch."
            );
            return Err(ServiceError::InvalidCredentials);
        }

        tracing::debug!(user_id = user.id, "Fetching menus for user.");
        let menus = Self::get_user_menus(pool, user.id).await?;
        tracing::debug!(user_id = user.id, menu_count = menus.len(), "Successfully fetched menus.");

        let roles = UserRepository::get_user_roles(pool, user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

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

    /// Verifies user credentials for login.
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<UserEntity, ServiceError> {
        let user = UserRepository::find_by_username(pool, username)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or(ServiceError::InvalidCredentials)?;

        // 检查用户状态
        if let Some(status) = UserStatus::from_i16(user.status) {
            if !status.is_active() {
                return Err(ServiceError::UserIsDisabled);
            }
        } else {
            // 无效状态也拒绝登录
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

    /// Retrieves all menus a user has access to, based on their roles.
    pub async fn get_user_menus(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<MenuResponse>, ServiceError> {
        let roles = UserRepository::get_user_roles(pool, user_id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        if roles.is_empty() {
            return Ok(vec![]);
        }

        let role_ids: Vec<i64> = roles.into_iter().map(|r| r.id).collect();

        MenuService::get_menus_by_role_ids(pool, &role_ids).await
    }

    /// Internal helper to get authenticated user information.
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
