use super::{
    repo::{CreateUserCommand, UserListQuery, UserRepository},
    types::{
        CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload, UpdateUserStatusPayload,
        UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
};
use crate::{
    common::{
        error::ServiceError,
        pagination::{Pagination, PaginationQuery},
        query::parse_optional_i16_filter,
    },
    infra::password::PasswordUtils,
};

use sqlx::PgPool;

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Get user list with pagination
    pub async fn list_users(
        pool: &PgPool,
        query: UserQuery,
    ) -> Result<(Vec<UserItemResp>, i64), ServiceError> {
        tracing::info!("Fetching user list with query: {:?}", query);

        let UserQuery { current, page_size, username, status, real_name, email } = query;
        let pagination = Pagination::from_query(PaginationQuery { current, page_size });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let status = parse_optional_i16_filter(status.as_deref(), "user status", None)?;
        let repo_query = UserListQuery { username, status, real_name, email };

        let (users, total) = UserRepository::list_users(pool, offset, limit, repo_query).await?;

        Ok((users.into_iter().map(UserItemResp::try_from).collect::<Result<Vec<_>, _>>()?, total))
    }

    /// Create user
    pub async fn create_user(pool: &PgPool, dto: CreateUserRequest) -> Result<i64, ServiceError> {
        tracing::debug!("Creating user: {}", dto.username);
        if UserRepository::username_exists(pool, &dto.username).await? {
            return Err(ServiceError::UsernameConflict);
        }
        if UserRepository::email_exists(pool, &dto.email).await? {
            return Err(ServiceError::EmailConflict);
        }
        let password_hash = PasswordUtils::hash_password(&dto.password)?;
        let create_cmd = CreateUserCommand {
            username: dto.username,
            email: dto.email,
            password_hash,
            real_name: dto.real_name,
            status: dto.status,
            role_ids: dto.role_ids,
        };

        let user_id = UserRepository::create_user(pool, &create_cmd).await?;

        Ok(user_id)
    }

    /// Update user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserPayload,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating user ID: {}", id);
        let user = UserRepository::find_user_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User id: {}", id)))?;
        if user.is_system {
            return Err(ServiceError::UserIsAdmin);
        }
        UserRepository::update_user(pool, id, &request.email, &request.real_name, &request.role_ids)
            .await
    }

    /// Delete user
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Deleting user ID: {}", id);
        let user = UserRepository::find_user_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User id: {}", id)))?;
        if user.is_system {
            return Err(ServiceError::UserIsAdmin);
        }
        UserRepository::soft_delete(pool, id).await?;

        Ok(())
    }

    /// Get user status options
    pub fn get_user_status_options() -> Vec<UserOptionResp> {
        vec![
            UserOptionResp { label: "Normal".to_string(), value: 1 },
            UserOptionResp { label: "Disabled".to_string(), value: 2 },
        ]
    }

    /// Get user options for dropdowns
    pub async fn get_user_options(
        pool: &PgPool,
        query: UserOptionsQuery,
    ) -> Result<Vec<UserOptionResp>, ServiceError> {
        tracing::debug!("Getting user options with query: {:?}", query);
        Ok(UserRepository::list_user_options(pool, query.status, query.q.as_deref(), query.limit)
            .await?
            .into_iter()
            .map(|(value, label)| UserOptionResp { label, value })
            .collect())
    }

    pub async fn update_user_password(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserPasswordPayload,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user password for user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id).await?;
        let password_hash = PasswordUtils::hash_password(&dto.password)?;
        UserRepository::update_user_password(pool, id, &password_hash).await
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserStatusPayload,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user status for user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id).await?;
        UserRepository::update_user_status(pool, id, dto.status).await
    }

    async fn ensure_user_is_mutable(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        let user = UserRepository::find_user_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User id: {}", id)))?;
        if user.is_system {
            return Err(ServiceError::UserIsAdmin);
        }
        Ok(())
    }
}
