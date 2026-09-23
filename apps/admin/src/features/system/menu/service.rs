use super::{
    repo::MenuRepository,
    types::{MenuItemResp, MenuListQuery, MenuOptionResp, MenuQuery, UpdateMenuPayload},
};
use crate::common::{api::OptionsQuery, error::ServiceError, query::parse_optional_i16_filter};
use crate::infra::permission::PermissionService;
use rustzen_auth::auth::AuthClaims;
use rustzen_auth::capability::SYSTEM_WILDCARD;

use sqlx::SqlitePool;

pub struct MenuService;

impl MenuService {
    /// Get menu list as tree structure with optional filtering
    pub async fn list_menus(
        pool: &SqlitePool,
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

    pub async fn list_module_menu_inventory(
        pool: &SqlitePool,
    ) -> Result<Vec<MenuItemResp>, ServiceError> {
        Ok(MenuRepository::list_module_menu_inventory(pool)
            .await?
            .into_iter()
            .map(MenuItemResp::from)
            .collect())
    }

    /// Update existing menu with validation
    #[cfg(test)]
    pub async fn update_menu(
        pool: &SqlitePool,
        id: i64,
        request: UpdateMenuPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);
        let _module_menu_guard = PermissionService::lock_module_menu_mutation().await;
        let menu_id = MenuRepository::update_navigation(
            pool,
            id,
            &request.name,
            request.icon.as_deref().filter(|icon| !icon.trim().is_empty()),
            request.sort_order,
            request.status,
        )
        .await?;
        Ok(menu_id)
    }

    pub async fn update_menu_authorized(
        pool: &SqlitePool,
        id: i64,
        request: UpdateMenuPayload,
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        let _module_menu_guard = PermissionService::lock_module_menu_mutation().await;
        MenuRepository::update_navigation_authorized(pool, id, &request, actor).await
    }

    pub async fn delete_menu_authorized(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
        actor: &AuthClaims,
    ) -> Result<(), ServiceError> {
        Self::ensure_menu_is_mutable(pool, id, current_user_id).await?;
        if MenuRepository::disable_authorized(pool, id, actor).await? {
            PermissionService::refresh_all_user_permissions(pool).await?;
            Ok(())
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    async fn ensure_menu_is_mutable(
        pool: &SqlitePool,
        id: i64,
        current_user_id: i64,
    ) -> Result<(), ServiceError> {
        match MenuRepository::identity(pool, id).await? {
            Some((true, _, _)) => {
                let is_owner = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM user_permissions WHERE user_id=? AND menu_code=?)",
                )
                .bind(current_user_id)
                .bind(SYSTEM_WILDCARD)
                .fetch_one(pool)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "checking current menu owner authority");
                    ServiceError::DatabaseQueryFailed
                })?;
                if is_owner { Ok(()) } else { Err(ServiceError::MenuIsSystem) }
            }
            Some((false, _, _)) => Ok(()),
            None => Err(ServiceError::NotFound(format!("Menu id: {}", id))),
        }
    }

    /// Get menu options for dropdowns
    pub async fn get_menu_options(
        pool: &SqlitePool,
        query: OptionsQuery,
    ) -> Result<Vec<MenuOptionResp>, ServiceError> {
        tracing::info!("Fetching menu options: {:?}", query);
        Ok(MenuRepository::list_menu_options(pool, query.q.as_deref(), query.limit)
            .await?
            .into_iter()
            .map(|(id, name, code, is_system, module_id, module_menu_code)| MenuOptionResp {
                label: name,
                value: id,
                code,
                is_system,
                module_id,
                module_menu_code,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::MenuService;
    use crate::common::error::ServiceError;
    use crate::features::system::menu::types::UpdateMenuPayload;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn core_capability_rows_cannot_be_redefined_through_menu_updates() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let menu_id: i64 = sqlx::query_scalar("SELECT id FROM menus WHERE code = '*'")
            .fetch_one(&pool)
            .await
            .expect("core capability row");

        let result = MenuService::update_menu(
            &pool,
            menu_id,
            UpdateMenuPayload {
                name: "Escalated".to_string(),
                sort_order: 1,
                status: 1,
                icon: None,
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
        let code: String = sqlx::query_scalar("SELECT code FROM menus WHERE id = ?")
            .bind(menu_id)
            .fetch_one(&pool)
            .await
            .expect("unchanged capability row");
        assert_eq!(code, "*");
    }

    #[tokio::test]
    async fn unsupported_manual_capability_rows_cannot_be_updated() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let menu_id: i64 = sqlx::query_scalar(
            "INSERT INTO menus
             (parent_id, name, code, menu_type, status, is_system, is_manual, sort_order)
             VALUES (0, 'Unsupported manual', 'unsupported:manual', 2, 1, FALSE, TRUE, 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("unsupported manual capability row");

        let result = MenuService::update_menu(
            &pool,
            menu_id,
            UpdateMenuPayload {
                name: "Changed".to_string(),
                sort_order: 99,
                status: 2,
                icon: Some("lock".to_string()),
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
        let row: (String, i32, i16, Option<String>) =
            sqlx::query_as("SELECT name, sort_order, status, icon FROM menus WHERE id = ?")
                .bind(menu_id)
                .fetch_one(&pool)
                .await
                .expect("unchanged unsupported manual capability row");
        assert_eq!(row, ("Unsupported manual".to_string(), 1, 1, None));
    }

    #[tokio::test]
    async fn module_menu_inventory_keeps_disabled_active_rows_available_for_reenable() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let menu_id: i64 = sqlx::query_scalar(
            "INSERT INTO module_navigation
             (name,code,status,is_manual,sort_order,path,icon,module_id,module_menu_code)
             VALUES ('Monitor','monitor:view',2,TRUE,1,'/monitoring','monitor','monitor','monitor')
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("module menu");

        let inventory =
            MenuService::list_module_menu_inventory(&pool).await.expect("module menu inventory");
        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0].id, menu_id);
        assert_eq!(inventory[0].status, 2);

        MenuService::update_menu(
            &pool,
            menu_id,
            UpdateMenuPayload {
                name: "Monitor".to_string(),
                sort_order: 1,
                status: 1,
                icon: Some("monitor".to_string()),
            },
        )
        .await
        .expect("re-enable module menu");
        let status: i16 = sqlx::query_scalar("SELECT status FROM module_navigation WHERE id = ?")
            .bind(menu_id)
            .fetch_one(&pool)
            .await
            .expect("re-enabled status");
        assert_eq!(status, 1);
    }
}
