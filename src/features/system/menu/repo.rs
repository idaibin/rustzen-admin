// Database operations related to the `sys_menu` table go here.

use super::entity::MenuEntity;
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::PgPool;

/// Menu data access layer
pub struct MenuRepository;

impl MenuRepository {
    /// Retrieves a menu by its ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<MenuEntity>, ServiceError> {
        let menu = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, title, path, component, icon, sort_order, status,
             created_at, updated_at,  permission_code
             FROM menus WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding menu by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menu)
    }

    /// Retrieves all menus (for building tree structure)
    pub async fn find_all(pool: &PgPool) -> Result<Vec<MenuEntity>, ServiceError> {
        let menus = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, title, path, component, icon, sort_order, status,
             created_at, updated_at, permission_code
             FROM menus WHERE deleted_at IS NULL
             ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding all menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }

    /// Queries menus based on conditions
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        title_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<MenuEntity>, ServiceError> {
        let menus = if title_filter.is_none() && status_filter.is_none() {
            // No filtering conditions
            sqlx::query_as::<_, MenuEntity>(
                "SELECT id, parent_id, title, path, component, icon, sort_order, status,
                 created_at, updated_at,  permission_code
                 FROM menus WHERE deleted_at IS NULL AND menu_type != 3
                 ORDER BY sort_order ASC, id ASC
                 LIMIT $1 OFFSET $2",
            )
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error finding menus with conditions: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
        } else {
            // With filtering conditions, implement a simple version
            sqlx::query_as::<_, MenuEntity>(
                "SELECT id, parent_id, title, path, component, icon, sort_order, status,
                 created_at, updated_at, permission_code
                 FROM menus WHERE deleted_at IS NULL AND menu_type != 3
                 ORDER BY sort_order ASC, id ASC
                 LIMIT $1 OFFSET $2",
            )
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error finding menus with conditions: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
        };

        Ok(menus)
    }

    /// Gets the total count of menus
    pub async fn count_menus(
        pool: &PgPool,
        title_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<i64, ServiceError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM menus WHERE deleted_at IS NULL")
            .bind(title_filter)
            .bind(status_filter)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error counting menus: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(count.0)
    }

    /// Creates a new menu
    pub async fn create(
        pool: &PgPool,
        parent_id: Option<i64>,
        title: &str,
        path: Option<&str>,
        component: Option<&str>,
        icon: Option<&str>,
        sort_order: i32,
        status: i16,
    ) -> Result<MenuEntity, ServiceError> {
        let menu = sqlx::query_as::<_, MenuEntity>(
            "INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
             RETURNING id, parent_id, title, path, component, icon, sort_order, status,
             created_at, updated_at, deleted_at"
        )
        .bind(parent_id)
        .bind(title)
        .bind(path)
        .bind(component)
        .bind(icon)
        .bind(sort_order)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating menu: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menu)
    }

    /// Updates an existing menu
    pub async fn update(
        pool: &PgPool,
        id: i64,
        parent_id: Option<i64>,
        title: Option<&str>,
        path: Option<&str>,
        component: Option<&str>,
        icon: Option<&str>,
        sort_order: Option<i32>,
        status: Option<i16>,
    ) -> Result<Option<MenuEntity>, ServiceError> {
        // Simplified implementation: first query existing menu
        let existing = Self::find_by_id(pool, id).await?;
        if let Some(existing_menu) = existing {
            let updated_parent_id = parent_id.or(existing_menu.parent_id);
            let updated_title = title.unwrap_or(&existing_menu.title);
            let updated_path = path.or(existing_menu.path.as_deref());
            let updated_component = component.or(existing_menu.component.as_deref());
            let updated_icon = icon.or(existing_menu.icon.as_deref());
            let updated_sort_order = sort_order.unwrap_or(existing_menu.sort_order);
            let updated_status = status.unwrap_or(existing_menu.status);

            let menu = sqlx::query_as::<_, MenuEntity>(
                "UPDATE menus
                 SET parent_id = $2, title = $3, path = $4, component = $5, icon = $6,
                     sort_order = $7, status = $8, updated_at = $9
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, parent_id, title, path, component, icon, sort_order, status,
                 created_at, updated_at, permission_code",
            )
            .bind(id)
            .bind(updated_parent_id)
            .bind(updated_title)
            .bind(updated_path)
            .bind(updated_component)
            .bind(updated_icon)
            .bind(updated_sort_order)
            .bind(updated_status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating menu: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

            Ok(menu)
        } else {
            Ok(None)
        }
    }

    /// Soft deletes a menu
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE menus SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }

    /// Finds a single menu by its title.
    pub async fn find_by_title(
        title: &str,
        pool: &PgPool,
    ) -> Result<Option<MenuEntity>, ServiceError> {
        let menu = sqlx::query_as::<_, MenuEntity>(
            "SELECT * FROM menus WHERE title = $1 AND deleted_at IS NULL",
        )
        .bind(title)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding menu by title: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menu)
    }

    /// Retrieves menu list for Options API
    pub async fn find_options(
        pool: &PgPool,
        status: Option<&str>,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query = String::from("SELECT id, title FROM menus WHERE deleted_at IS NULL");

        // Process status
        if let Some(status) = status {
            if status == "enabled" {
                query.push_str(" AND status = 1");
            } else if status == "disabled" {
                query.push_str(" AND status = 0"); // Assuming 0 is disabled
            }
        }
        // status == "all" does not add status condition

        // Process fuzzy search
        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND title ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(" ORDER BY sort_order ASC, title ASC");

        // Process limit
        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let menus = sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menu options: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }
}
