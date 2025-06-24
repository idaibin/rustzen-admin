// Role-related business logic (validation, combining repo methods, etc.) goes here.

use super::model::{
    CreateRoleRequest, RoleListResponse, RoleQueryParams, RoleResponse, UpdateRoleRequest,
};
use super::repo::RoleRepository;
use crate::common::api::{OptionItem, OptionsQuery};
use crate::common::error::ServiceError;
use axum::extract::Query;
use sqlx::PgPool;

/// A service for role-related operations.
pub struct RoleService;

impl RoleService {
    /// Retrieves a paginated list of roles
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Returns
    /// * `Result<RoleListResponse, ServiceError>` - Paginated role list or service error
    pub async fn get_role_list(
        pool: &PgPool,
        params: RoleQueryParams,
    ) -> Result<RoleListResponse, ServiceError> {
        let page = params.current.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * page_size;

        tracing::info!(
            "Retrieving role list with page: {}, page_size: {}, role_name: {:?}, status: {:?}",
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

        tracing::info!("Successfully retrieved {} roles (total: {})", role_responses.len(), total);
        Ok(RoleListResponse { list: role_responses, total, page, page_size })
    }

    /// Retrieves a single role by its ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Role ID
    ///
    /// # Returns
    /// * `Result<RoleResponse, ServiceError>` - Role response or service error
    pub async fn get_role_by_id(pool: &PgPool, id: i64) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Retrieving role with id: {}", id);

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
                tracing::info!("Successfully retrieved role with id: {}", id);
                Ok(role_response)
            }
            None => {
                tracing::warn!("Role with id {} not found", id);
                Err(ServiceError::NotFound("Role".to_string()))
            }
        }
    }

    /// Creates a new role
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Role creation request data
    ///
    /// # Returns
    /// * `Result<RoleResponse, ServiceError>` - Created role response or service error
    ///
    /// # Errors
    /// Returns `ServiceError::RoleNameConflict` if the role name already exists.
    pub async fn create_role(
        pool: &PgPool,
        request: CreateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Creating new role with name: {}", request.role_name);

        if RoleRepository::find_by_role_name(pool, &request.role_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check role name conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            tracing::warn!("Role name already exists: {}", request.role_name);
            return Err(ServiceError::RoleNameConflict);
        }

        let role = RoleRepository::create(pool, &request.role_name, request.status.unwrap_or(1))
            .await
            .map_err(|e| {
                tracing::error!("Failed to create role in database: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

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

        tracing::info!("Successfully created role with id: {}", role_response.id);
        Ok(role_response)
    }

    /// Updates an existing role
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Role ID to update
    /// * `request` - Role update request data
    ///
    /// # Returns
    /// * `Result<RoleResponse, ServiceError>` - Updated role response or service error
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if the role does not exist.
    /// Returns `ServiceError::RoleNameConflict` if the new role name is taken by another role.
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        tracing::info!("Updating role with id: {}", id);
        if RoleRepository::find_by_id(pool, id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?
            .is_none()
        {
            return Err(ServiceError::NotFound("Role".to_string()));
        }

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

    /// Deletes a role by its ID.
    /// # Errors
    ///
    /// Returns `ServiceError::InvalidOperation` if the role is still assigned to users.
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
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

    /// Sets menu permissions for a role
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `role_id` - Role ID to set menus for
    /// * `menu_ids` - List of menu IDs to assign to the role
    ///
    /// # Returns
    /// * `Result<(), ServiceError>` - Success or service error
    pub async fn set_role_menus(
        pool: &PgPool,
        role_id: i64,
        menu_ids: Vec<i64>,
    ) -> Result<(), ServiceError> {
        tracing::info!("Setting menus for role id: {}, menu_ids: {:?}", role_id, menu_ids);

        if RoleRepository::find_by_id(pool, role_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check role existence: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_none()
        {
            tracing::warn!("Role with id {} not found", role_id);
            return Err(ServiceError::NotFound("Role".to_string()));
        }

        RoleRepository::set_role_menus(pool, role_id, &menu_ids).await.map_err(|e| {
            tracing::error!("Failed to set role menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::info!("Successfully set {} menus for role id: {}", menu_ids.len(), role_id);
        Ok(())
    }

    /// Retrieves menu permissions for a role
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `role_id` - Role ID to get menus for
    ///
    /// # Returns
    /// * `Result<Vec<i64>, ServiceError>` - List of menu IDs or service error
    pub async fn get_role_menus(pool: &PgPool, role_id: i64) -> Result<Vec<i64>, ServiceError> {
        tracing::info!("Retrieving menus for role id: {}", role_id);

        if RoleRepository::find_by_id(pool, role_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check role existence: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_none()
        {
            tracing::warn!("Role with id {} not found", role_id);
            return Err(ServiceError::NotFound("Role".to_string()));
        }

        let menu_ids = RoleRepository::get_role_menu_ids(pool, role_id).await.map_err(|e| {
            tracing::error!("Failed to retrieve role menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::info!("Successfully retrieved {} menus for role id: {}", menu_ids.len(), role_id);
        Ok(menu_ids)
    }

    /// Retrieves role options for dropdown selections
    ///
    /// Returns simplified role data optimized for frontend dropdown components.
    /// Supports filtering by status, search term, and result limiting.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `query` - Query parameters (status, q, limit)
    ///
    /// # Returns
    /// * `Result<Vec<OptionItem<i64>>, ServiceError>` - List of option items or service error
    pub async fn get_role_options(
        pool: &PgPool,
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Retrieving role options with query: status={:?}, search={:?}, limit={:?}",
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

        tracing::info!("Successfully retrieved {} role options", options.len());
        Ok(options)
    }
}
