use super::{
    dto::{CreateUserDto, UpdateUserDto, UserOptionsDto, UserQueryDto},
    entity::UserWithRolesEntity,
    repo::UserRepository,
    vo::{RoleVo, UserDetailVo, UserListVo, UserOptionVo, UserStatusOptionVo},
};
use crate::common::error::ServiceError;
use crate::core::password::PasswordUtils;
use sqlx::PgPool;

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Get user list with pagination
    pub async fn get_user_list(
        pool: &PgPool,
        query: UserQueryDto,
    ) -> Result<(Vec<UserListVo>, i64), ServiceError> {
        let page = query.current.unwrap_or(1);
        let limit = query.page_size.unwrap_or(10);
        let offset = (page - 1) * limit;

        tracing::debug!("Getting user list: page={}, size={}", page, limit);

        let (users, total) =
            UserRepository::find_with_pagination(pool, offset, limit, &query).await?;

        let list = users.into_iter().map(|u| Self::to_user_list_vo(u)).collect();

        Ok((list, total))
    }

    /// Get user by ID
    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<UserDetailVo, ServiceError> {
        tracing::debug!("Getting user by ID: {}", id);

        let user_with_roles = UserRepository::find_by_id(pool, id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;
        Ok(Self::to_user_detail_vo(user_with_roles))
    }

    /// Create user
    pub async fn create_user(pool: &PgPool, dto: CreateUserDto) -> Result<bool, ServiceError> {
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

        UserRepository::create_user(pool, &create_dto).await?;

        Ok(true)
    }

    /// Update user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserDto,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user ID: {}", id);

        // Check if user exists
        let existing_user = UserRepository::find_by_id(pool, id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;

        // Check email uniqueness if email is being updated
        if let Some(ref email) = dto.email {
            if email != &existing_user.email {
                if UserRepository::email_exists(pool, email).await? {
                    return Err(ServiceError::EmailConflict);
                }
            }
        }

        let password_hash = if let Some(password) = dto.password {
            Some(PasswordUtils::hash_password(&password)?)
        } else {
            None
        };

        // Update user
        UserRepository::update_user(pool, id, dto.email, dto.real_name, dto.status, password_hash)
            .await?;

        // Update roles if provided
        if let Some(role_ids) = dto.role_ids {
            UserRepository::set_user_roles(pool, id, &role_ids).await?;
        }

        Ok(true)
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
    pub fn get_user_status_options() -> Vec<UserStatusOptionVo> {
        vec![
            UserStatusOptionVo { label: "Normal".to_string(), value: 1 },
            UserStatusOptionVo { label: "Disabled".to_string(), value: 2 },
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

    /// Convert UserWithRoles to UserListVo
    fn to_user_list_vo(user: UserWithRolesEntity) -> UserListVo {
        UserListVo {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: user.status,
            last_login_at: user.last_login_at,
            roles: serde_json::from_value::<Vec<RoleVo>>(user.roles).unwrap_or_default(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    /// Convert UserWithRoles to UserDetailVo
    fn to_user_detail_vo(user: UserWithRolesEntity) -> UserDetailVo {
        UserDetailVo {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: user.status,
            roles: serde_json::from_value::<Vec<RoleVo>>(user.roles).unwrap_or_default(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
