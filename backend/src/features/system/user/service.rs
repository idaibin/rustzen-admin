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
    common::api::{OptionItem, OptionsQuery},
    common::error::ServiceError,
};
use axum::extract::Query;
use sqlx::PgPool;

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Create new user with unified logic for all scenarios
    /// Handles: registration, admin creation, custom creation
    pub async fn create_user(
        pool: &PgPool,
        request: &CreateUserRequest,
    ) -> Result<UserResponse, ServiceError> {
        tracing::info!("Creating user: {}", request.username);

        // Check username conflict
        if UserRepository::find_by_username(pool, &request.username)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check username conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Username exists: {}", request.username);
            return Err(ServiceError::UsernameConflict);
        }

        // Check email conflict
        if UserRepository::find_by_email(pool, &request.email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check email conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Email exists: {}", request.email);
            return Err(ServiceError::EmailConflict);
        }

        // Create user using unified repository method
        let user = UserRepository::create_user(pool, request).await.map_err(|e| {
            tracing::error!("Failed to create user: {:?}", e);

            // Check for role-related errors
            match &e {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23503" && db_err.message().contains("role") {
                            return ServiceError::InvalidRoleId;
                        }
                    }
                }
                sqlx::Error::RowNotFound => {
                    return ServiceError::InvalidRoleId;
                }
                _ => {}
            }

            ServiceError::DatabaseQueryFailed
        })?;

        // Get user roles
        let roles = UserRepository::get_user_role_infos(pool, user.id).await.map_err(|e| {
            tracing::error!("Failed to retrieve user roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let mut user_response = UserResponse::from(user);
        user_response.roles = roles;

        tracing::info!("Created user: {}", user_response.id);
        Ok(user_response)
    }

    /// Update existing user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, ServiceError> {
        tracing::info!("Updating user: {}", id);

        let user = UserRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        // Check email conflict if changing email
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

        // Update roles if provided
        if let Some(role_ids) = request.role_ids {
            UserRepository::set_user_roles(pool, updated_user.id, &role_ids)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        }

        let roles = UserRepository::get_user_role_infos(pool, updated_user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        let mut user_response = UserResponse::from(updated_user);
        user_response.roles = roles;
        Ok(user_response)
    }

    /// Delete user by ID
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        let deleted = UserRepository::soft_delete(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        if !deleted {
            return Err(ServiceError::NotFound("User not found".to_string()));
        }
        Ok(())
    }

    /// Get paginated user list with filtering
    pub async fn get_user_list(
        pool: &PgPool,
        params: UserQueryParams,
    ) -> Result<UserListResponse, ServiceError> {
        let page = params.current.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        let total =
            UserRepository::count_users(pool, params.username.as_deref(), params.status.as_deref())
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
            params.status.as_deref(),
        )
        .await
        .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        let mut list = Vec::with_capacity(user_entities.len());
        for user in user_entities {
            let roles = UserRepository::get_user_role_infos(pool, user.id)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;
            let mut user_response = UserResponse::from(user);
            user_response.roles = roles;
            list.push(user_response);
        }

        Ok(UserListResponse { list, total, page, page_size })
    }

    /// Get single user by ID
    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<UserResponse, ServiceError> {
        let user = UserRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        let roles = UserRepository::get_user_role_infos(pool, user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        let mut user_response = UserResponse::from(user);
        user_response.roles = roles;
        Ok(user_response)
    }

    /// Get user options for dropdowns
    pub async fn get_user_options(
        pool: &PgPool,
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Retrieving user options: status={:?}, search={:?}, limit={:?}",
            query.status,
            query.q,
            query.limit
        );

        // Parse status: support numbers or "all"
        let status = query.status.as_deref().and_then(|s| {
            match s {
                "1" => Some(1),
                "2" => Some(2),
                "all" => None,
                _ => {
                    tracing::warn!("Invalid status '{}'. Valid: '1', '2', 'all'", s);
                    Some(1) // Default to active
                }
            }
        });

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

        tracing::info!("Retrieved {} user options", options.len());
        Ok(options)
    }
}
