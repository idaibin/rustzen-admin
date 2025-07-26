use super::{
    dto::{CreateRoleDto, RoleQueryDto, UpdateRoleDto},
    repo::RoleRepository,
    vo::RoleItemVo,
};
use crate::common::{
    api::{OptionItem, OptionsQuery},
    error::ServiceError,
    pagination::Pagination,
};

use sqlx::PgPool;

pub struct RoleService;

impl RoleService {
    /// Get paginated role list with filtering
    pub async fn get_role_list(
        pool: &PgPool,
        query: RoleQueryDto,
    ) -> Result<(Vec<RoleItemVo>, i64), ServiceError> {
        tracing::info!("Fetching role list with query: {:?}", query);

        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);

        let (roles, total) =
            RoleRepository::find_with_pagination(pool, offset, limit, query).await?;

        let list = roles.into_iter().map(RoleItemVo::from).collect();

        Ok((list, total))
    }

    /// Create new role with validation
    pub async fn create_role(pool: &PgPool, request: CreateRoleDto) -> Result<(), ServiceError> {
        tracing::info!("Creating role: {}", request.name);

        let id: i64 = RoleRepository::create(
            pool,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;

        tracing::info!("Created role: {}", id);
        Ok(())
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRoleDto,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating role: {}", id);

        let new_id: i64 = RoleRepository::update(
            pool,
            id,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;

        tracing::info!("Updated role: {}", new_id);
        Ok(())
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);

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

    /// Get role options for dropdowns
    pub async fn get_role_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Retrieving role options: {:?}", query);

        let roles = RoleRepository::find_options(pool, query.q.as_deref(), query.limit).await?;

        let options: Vec<OptionItem<i64>> =
            roles.into_iter().map(|(id, name)| OptionItem { label: name, value: id }).collect();

        tracing::info!("Retrieved {} role options", options.len());
        Ok(options)
    }
}
