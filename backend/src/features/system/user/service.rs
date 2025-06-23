// backend/src/features/user/service.rs

// Business logic for user management.
//
// This service layer handles the core business operations for users,
// such as validation, combining repository methods, and ensuring
// data consistency. It is designed to be independent of the web framework,
// returning pure data models or `ServiceError` variants.

use super::{
    model::{
        CreateUserRequest, UpdateUserRequest, UserListResponse, UserQueryParams, UserResponse,
    },
    repo::UserRepository,
};
use crate::{
    common::api::{OptionItem, OptionsQuery, ServiceError},
    core::password::PasswordUtils,
};
use axum::extract::Query;
use sqlx::PgPool;

/// Service for user-related operations.
pub struct UserService;

impl UserService {
    // --- Public API Methods ---

    /// Creates a new user.
    pub async fn create_user(
        pool: &PgPool,
        request: CreateUserRequest,
    ) -> Result<UserResponse, ServiceError> {
        tracing::info!("Creating new user with username: {}", request.username);

        if UserRepository::find_by_username(pool, &request.username)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check username conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Username already exists: {}", request.username);
            return Err(ServiceError::UsernameConflict);
        }

        if UserRepository::find_by_email(pool, &request.email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check email conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Email already exists: {}", request.email);
            return Err(ServiceError::EmailConflict);
        }

        let password_hash = PasswordUtils::hash_password(&request.password)?;

        let user = UserRepository::create(
            pool,
            &request.username,
            &request.email,
            &password_hash,
            request.real_name.as_deref(),
            request.status.unwrap_or(1),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user in database: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if !request.role_ids.is_empty() {
            UserRepository::set_user_roles(pool, user.id, &request.role_ids).await.map_err(
                |e| {
                    tracing::error!("Failed to set user roles: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                },
            )?;
        }

        let roles = UserRepository::get_user_roles(pool, user.id).await.map_err(|e| {
            tracing::error!("Failed to retrieve user roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let mut user_response = UserResponse::from(user);
        user_response.roles = roles;

        tracing::info!("Successfully created user with id: {}", user_response.id);
        Ok(user_response)
    }

    /// Updates an existing user.
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, ServiceError> {
        tracing::info!("Updating user with id: {}", id);

        let user = UserRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        if let Some(email) = &request.email {
            if !email.eq_ignore_ascii_case(&user.email) {
                if UserRepository::find_by_email(pool, email)
                    .await
                    .map_err(|_| ServiceError::DatabaseQueryFailed)?
                    .is_some()
                {
                    return Err(ServiceError::EmailConflict);
                }
            }
        }

        let updated_user = UserRepository::update(
            pool,
            id,
            request.email.as_deref(),
            request.real_name.as_deref(),
            request.status,
        )
        .await
        .map_err(|_| ServiceError::DatabaseQueryFailed)?
        .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        if let Some(role_ids) = request.role_ids {
            UserRepository::set_user_roles(pool, updated_user.id, &role_ids)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        }

        let roles = UserRepository::get_user_roles(pool, updated_user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        let mut user_response = UserResponse::from(updated_user);
        user_response.roles = roles;
        Ok(user_response)
    }

    /// Deletes a user by their ID.
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        let deleted = UserRepository::soft_delete(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        if !deleted {
            return Err(ServiceError::NotFound("User not found".to_string()));
        }
        Ok(())
    }

    /// Retrieves a paginated list of users.
    pub async fn get_user_list(
        pool: &PgPool,
        params: UserQueryParams,
    ) -> Result<UserListResponse, ServiceError> {
        let page = params.current.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        let total = UserRepository::count_users(pool, params.username.as_deref(), params.status)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        if total == 0 {
            return Ok(UserListResponse::default());
        }

        let user_entities = UserRepository::find_with_pagination(
            pool,
            offset,
            page_size,
            params.username.as_deref(),
            params.status,
        )
        .await
        .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        let mut list = Vec::with_capacity(user_entities.len());
        for user in user_entities {
            let roles = UserRepository::get_user_roles(pool, user.id)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;
            let mut user_response = UserResponse::from(user);
            user_response.roles = roles;
            list.push(user_response);
        }

        Ok(UserListResponse { list, total, page, page_size })
    }

    /// Retrieves a single user by their ID.
    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<UserResponse, ServiceError> {
        let user = UserRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        let roles = UserRepository::get_user_roles(pool, user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        let mut user_response = UserResponse::from(user);
        user_response.roles = roles;
        Ok(user_response)
    }

    /// Retrieves user options for dropdown selections
    ///
    /// Returns simplified user data optimized for frontend dropdown components.
    /// Supports filtering by status, search term, and result limiting.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `query` - Query parameters (status, q, limit)
    ///
    /// # Returns
    /// * `Result<Vec<OptionItem<i64>>, ServiceError>` - List of option items or service error
    pub async fn get_user_options(
        pool: &PgPool,
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Retrieving user options with query: status={:?}, search={:?}, limit={:?}",
            query.status,
            query.q,
            query.limit
        );

        let status = query.status.as_deref().unwrap_or("enabled");
        let users = UserRepository::find_options(pool, status, query.q.as_deref(), query.limit)
            .await
            .map_err(|e| {
                tracing::error!("Failed to retrieve user options from database: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        let options: Vec<OptionItem<i64>> = users
            .into_iter()
            .map(|(id, display_name)| OptionItem { label: display_name, value: id })
            .collect();

        tracing::info!("Successfully retrieved {} user options", options.len());
        Ok(options)
    }
}
