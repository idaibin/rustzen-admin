use crate::common::{
    error::ServiceError,
    query::{fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::types::{MenuListQuery, MenuRow};

/// Menu data access layer
pub struct MenuRepository;

impl MenuRepository {
    fn format_query(query: &MenuListQuery, query_builder: &mut QueryBuilder<Sqlite>) {
        push_ilike(query_builder, "name", query.name.as_deref());
        push_ilike(query_builder, "code", query.code.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Queries menus based on conditions
    pub async fn list_menus(
        pool: &SqlitePool,
        query: MenuListQuery,
    ) -> Result<Vec<MenuRow>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, parent_id, parent_code, name, code, menu_type, status, is_system, is_manual, sort_order, path, icon, module_id, module_menu_code, is_active, created_at, updated_at FROM menus WHERE 1=1 AND is_active = TRUE AND deleted_at IS NULL",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("sort_order ASC, id ASC"),
            None,
            None,
        )
        .await
    }

    pub async fn list_module_menu_inventory(
        pool: &SqlitePool,
    ) -> Result<Vec<MenuRow>, ServiceError> {
        sqlx::query_as::<_, MenuRow>(
            "SELECT id,0 AS parent_id,NULL AS parent_code,name,code,2 AS menu_type,status,
                    TRUE AS is_system,is_manual,sort_order,path,icon,module_id,module_menu_code,
                    is_active,created_at,updated_at
             FROM module_navigation WHERE is_active=TRUE ORDER BY sort_order,id",
        )
        .fetch_all(pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, "loading module navigation inventory");
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn update_navigation(
        pool: &SqlitePool,
        id: i64,
        name: &str,
        icon: Option<&str>,
        sort_order: i16,
        status: i16,
    ) -> Result<i64, ServiceError> {
        sqlx::query_scalar(
            "UPDATE module_navigation
             SET name=?,icon=COALESCE(?,icon),sort_order=?,status=?,is_manual=TRUE,updated_at=?
             WHERE id=? AND is_active=TRUE RETURNING id",
        )
        .bind(name)
        .bind(icon)
        .bind(sort_order)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, id, "updating module navigation");
            ServiceError::DatabaseQueryFailed
        })?
        .ok_or_else(|| ServiceError::NotFound("Module navigation".into()))
    }

    /// Returns whether the menu is a system built-in menu.
    pub async fn identity(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<(bool, Option<String>, Option<String>)>, ServiceError> {
        sqlx::query_as::<_, (bool, Option<String>, Option<String>)>(
            "SELECT is_system, module_id, module_menu_code
             FROM menus WHERE id = ? AND is_active = TRUE AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching menu {} system flag: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Disable a menu.
    pub async fn disable(pool: &SqlitePool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE menus SET status = 2, updated_at = ? WHERE id = ? AND is_system = false AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error disabling menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieves menu list for Options API
    pub async fn list_menu_options(
        pool: &SqlitePool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String, String, bool, Option<String>, Option<String>)>, ServiceError>
    {
        fetch_with_filters(
            pool,
            "SELECT id, name, code, is_system, module_id, module_menu_code FROM menus WHERE is_active = TRUE AND deleted_at IS NULL",
            |query_builder| {
                push_ilike(query_builder, "name", search_query);
            },
            Some("sort_order ASC, name ASC"),
            limit,
            None,
        )
        .await
    }
}
