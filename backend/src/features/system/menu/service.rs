// Menu business logic

use super::model::{
    CreateMenuRequest, MenuListResponse, MenuQueryParams, MenuResponse, UpdateMenuRequest,
};
use super::repo::MenuRepository;
use crate::common::api::{OptionItem, OptionsQuery};
use crate::common::error::ServiceError;
use crate::features::system::user::repo::UserRepository;
use axum::extract::Query;
use sqlx::PgPool;
use std::collections::HashMap;

/// Menu service for business operations
pub struct MenuService;

impl MenuService {
    /// Get menu list as tree structure with optional filtering
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

    /// Get single menu by ID
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

    /// Create new menu with validation
    pub async fn create_menu(
        pool: &PgPool,
        request: CreateMenuRequest,
    ) -> Result<MenuResponse, ServiceError> {
        tracing::info!("Attempting to create menu with title: {}", request.title);

        // Check parent exists
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

        // Check title conflict
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

    /// Update existing menu with validation
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuRequest,
    ) -> Result<MenuResponse, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);

        // Check menu exists
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

        // Validate parent_id if provided
        if let Some(parent_id) = request.parent_id {
            // Cannot be its own parent
            if parent_id == id {
                return Err(ServiceError::InvalidOperation(
                    "Cannot set a menu as its own parent.".to_string(),
                ));
            }
            // Parent must exist (unless root)
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

    /// Delete menu with child validation
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

    /// Get menus by role IDs
    pub async fn get_menus_by_role_ids(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<MenuResponse>, ServiceError> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        tracing::info!("Fetching menus for role IDs: {:?}", role_ids);

        let menus = MenuRepository::find_menus_by_role(pool, role_ids).await.map_err(|e| {
            tracing::error!("Failed to fetch menus by role IDs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let menu_responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();
        Ok(menu_responses)
    }

    pub async fn get_permissions_by_menu(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        let role_ids = UserRepository::get_user_role_ids(pool, user_id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;

        let permissions =
            MenuRepository::find_permissions_by_role(pool, &role_ids).await.map_err(|e| {
                tracing::error!("Failed to fetch permissions by menu IDs: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(permissions)
    }

    /// Build hierarchical tree from flat menu list
    pub fn build_menu_tree(menus: Vec<MenuResponse>) -> Vec<MenuResponse> {
        let mut menu_map: HashMap<i64, MenuResponse> =
            menus.into_iter().map(|m| (m.id, m)).collect();

        let mut root_menus = Vec::new();
        let mut child_menus = Vec::new();

        for menu in menu_map.values() {
            if menu.parent_id == Some(0) {
                root_menus.push(menu.id);
            } else {
                child_menus.push(menu.id);
            }
        }

        for id in child_menus {
            if let Some(menu) = menu_map.remove(&id) {
                if let Some(parent_id) = menu.parent_id {
                    if let Some(parent) = menu_map.get_mut(&parent_id) {
                        if parent.children.is_none() {
                            parent.children = Some(Vec::new());
                        }
                        parent.children.as_mut().unwrap().push(menu);
                    }
                }
            }
        }

        let mut result: Vec<MenuResponse> =
            root_menus.into_iter().filter_map(|id| menu_map.remove(&id)).collect();

        // Sort all levels recursively
        fn sort_recursive(menus: &mut Vec<MenuResponse>) {
            menus.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
            for menu in menus {
                if let Some(children) = &mut menu.children {
                    if children.is_empty() {
                        menu.children = None; // 空数组改为 None
                    } else {
                        sort_recursive(children);
                    }
                }
            }
        }

        sort_recursive(&mut result);
        result
    }

    /// Get menu options for dropdowns
    pub async fn get_menu_options(
        pool: &PgPool,
        query: Query<OptionsQuery>,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!(
            "Fetching menu options: status={:?}, q={:?}, limit={:?}",
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
}
