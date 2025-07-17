// Role business logic

use super::dto::{CreateRoleDto, RoleQueryDto, UpdateRoleDto};
use super::entity::RoleWithMenuEntity;
use super::repo::RoleRepository;
use super::vo::{MenuItemVo, RoleDetailVo};
use crate::common::api::{OptionItem, OptionsQuery};
use crate::common::error::ServiceError;
use axum::extract::Query;
use sqlx::PgPool;

/// Role service for business operations
pub struct RoleService;

impl RoleService {
    /// Get paginated role list with filtering
    pub async fn get_role_list(
        pool: &PgPool,
        query: RoleQueryDto,
    ) -> Result<(Vec<RoleDetailVo>, i64), ServiceError> {
        let page = query.current.unwrap_or(1).max(1);
        let limit = query.page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * limit;

        tracing::info!("Retrieving role list: page={}, size={}", page, limit,);

        let (roles, total) =
            RoleRepository::find_with_pagination(pool, offset, limit, query).await?;

        // let mut role_responses = Vec::new();
        // for role in roles {
        //     let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
        //     let mut role_response = RoleDetailVo::from(role);
        //     role_response.menu_ids = menu_ids;
        //     role_responses.push(role_response);
        // }
        let list = roles.into_iter().map(|u| Self::to_role_list_vo(u)).collect();

        Ok((list, total))
    }

    /// Get single role by ID with menu permissions
    pub async fn get_role_by_id(pool: &PgPool, id: i64) -> Result<RoleDetailVo, ServiceError> {
        tracing::info!("Retrieving role: {}", id);

        let role = RoleRepository::find_by_id(pool, id).await?;

        match role {
            Some(role) => {
                let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
                let mut role_response = RoleDetailVo::from(role);
                role_response.menu_ids = menu_ids;
                tracing::info!("Retrieved role: {}", id);
                Ok(role_response)
            }
            None => {
                tracing::warn!("Role not found: {}", id);
                Err(ServiceError::NotFound("Role".to_string()))
            }
        }
    }

    /// Create new role with validation
    pub async fn create_role(
        pool: &PgPool,
        request: CreateRoleDto,
    ) -> Result<RoleDetailVo, ServiceError> {
        tracing::info!("Creating role: {}", request.role_name);

        // Check name conflict
        if RoleRepository::find_by_role_name(pool, &request.role_name).await?.is_some() {
            tracing::warn!("Role name exists: {}", request.role_name);
            return Err(ServiceError::RoleNameConflict);
        }

        let role = RoleRepository::create(
            pool,
            &request.role_name,
            &request.role_code,
            request.description.as_deref(),
            request.status.unwrap_or(1),
        )
        .await?;

        // Set menu permissions if provided
        if !request.menu_ids.is_empty() {
            RoleRepository::set_role_menus(pool, role.id, &request.menu_ids).await?;
        }

        let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
        let mut role_response = RoleDetailVo::from(role);
        role_response.menu_ids = menu_ids;

        tracing::info!("Created role: {}", role_response.id);
        Ok(role_response)
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRoleDto,
    ) -> Result<RoleDetailVo, ServiceError> {
        tracing::info!("Updating role: {}", id);

        // Check role exists
        if RoleRepository::find_by_id(pool, id).await?.is_none() {
            return Err(ServiceError::NotFound("Role".to_string()));
        }

        // Check name conflict if changing name
        if let Some(ref role_name) = request.role_name {
            if let Some(existing_role) = RoleRepository::find_by_role_name(pool, role_name).await? {
                if existing_role.id != id {
                    return Err(ServiceError::RoleNameConflict);
                }
            }
        }

        let updated_role = RoleRepository::update(
            pool,
            id,
            request.role_name.as_deref(),
            request.role_code.as_deref(),
            request.description.as_deref(),
            request.status,
        )
        .await?;

        match updated_role {
            Some(role) => {
                // Update menu permissions if provided
                if let Some(menu_ids) = request.menu_ids {
                    RoleRepository::set_role_menus(pool, role.id, &menu_ids).await?;
                }

                let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
                let mut role_response = RoleDetailVo::from(role);
                role_response.menu_ids = menu_ids;

                Ok(role_response)
            }
            None => Err(ServiceError::NotFound("Role".to_string())),
        }
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);

        // First check if role exists
        let role = RoleRepository::find_by_id(pool, id).await?;

        let existing_role = role.ok_or(ServiceError::NotFound("Role".to_string()))?;

        if existing_role.is_system.unwrap_or(false) {
            tracing::warn!("Attempted to delete system role: {}", id);
            return Err(ServiceError::InvalidOperation("Cannot delete system roles".to_string()));
        }

        if existing_role.status == 2 {
            tracing::warn!("Attempted to delete disabled role: {}", id);
            return Err(ServiceError::InvalidOperation("Cannot delete disabled roles".to_string()));
        }

        // Check if role is still assigned to users
        let user_count = RoleRepository::get_role_user_count(pool, id).await?;

        if user_count > 0 {
            tracing::warn!("Cannot delete role {} - still assigned to {} users", id, user_count);
            return Err(ServiceError::InvalidOperation(format!(
                "Cannot delete role '{}' - it is still assigned to {} user(s). Please remove all user assignments before deleting the role.",
                existing_role.role_name, user_count
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
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Retrieving role options: status={:?}, search={:?}, limit={:?}",
            query.status,
            query.q,
            query.limit
        );

        let status = query.status.as_deref().unwrap_or("enabled");
        let roles =
            RoleRepository::find_options(pool, Some(status), query.q.as_deref(), query.limit)
                .await?;

        let options: Vec<OptionItem<i64>> = roles
            .into_iter()
            .map(|(id, role_name)| OptionItem { label: role_name, value: id })
            .collect();

        tracing::info!("Retrieved {} role options", options.len());
        Ok(options)
    }

    /// Convert UserWithRoles to UserListVo
    fn to_role_list_vo(user: RoleWithMenuEntity) -> RoleDetailVo {
        RoleDetailVo {
            id: user.id,
            role_name: user.role_name,
            role_code: user.role_code,
            description: user.description,
            status: user.status,
            menu_ids: serde_json::from_value::<Vec<MenuItemVo>>(user.menus)
                .map(|vec| vec.into_iter().map(|v| v.id).collect())
                .unwrap_or_default(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
