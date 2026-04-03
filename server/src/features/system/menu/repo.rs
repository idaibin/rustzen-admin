use crate::common::{
    error::ServiceError,
    query::{fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

use super::types::{MenuRow, UpdateMenuPayload};

/// Menu data access layer
pub struct MenuRepository;

#[derive(Debug, Clone)]
pub struct MenuListQuery {
    pub name: Option<String>,
    pub code: Option<String>,
    pub status: Option<i16>,
}

impl MenuRepository {
    fn format_query(query: &MenuListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        push_ilike(query_builder, "name", query.name.as_deref());
        push_ilike(query_builder, "code", query.code.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Queries menus based on conditions
    pub async fn list_menus(
        pool: &PgPool,
        query: MenuListQuery,
    ) -> Result<Vec<MenuRow>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, parent_id, parent_code, name, code, menu_type, status, is_system, is_manual, sort_order, created_at, updated_at FROM menus WHERE 1=1 AND deleted_at IS NULL",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("sort_order ASC, id ASC"),
            None,
            None,
        )
        .await
    }

    /// Creates a new menu
    pub async fn create(
        pool: &PgPool,
        parent_id: i64,
        name: &str,
        code: &str,
        menu_type: i16,
        sort_order: i16,
        status: i16,
    ) -> Result<i64, ServiceError> {
        let now = Utc::now().naive_utc();
        let menu_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_manual, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7, $7)
             RETURNING id",
        )
        .bind(parent_id)
        .bind(name)
        .bind(code)
        .bind(menu_type)
        .bind(sort_order)
        .bind(status)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating menu: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menu_id)
    }

    /// Updates an existing menu
    pub async fn update(
        pool: &PgPool,
        id: i64,
        request: &UpdateMenuPayload,
    ) -> Result<i64, ServiceError> {
        let menu_id = sqlx::query_scalar::<_, i64>(
                "UPDATE menus
                 SET parent_id = $2, name = $3, code = $4, menu_type = $5, sort_order = $6, status = $7, is_manual = TRUE, updated_at = $8
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id",
            )
            .bind(id)
            .bind(request.parent_id)
            .bind(&request.name)
            .bind(&request.code)
            .bind(request.menu_type)
            .bind(request.sort_order)
            .bind(request.status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating menu: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        if let Some(menu_id) = menu_id {
            Ok(menu_id)
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    /// Returns whether the menu is a system built-in menu.
    pub async fn is_system_menu(pool: &PgPool, id: i64) -> Result<Option<bool>, ServiceError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT is_system FROM menus WHERE id = $1 AND deleted_at IS NULL",
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
    pub async fn disable(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE menus SET status = 2, updated_at = $1 WHERE id = $2 AND is_system = false AND deleted_at IS NULL"
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
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, name FROM menus WHERE status = 1 AND deleted_at IS NULL",
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
