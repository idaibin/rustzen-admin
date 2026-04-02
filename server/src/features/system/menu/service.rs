use super::{
    repo::{MenuListQuery, MenuRepository},
    types::{CreateMenuRequest, MenuItemResp, MenuQuery, UpdateMenuPayload},
};
use crate::common::{
    api::{OptionItem, OptionsQuery},
    error::ServiceError,
    query::parse_optional_i16_filter,
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

        let MenuQuery { name, code, status } = query;
        let status = parse_optional_i16_filter(status.as_deref(), "menu status", None)?;
        let repo_query = MenuListQuery { name, code, status };

        let menus = MenuRepository::list_menus(pool, repo_query).await?;
        let menu_responses: Vec<MenuItemResp> = menus.into_iter().map(MenuItemResp::from).collect();
        let count = menu_responses.len() as i64;
        Ok((menu_responses, count))
    }

    /// Create new menu with validation
    pub async fn create_menu(
        pool: &PgPool,
        request: CreateMenuRequest,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to create menu with name: {}", request.name);
        MenuRepository::create(
            pool,
            request.parent_id,
            &request.name,
            &request.code,
            request.menu_type,
            request.sort_order,
            request.status,
        )
        .await
    }

    /// Update existing menu with validation
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);
        Self::ensure_menu_is_mutable(pool, id).await?;
        MenuRepository::update(pool, id, &request).await
    }

    /// Delete menu with child validation
    pub async fn delete_menu(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete menu: {}", id);

        if MenuRepository::soft_delete(pool, id).await? {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    async fn ensure_menu_is_mutable(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        match MenuRepository::is_system_menu(pool, id).await? {
            Some(true) => Err(ServiceError::MenuIsSystem),
            Some(false) => Ok(()),
            None => Err(ServiceError::NotFound(format!("Menu id: {}", id))),
        }
    }

    /// Get menu options for dropdowns
    pub async fn get_menu_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Fetching menu options: {:?}", query);
        Ok(MenuRepository::list_menu_options(pool, query.q.as_deref(), query.limit)
            .await?
            .into_iter()
            .map(|(id, name)| OptionItem { label: name, value: id })
            .collect())
    }
}
