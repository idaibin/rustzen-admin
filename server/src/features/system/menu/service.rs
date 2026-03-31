use super::{
    repo::{MenuListQuery, MenuRepository},
    types::{CreateMenuRequest, MenuItemResp, MenuQuery, UpdateMenuPayload},
};
use crate::common::{
    api::{OptionItem, OptionsQuery},
    error::ServiceError,
};

use sqlx::PgPool;

pub struct MenuService;

impl MenuService {
    /// Get menu list as tree structure with optional filtering
    pub async fn list_menus(
        pool: &PgPool,
        query: MenuQuery,
    ) -> Result<(Vec<MenuItemResp>, i64), ServiceError> {
        tracing::info!("Fetching menu list with query: {:?}", query);

        let repo_query = MenuListQuery {
            name: query.name,
            code: query.code,
            status: query.status,
        };

        let menus = MenuRepository::list_menus(pool, repo_query).await?;

        let menu_responses: Vec<MenuItemResp> = menus.into_iter().map(MenuItemResp::from).collect();
        let count = menu_responses.len() as i64;

        Ok((menu_responses, count))
    }

    /// Create new menu with validation
    pub async fn create_menu(pool: &PgPool, request: CreateMenuRequest) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to create menu with name: {}", request.name);

        let menu_id = MenuRepository::create(
            pool,
            request.parent_id,
            &request.name,
            &request.code,
            request.menu_type,
            request.sort_order,
            request.status,
        )
        .await?;

        tracing::info!("Successfully created menu: {}", menu_id);
        Ok(menu_id)
    }

    /// Update existing menu with validation
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);

        let menu_id = MenuRepository::update(pool, id, &request)
        .await?;

        tracing::info!("Successfully updated menu: {}", menu_id);
        Ok(menu_id)
    }

    /// Delete menu with child validation
    pub async fn delete_menu(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete menu: {}", id);

        let success = MenuRepository::soft_delete(pool, id).await?;

        if success {
            tracing::info!("Successfully deleted menu: {}", id);
            Ok(())
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    /// Get menu options for dropdowns
    pub async fn get_menu_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Fetching menu options: {:?}", query);

        let menus = MenuRepository::list_menu_options(pool, query.q.as_deref(), query.limit).await?;

        let options: Vec<OptionItem<i64>> =
            menus.into_iter().map(|(id, name)| OptionItem { label: name, value: id }).collect();

        tracing::info!("Successfully retrieved {} menu options", options.len());
        Ok(options)
    }
}
