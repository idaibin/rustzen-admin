use super::{
    repo::{RoleListQuery, RoleRepository},
    types::{CreateRoleRequest, RoleItemResp, RoleQuery, UpdateRolePayload},
};
use crate::common::{
    api::{OptionItem, OptionsQuery},
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
    query::parse_optional_i16_filter,
};

use sqlx::PgPool;

pub struct RoleService;

impl RoleService {
    /// Get paginated role list with filtering
    pub async fn list_roles(
        pool: &PgPool,
        query: RoleQuery,
    ) -> Result<(Vec<RoleItemResp>, i64), ServiceError> {
        tracing::info!("Fetching role list with query: {:?}", query);

        let RoleQuery { current, page_size, role_name, role_code, status } = query;
        let pagination = Pagination::from_query(PaginationQuery { current, page_size });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let status = parse_optional_i16_filter(status.as_deref(), "role status", None)?;
        let repo_query = RoleListQuery {
            role_name,
            role_code,
            status,
        };

        let (roles, total) = RoleRepository::list_roles(pool, offset, limit, repo_query).await?;

        Ok((
            roles
                .into_iter()
                .map(RoleItemResp::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            total,
        ))
    }

    /// Create new role with validation
    pub async fn create_role(pool: &PgPool, request: CreateRoleRequest) -> Result<(), ServiceError> {
        tracing::info!("Creating role: {}", request.name);
        RoleRepository::create(
            pool,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;
        Ok(())
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRolePayload,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating role: {}", id);
        Self::ensure_role_is_mutable(pool, id).await?;
        RoleRepository::update(
            pool,
            id,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;
        Ok(())
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);
        Self::ensure_role_is_mutable(pool, id).await?;

        // Check if role is still assigned to users
        let user_count = RoleRepository::get_role_user_count(pool, id).await?;
        if user_count > 0 {
            tracing::warn!("Cannot delete role {} - still assigned to {} users", id, user_count);
            return Err(ServiceError::InvalidOperation(format!(
                "Cannot delete role '{}' - it is still assigned to {} user(s). Please remove all user assignments before deleting the role.",
                id, user_count
            )));
        }

        // Perform the deletion
        let success = RoleRepository::soft_delete(pool, id).await?;

        if success {
            tracing::info!("Successfully deleted role: {}", id);
            Ok(())
        } else {
            tracing::warn!("Role not found during deletion: {}", id);
            Err(ServiceError::NotFound("Role".to_string()))
        }
    }

    async fn ensure_role_is_mutable(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        match RoleRepository::is_system_role(pool, id).await? {
            Some(true) => Err(ServiceError::RoleIsSystem),
            Some(false) => Ok(()),
            None => Err(ServiceError::NotFound(format!("Role id: {}", id))),
        }
    }

    /// Get role options for dropdowns
    pub async fn get_role_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Retrieving role options: {:?}", query);
        Ok(
            RoleRepository::list_role_options(pool, query.q.as_deref(), query.limit)
                .await?
                .into_iter()
                .map(|(id, name)| OptionItem { label: name, value: id })
                .collect(),
        )
    }
}
