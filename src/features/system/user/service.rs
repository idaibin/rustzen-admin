use super::{
    dto::{
        CreateUserDto, UpdateUserDto, UpdateUserPasswordDto, UpdateUserStatusDto, UserOptionsDto,
        UserQueryDto,
    },
    repo::UserRepository,
    vo::{UserItemVo, UserOptionVo},
};
use crate::{
    common::{error::ServiceError, pagination::Pagination},
    core::password::PasswordUtils,
};

use sqlx::PgPool;

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Get user list with pagination
    pub async fn get_user_list(
        pool: &PgPool,
        query: UserQueryDto,
    ) -> Result<(Vec<UserItemVo>, i64), ServiceError> {
        tracing::info!("Fetching user list with query: {:?}", query);

        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);

        let (users, total) =
            UserRepository::find_with_pagination(pool, offset, limit, query).await?;

        tracing::info!("Users: {:?}", users);
        let list = users.into_iter().map(UserItemVo::from).collect();

        Ok((list, total))
    }

    /// Create user
    pub async fn create_user(pool: &PgPool, dto: CreateUserDto) -> Result<i64, ServiceError> {
        tracing::debug!("Creating user: {}", dto.username);

        // Check if username already exists
        if UserRepository::username_exists(pool, &dto.username).await? {
            return Err(ServiceError::UsernameConflict);
        }

        // Check if email already exists
        if UserRepository::email_exists(pool, &dto.email).await? {
            return Err(ServiceError::EmailConflict);
        }

        // Hash password
        let password_hash = PasswordUtils::hash_password(&dto.password)?;

        // Create user DTO with hashed password
        let create_dto = CreateUserDto {
            username: dto.username,
            email: dto.email,
            password: password_hash,
            real_name: dto.real_name,
            status: dto.status,
            role_ids: dto.role_ids,
        };

        let user_id = UserRepository::create_user(pool, &create_dto).await?;

        Ok(user_id)
    }

    /// Update user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserDto,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating user ID: {}", id);

        // Update user
        let user_id = UserRepository::update_user(
            pool,
            id,
            &request.email,
            &request.real_name,
            &request.role_ids,
        )
        .await?;

        Ok(user_id)
    }

    /// Delete user
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Deleting user ID: {}", id);

        // Check if user exists
        if UserRepository::find_by_id(pool, id).await?.is_none() {
            return Err(ServiceError::NotFound("User".to_string()));
        }

        // Soft delete user
        UserRepository::soft_delete(pool, id).await?;

        Ok(())
    }

    /// Get user status options
    pub fn get_user_status_options() -> Vec<UserOptionVo> {
        vec![
            UserOptionVo { label: "Normal".to_string(), value: 1 },
            UserOptionVo { label: "Disabled".to_string(), value: 2 },
        ]
    }

    /// Get user options for dropdowns
    pub async fn get_user_options(
        pool: &PgPool,
        query: UserOptionsDto,
    ) -> Result<Vec<UserOptionVo>, ServiceError> {
        tracing::debug!("Getting user options with query: {:?}", query);

        let options =
            UserRepository::find_options(pool, query.status, query.q.as_deref(), query.limit)
                .await?;

        let user_options =
            options.into_iter().map(|(value, label)| UserOptionVo { label, value }).collect();

        Ok(user_options)
    }

    pub async fn update_user_password(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserPasswordDto,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user password for user ID: {}", id);

        let password_hash = PasswordUtils::hash_password(&dto.password)?;

        let result = UserRepository::update_user_password(pool, id, &password_hash).await?;

        Ok(result)
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserStatusDto,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user status for user ID: {}", id);

        let result = UserRepository::update_user_status(pool, id, dto.status).await?;

        Ok(result)
    }
}
