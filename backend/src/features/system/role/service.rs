// Role business logic

use super::model::{
    CreateRoleRequest, RoleListResponse, RoleQueryParams, RoleResponse, UpdateRoleRequest,
};
use super::repo::RoleRepository;
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
        params: RoleQueryParams,
    ) -> Result<RoleListResponse, ServiceError> {
        let page = params.current.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * page_size;

        tracing::info!(
            "Retrieving role list: page={}, size={}, name={:?}, status={:?}",
            page,
            page_size,
            params.role_name,
            params.status
        );

        let roles = RoleRepository::find_with_pagination(
            pool,
            offset,
            page_size,
            params.role_name.as_deref(),
            params.status,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve roles from database: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let total = RoleRepository::count_roles(pool, params.role_name.as_deref(), params.status)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count roles from database: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        let mut role_responses = Vec::new();
        for role in roles {
            let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await.map_err(|e| {
                tracing::error!("Failed to retrieve menu IDs for role {}: {:?}", role.id, e);
                ServiceError::DatabaseQueryFailed
            })?;
            let mut role_response = RoleResponse::from(role);
            role_response.menu_ids = menu_ids;
            role_responses.push(role_response);
        }

        tracing::info!("Retrieved {} roles (total: {})", role_responses.len(), total);
        Ok(RoleListResponse { list: role_responses, total, page, page_size })
    }

    /// Get single role by ID with menu permissions
    pub async fn get_role_by_id(pool: &PgPool, id: i64) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Retrieving role: {}", id);

        let role = RoleRepository::find_by_id(pool, id).await.map_err(|e| {
            tracing::error!("Failed to retrieve role {} from database: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        match role {
            Some(role) => {
                let menu_ids =
                    RoleRepository::get_role_menu_ids(pool, role.id).await.map_err(|e| {
                        tracing::error!(
                            "Failed to retrieve menu IDs for role {}: {:?}",
                            role.id,
                            e
                        );
                        ServiceError::DatabaseQueryFailed
                    })?;
                let mut role_response = RoleResponse::from(role);
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
        request: CreateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Creating role: {}", request.role_name);

        // Check name conflict
        if RoleRepository::find_by_role_name(pool, &request.role_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check role name conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Role name exists: {}", request.role_name);
            return Err(ServiceError::RoleNameConflict);
        }

        let role = RoleRepository::create(pool, &request.role_name, request.status.unwrap_or(1))
            .await
            .map_err(|e| {
                tracing::error!("Failed to create role in database: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        // Set menu permissions if provided
        if !request.menu_ids.is_empty() {
            RoleRepository::set_role_menus(pool, role.id, &request.menu_ids).await.map_err(
                |e| {
                    tracing::error!("Failed to set role menus: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                },
            )?;
        }

        let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await.map_err(|e| {
            tracing::error!("Failed to retrieve menu IDs for new role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let mut role_response = RoleResponse::from(role);
        role_response.menu_ids = menu_ids;

        tracing::info!("Created role: {}", role_response.id);
        Ok(role_response)
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Updating role: {}", id);

        // Check role exists
        if RoleRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .is_none()
        {
            return Err(ServiceError::NotFound("Role".to_string()));
        }

        // Check name conflict if changing name
        if let Some(ref role_name) = request.role_name {
            if let Some(existing_role) = RoleRepository::find_by_role_name(pool, role_name)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?
            {
                if existing_role.id != id {
                    return Err(ServiceError::RoleNameConflict);
                }
            }
        }

        let updated_role =
            RoleRepository::update(pool, id, request.role_name.as_deref(), request.status)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        match updated_role {
            Some(role) => {
                // Update menu permissions if provided
                if let Some(menu_ids) = request.menu_ids {
                    RoleRepository::set_role_menus(pool, role.id, &menu_ids)
                        .await
                        .map_err(|_| ServiceError::DatabaseQueryFailed)?;
                }

                let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id)
                    .await
                    .map_err(|_| ServiceError::DatabaseQueryFailed)?;
                let mut role_response = RoleResponse::from(role);
                role_response.menu_ids = menu_ids;

                Ok(role_response)
            }
            None => Err(ServiceError::NotFound("Role".to_string())),
        }
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        // Check if role is still assigned to users
        let user_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_roles WHERE role_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        if user_count.0 > 0 {
            return Err(ServiceError::InvalidOperation(
                "Cannot delete a role that is still assigned to users.".to_string(),
            ));
        }

        let success = RoleRepository::soft_delete(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        if success { Ok(()) } else { Err(ServiceError::NotFound("Role".to_string())) }
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
                .await
                .map_err(|e| {
                    tracing::error!("Failed to retrieve role options from database: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;

        let options: Vec<OptionItem<i64>> = roles
            .into_iter()
            .map(|(id, role_name)| OptionItem { label: role_name, value: id })
            .collect();

        tracing::info!("Retrieved {} role options", options.len());
        Ok(options)
    }
}
