#[cfg(feature = "full")]
use super::types::UserDashboardCounts;
use super::{
    repo::UserRepository,
    types::{
        CreateUserCommand, CreateUserRequest, UpdateUserPasswordPayload, UpdateUserPayload,
        UpdateUserStatusPayload, UserItemResp, UserListQuery, UserOptionResp, UserOptionsQuery,
        UserQuery,
    },
};
use crate::features::auth::session::SessionRepository;
use crate::{
    common::{
        error::ServiceError,
        pagination::{Pagination, PaginationQuery},
        query::parse_optional_i16_filter,
    },
    infra::password::PasswordUtils,
    infra::permission::PermissionService,
};
use rustzen_auth::{auth::AuthClaims, capability::SYSTEM_WILDCARD};

use sqlx::SqlitePool;

const OWNER_ROLE_CODE: &str = "owner";

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Get user list with pagination
    pub async fn list_users(
        pool: &SqlitePool,
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
        let mut users =
            users.into_iter().map(UserItemResp::try_from).collect::<Result<Vec<_>, _>>()?;
        let role_ids = users
            .iter()
            .flat_map(|user| user.roles.iter().map(|role| role.value))
            .collect::<Vec<_>>();
        let role_identities = UserRepository::list_role_identities_by_ids(pool, &role_ids)
            .await?
            .into_iter()
            .map(|(id, code, is_system)| (id, (code, is_system)))
            .collect::<std::collections::HashMap<_, _>>();
        for role in users.iter_mut().flat_map(|user| &mut user.roles) {
            if let Some((code, is_system)) = role_identities.get(&role.value) {
                role.code.clone_from(code);
                role.is_system = *is_system;
            }
        }

        Ok((users, total))
    }

    /// Create user
    pub async fn create_user(
        pool: &SqlitePool,
        current_user_id: i64,
        dto: CreateUserRequest,
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating user: {}", dto.username);
        Self::ensure_roles_are_assignable(pool, current_user_id, &dto.role_ids).await?;
        if !is_valid_user_status(dto.status.unwrap_or(1)) {
            return Err(ServiceError::InvalidUserStatus);
        }
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

        let user_id = UserRepository::create_user(pool, &create_cmd, actor).await?;
        PermissionService::refresh_all_user_permissions(pool).await?;

        Ok(user_id)
    }

    /// Update user
    pub async fn update_user(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        request: UpdateUserPayload,
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id, current_user_id).await?;
        Self::ensure_roles_are_assignable(pool, current_user_id, &request.role_ids).await?;
        if UserRepository::email_exists_for_other_user(pool, &request.email, id).await? {
            return Err(ServiceError::EmailConflict);
        }
        let user_id = UserRepository::update_user(
            pool,
            id,
            &request.email,
            &request.real_name,
            &request.role_ids,
            actor,
        )
        .await?;
        PermissionService::refresh_all_user_permissions(pool).await?;
        Ok(user_id)
    }

    /// Delete user
    pub async fn delete_user(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        actor: &AuthClaims,
    ) -> Result<(), ServiceError> {
        tracing::debug!("Deleting user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id, current_user_id).await?;
        if !UserRepository::soft_delete_authorized(pool, id, actor).await? {
            return Err(ServiceError::NotFound(format!("User id: {}", id)));
        }
        PermissionService::refresh_all_user_permissions(pool).await?;

        Ok(())
    }

    /// Get user status options
    pub fn get_user_status_options() -> Vec<UserOptionResp> {
        vec![
            UserOptionResp { label: "启用".to_string(), value: 1 },
            UserOptionResp { label: "禁用".to_string(), value: 2 },
            UserOptionResp { label: "待审核".to_string(), value: 3 },
            UserOptionResp { label: "已锁定".to_string(), value: 4 },
        ]
    }

    /// Get user options for dropdowns
    pub async fn get_user_options(
        pool: &SqlitePool,
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
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        dto: UpdateUserPasswordPayload,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user password for user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id, current_user_id).await?;
        let password_hash = PasswordUtils::hash_password(&dto.password)?;
        UserRepository::update_user_password(pool, id, &password_hash, actor).await
    }

    pub async fn update_user_status(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        dto: UpdateUserStatusPayload,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user status for user ID: {}", id);
        Self::ensure_user_is_mutable(pool, id, current_user_id).await?;
        if !is_valid_user_status(dto.status) {
            return Err(ServiceError::InvalidUserStatus);
        }
        let updated = UserRepository::update_user_status(pool, id, dto.status, actor).await?;
        PermissionService::refresh_all_user_permissions(pool).await?;
        Ok(updated)
    }

    pub async fn revoke_user_sessions(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        Self::ensure_user_is_mutable(pool, id, current_user_id).await?;
        let updated = SessionRepository::revoke_all_authorized(
            pool,
            id,
            actor,
            rustzen_auth::capability::system_user::RESET_PASSWORD,
            chrono::Utc::now().timestamp(),
        )
        .await?;
        if updated {
            PermissionService::clear_user_cache(id);
        }
        Ok(updated)
    }

    #[cfg(feature = "full")]
    pub async fn dashboard_counts(pool: &SqlitePool) -> Result<UserDashboardCounts, ServiceError> {
        UserRepository::dashboard_counts(pool).await
    }

    async fn ensure_user_is_mutable(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
    ) -> Result<(), ServiceError> {
        let user = UserRepository::find_user_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User id: {}", id)))?;
        let actor_is_owner = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_permissions WHERE user_id = ? AND menu_code = ?)",
        )
        .bind(current_user_id)
        .bind(SYSTEM_WILDCARD)
        .fetch_one(pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Failed to verify current owner authority");
            ServiceError::DatabaseQueryFailed
        })?;
        if user.is_system && !actor_is_owner {
            return Err(ServiceError::UserIsAdmin);
        }
        Ok(())
    }

    async fn ensure_roles_are_assignable(
        pool: &SqlitePool,
        current_user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        ensure_user_has_roles(role_ids)?;

        let roles = UserRepository::list_role_identities_by_ids(pool, role_ids).await?;
        if roles.len() != role_ids.iter().copied().collect::<std::collections::HashSet<_>>().len() {
            return Err(ServiceError::NotFound("Role".to_string()));
        }

        let actor_is_owner = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_permissions WHERE user_id=? AND menu_code=?)",
        )
        .bind(current_user_id)
        .bind(SYSTEM_WILDCARD)
        .fetch_one(pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, "checking current owner-role authority");
            ServiceError::DatabaseQueryFailed
        })?;
        if roles.iter().any(|(_, code, _)| role_requires_owner_permission(code)) && !actor_is_owner
        {
            return Err(ServiceError::InvalidOperation(
                "Owner role can only be assigned by an owner user.".to_string(),
            ));
        }

        Ok(())
    }
}

fn ensure_user_has_roles(role_ids: &[i64]) -> Result<(), ServiceError> {
    if role_ids.is_empty() {
        return Err(ServiceError::InvalidOperation(
            "A user requires at least one role.".to_string(),
        ));
    }
    Ok(())
}

fn role_requires_owner_permission(role_code: &str) -> bool {
    role_code == OWNER_ROLE_CODE
}

fn is_valid_user_status(status: i16) -> bool {
    matches!(status, 1..=4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_role_requires_owner_permission() {
        assert!(role_requires_owner_permission("owner"));
        assert!(!role_requires_owner_permission("admin"));
        assert!(!role_requires_owner_permission("viewer"));
    }

    #[test]
    fn users_require_at_least_one_role() {
        let error = ensure_user_has_roles(&[]).expect_err("empty role assignment should fail");
        assert!(matches!(error, ServiceError::InvalidOperation(_)));
        assert!(ensure_user_has_roles(&[1]).is_ok());
    }

    #[test]
    fn validates_supported_user_status_values() {
        assert!(is_valid_user_status(1));
        assert!(is_valid_user_status(4));
        assert!(!is_valid_user_status(0));
        assert!(!is_valid_user_status(5));
    }
}
