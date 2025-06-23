// Menu-related business logic (validation, combining repo methods, etc.) goes here.

use super::model::{
    CreateMenuRequest, MenuListResponse, MenuQueryParams, MenuResponse, UpdateMenuRequest,
};
use super::repo::MenuRepository;
use crate::common::api::{OptionItem, OptionsQuery, ServiceError};
use axum::extract::Query;
use sqlx::PgPool;
use std::collections::HashMap;

/// A service for menu-related operations.
pub struct MenuService;

impl MenuService {
    /// Retrieves a list of menus, optionally filtered, as a tree structure.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::DatabaseQueryFailed` if any database operation fails.
    pub async fn get_menu_list(
        pool: &PgPool,
        params: MenuQueryParams,
    ) -> Result<MenuListResponse, ServiceError> {
        tracing::info!("Fetching menu list with params: {:?}", params);

        let menus =
            MenuRepository::find_with_conditions(pool, params.title.as_deref(), params.status)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch menus from database: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;

        let total = MenuRepository::count_menus(pool, params.title.as_deref(), params.status)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count menus from database: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        let menu_responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();
        let menu_tree = Self::build_menu_tree(menu_responses);
        let response = MenuListResponse { list: menu_tree, total };

        Ok(response)
    }

    /// Retrieves a single menu by its ID.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::NotFound` if the menu does not exist.
    /// Returns `ServiceError::DatabaseQueryFailed` if the database operation fails.
    pub async fn get_menu_by_id(pool: &PgPool, id: i64) -> Result<MenuResponse, ServiceError> {
        tracing::info!("Fetching menu by ID: {}", id);

        let menu = MenuRepository::find_by_id(pool, id).await.map_err(|e| {
            tracing::error!("Failed to fetch menu {} from database: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        match menu {
            Some(menu) => Ok(MenuResponse::from(menu)),
            None => Err(ServiceError::NotFound("Menu".to_string())),
        }
    }

    /// Creates a new menu.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::NotFound` if the specified `parent_id` does not exist.
    /// Returns `ServiceError::MenuTitleConflict` if the title is already in use.
    /// Returns `ServiceError::DatabaseQueryFailed` if the database operation fails.
    pub async fn create_menu(
        pool: &PgPool,
        request: CreateMenuRequest,
    ) -> Result<MenuResponse, ServiceError> {
        tracing::info!("Attempting to create menu with title: {}", request.title);

        if let Some(parent_id) = request.parent_id {
            if MenuRepository::find_by_id(pool, parent_id)
                .await
                .map_err(|e| {
                    tracing::error!("DB error checking parent menu {}: {:?}", parent_id, e);
                    ServiceError::DatabaseQueryFailed
                })?
                .is_none()
            {
                return Err(ServiceError::NotFound("Parent menu".to_string()));
            }
        }

        // Check for title conflict
        if MenuRepository::find_by_title(&request.title, pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking for title conflict: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            return Err(ServiceError::MenuTitleConflict);
        }

        let menu = MenuRepository::create(
            pool,
            request.parent_id,
            &request.title,
            request.path.as_deref(),
            request.component.as_deref(),
            request.icon.as_deref(),
            request.sort_order.unwrap_or(0),
            request.status.unwrap_or(1),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create menu in database: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let menu_response = MenuResponse::from(menu);
        tracing::info!("Successfully created menu: {}", menu_response.id);
        Ok(menu_response)
    }

    /// Updates an existing menu.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::NotFound` if the menu or the new parent menu does not exist.
    /// Returns `ServiceError::InvalidOperation` if attempting to set a menu as its own parent.
    /// Returns `ServiceError::DatabaseQueryFailed` if any database operation fails.
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuRequest,
    ) -> Result<MenuResponse, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);

        if MenuRepository::find_by_id(pool, id)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking existence of menu {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_none()
        {
            return Err(ServiceError::NotFound("Menu".to_string()));
        }

        if let Some(parent_id) = request.parent_id {
            // Cannot be its own parent.
            if parent_id == id {
                return Err(ServiceError::InvalidOperation(
                    "Cannot set a menu as its own parent.".to_string(),
                ));
            }
            // If parent_id is not root (0), it must exist.
            if parent_id != 0
                && MenuRepository::find_by_id(pool, parent_id)
                    .await
                    .map_err(|e| {
                        tracing::error!("DB error checking parent menu {}: {:?}", parent_id, e);
                        ServiceError::DatabaseQueryFailed
                    })?
                    .is_none()
            {
                return Err(ServiceError::NotFound("Parent menu".to_string()));
            }
        }

        let updated_menu = MenuRepository::update(
            pool,
            id,
            request.parent_id,
            request.title.as_deref(),
            request.path.as_deref(),
            request.component.as_deref(),
            request.icon.as_deref(),
            request.sort_order,
            request.status,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to update menu {} in database: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        match updated_menu {
            Some(menu) => {
                let menu_response = MenuResponse::from(menu);
                tracing::info!("Successfully updated menu: {}", id);
                Ok(menu_response)
            }
            None => Err(ServiceError::NotFound("Menu".to_string())),
        }
    }

    /// Deletes a menu by its ID.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::InvalidOperation` if the menu has children.
    /// Returns `ServiceError::NotFound` if the menu does not exist.
    /// Returns `ServiceError::DatabaseQueryFailed` if any database operation fails.
    pub async fn delete_menu(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete menu: {}", id);
        let all_menus = MenuRepository::find_all(pool).await.map_err(|e| {
            tracing::error!("Failed to fetch all menus for delete check: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let has_children = all_menus.iter().any(|menu| menu.parent_id == Some(id));

        if has_children {
            return Err(ServiceError::InvalidOperation(
                "Cannot delete menu with children.".to_string(),
            ));
        }

        let success = MenuRepository::soft_delete(pool, id).await.map_err(|e| {
            tracing::error!("Failed to delete menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if success {
            tracing::info!("Successfully deleted menu: {}", id);
            Ok(())
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    /// Retrieves all menus assigned to a given list of role IDs.
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::DatabaseQueryFailed` if the database operation fails.
    pub async fn get_menus_by_role_ids(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<MenuResponse>, ServiceError> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        tracing::info!("Fetching menus for role IDs: {:?}", role_ids);

        let menus = MenuRepository::find_by_role_ids(pool, role_ids).await.map_err(|e| {
            tracing::error!("Failed to fetch menus by role IDs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let menu_responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();
        Ok(menu_responses)
    }

    /// Builds a tree structure from a flat list of menus.
    pub fn build_menu_tree(menus: Vec<MenuResponse>) -> Vec<MenuResponse> {
        let mut menu_map: HashMap<i64, MenuResponse> =
            menus.into_iter().map(|m| (m.id, m)).collect();

        let mut root_menus = Vec::new();
        let mut child_menus = Vec::new();

        for menu in menu_map.values() {
            if menu.parent_id.is_none() {
                root_menus.push(menu.id);
            } else {
                child_menus.push(menu.id);
            }
        }

        for id in child_menus {
            if let Some(menu) = menu_map.remove(&id) {
                if let Some(parent_id) = menu.parent_id {
                    if let Some(parent) = menu_map.get_mut(&parent_id) {
                        // To avoid cloning children again, we ensure children are built once.
                        if parent.children.is_empty() {
                            parent.children = Vec::new();
                        }
                        parent.children.push(menu);
                    }
                }
            }
        }

        let mut result: Vec<MenuResponse> =
            root_menus.into_iter().filter_map(|id| menu_map.remove(&id)).collect();

        // Sort all levels
        fn sort_recursive(menus: &mut Vec<MenuResponse>) {
            menus.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
            for menu in menus {
                if !menu.children.is_empty() {
                    sort_recursive(&mut menu.children);
                }
            }
        }

        sort_recursive(&mut result);
        result
    }

    /// Retrieves menu options for dropdown selections
    ///
    /// Returns simplified menu data optimized for frontend dropdown components.
    /// Supports filtering by status, search term, and result limiting.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `query` - Query parameters (status, q, limit)
    ///
    /// # Returns
    /// * `Result<Vec<OptionItem<i64>>, ServiceError>` - List of option items or service error
    pub async fn get_menu_options(
        pool: &PgPool,
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Fetching menu options with query: status={:?}, q={:?}, limit={:?}",
            query.status,
            query.q,
            query.limit
        );

        let status = query.status.as_deref().unwrap_or("enabled");
        let menus =
            MenuRepository::find_options(pool, Some(status), query.q.as_deref(), query.limit)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch menu options from database: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;

        let options: Vec<OptionItem<i64>> =
            menus.into_iter().map(|(id, title)| OptionItem { label: title, value: id }).collect();

        tracing::info!("Successfully retrieved {} menu options", options.len());
        Ok(options)
    }

    /// A private helper to recursively find and build children for the menu tree.
    fn find_children(
        menu_map: &HashMap<i64, MenuResponse>,
        parent_id: Option<i64>,
    ) -> Vec<MenuResponse> {
        let mut children: Vec<_> = menu_map
            .values()
            .filter(|menu| menu.parent_id == parent_id)
            .map(|menu| {
                let mut child = menu.clone();
                child.children = Self::find_children(menu_map, Some(menu.id));
                child
            })
            .collect();

        children.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
        children
    }
}
