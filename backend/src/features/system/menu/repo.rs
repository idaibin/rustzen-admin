// Database operations related to the `sys_menu` table go here.

use super::model::MenuEntity;
use chrono::Utc;
use sqlx::PgPool;

/// Menu data access layer
pub struct MenuRepository;

impl MenuRepository {
    /// Retrieves a menu by its ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<MenuEntity>, sqlx::Error> {
        let menu = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, title, path, component, icon, sort_order, status,
             created_at, updated_at,  permission_code
             FROM menus WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(menu)
    }

    /// Retrieves all menus (for building tree structure)
    pub async fn find_all(pool: &PgPool) -> Result<Vec<MenuEntity>, sqlx::Error> {
        let menus = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, title, path, component, icon, sort_order, status,
             created_at, updated_at, permission_code
             FROM menus WHERE deleted_at IS NULL
             ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(menus)
    }

    /// Queries menus based on conditions
    pub async fn find_with_conditions(
        pool: &PgPool,
        title_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<MenuEntity>, sqlx::Error> {
        let menus = if title_filter.is_none() && status_filter.is_none() {
            // No filtering conditions
            sqlx::query_as::<_, MenuEntity>(
                "SELECT id, parent_id, title, path, component, icon, sort_order, status,
                 created_at, updated_at,  permission_code
                 FROM menus WHERE deleted_at IS NULL AND menu_type != 3
                 ORDER BY sort_order ASC, id ASC",
            )
            .fetch_all(pool)
            .await?
        } else {
            // With filtering conditions, implement a simple version
            sqlx::query_as::<_, MenuEntity>(
                "SELECT id, parent_id, title, path, component, icon, sort_order, status,
                 created_at, updated_at, permission_code
                 FROM menus WHERE deleted_at IS NULL AND menu_type != 3
                 ORDER BY sort_order ASC, id ASC",
            )
            .fetch_all(pool)
            .await?
        };

        Ok(menus)
    }

    /// Gets the total count of menus
    pub async fn count_menus(
        pool: &PgPool,
        _title_filter: Option<&str>,
        _status_filter: Option<i16>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM menus WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;

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
    ) -> Result<MenuEntity, sqlx::Error> {
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
        .await?;

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
    ) -> Result<Option<MenuEntity>, sqlx::Error> {
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
                 created_at, updated_at, deleted_at",
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
            .await?;

            Ok(menu)
        } else {
            Ok(None)
        }
    }

    /// Soft deletes a menu
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE menus SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieves menus by role IDs
    pub async fn find_menus_by_role(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<MenuEntity>, sqlx::Error> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        // Build IN query placeholders
        let placeholders: Vec<String> = (1..=role_ids.len()).map(|i| format!("${}", i)).collect();
        let placeholders_str = placeholders.join(", ");

        let query = format!(
            "SELECT DISTINCT m.id, m.parent_id, m.title, m.path, m.component, m.icon,
            m.sort_order, m.status, m.permission_code, m.created_at, m.updated_at
            FROM menus m
            INNER JOIN role_menus rm ON m.id = rm.menu_id
            WHERE rm.role_id IN ({}) AND m.deleted_at IS NULL AND m.status = 1
            ORDER BY m.sort_order ASC, m.id ASC",
            placeholders_str
        );

        let mut query_builder = sqlx::query_as::<_, MenuEntity>(&query);

        // Bind parameters
        for role_id in role_ids {
            query_builder = query_builder.bind(role_id);
        }

        let menus = query_builder.fetch_all(pool).await?;
        Ok(menus)
    }

    pub async fn find_permissions_by_role(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<String>, sqlx::Error> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: Vec<String> = (1..=role_ids.len()).map(|i| format!("${}", i)).collect();
        let placeholders_str = placeholders.join(", ");
        let query = format!(
            "SELECT DISTINCT m.permission_code
             FROM menus m
             INNER JOIN role_menus rm ON m.id = rm.menu_id
             WHERE rm.role_id IN ({}) AND m.deleted_at IS NULL AND m.permission_code IS NOT NULL",
            placeholders_str
        );

        let mut query_builder = sqlx::query_scalar(&query);

        for role_id in role_ids {
            query_builder = query_builder.bind(role_id);
        }

        let permissions = query_builder.fetch_all(pool).await?;
        Ok(permissions)
    }
    /// Finds a single menu by its title.
    pub async fn find_by_title(
        title: &str,
        pool: &PgPool,
    ) -> Result<Option<MenuEntity>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM menus WHERE title = $1 AND deleted_at IS NULL")
            .bind(title)
            .fetch_optional(pool)
            .await
    }

    /// Retrieves menu list for Options API
    pub async fn find_options(
        pool: &PgPool,
        status: Option<&str>,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
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

        let menus = sqlx::query_as(&query).fetch_all(pool).await?;

        Ok(menus)
    }
}
