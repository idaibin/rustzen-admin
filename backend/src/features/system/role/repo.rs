// Database operations related to the `sys_role` table go here.

use super::model::RoleEntity;
use chrono::Utc;
use sqlx::PgPool;

/// Role data access layer
pub struct RoleRepository;

impl RoleRepository {
    /// Retrieves a role by its ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<RoleEntity>, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, status,
             created_at, updated_at, deleted_at
             FROM roles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// Retrieves a role by role name
    pub async fn find_by_role_name(
        pool: &PgPool,
        role_name: &str,
    ) -> Result<Option<RoleEntity>, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, status,
             created_at, updated_at, deleted_at
             FROM roles WHERE role_name = $1 AND deleted_at IS NULL",
        )
        .bind(role_name)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// Queries roles with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        role_name_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<RoleEntity>, sqlx::Error> {
        let roles = if role_name_filter.is_none() && status_filter.is_none() {
            // No filtering conditions
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, status,
                 created_at, updated_at, deleted_at
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            // With filtering conditions, implement as needed
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, status,
                 created_at, updated_at, deleted_at
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(roles)
    }

    /// Gets the total count of roles
    pub async fn count_roles(
        pool: &PgPool,
        _role_name_filter: Option<&str>,
        _status_filter: Option<i16>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;
        Ok(count.0)
    }

    /// Creates a new role
    pub async fn create(
        pool: &PgPool,
        role_name: &str,
        status: i16,
    ) -> Result<RoleEntity, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "INSERT INTO roles (role_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4)
             RETURNING id, role_name, status,
             created_at, updated_at, deleted_at",
        )
        .bind(role_name)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await?;

        Ok(role)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &PgPool,
        id: i64,
        role_name: Option<&str>,
        status: Option<i16>,
    ) -> Result<Option<RoleEntity>, sqlx::Error> {
        // Simplified implementation: first query existing role
        let existing = Self::find_by_id(pool, id).await?;
        if let Some(existing_role) = existing {
            let updated_role_name = role_name.unwrap_or(&existing_role.role_name);
            let updated_status = status.unwrap_or(existing_role.status);

            let role = sqlx::query_as::<_, RoleEntity>(
                "UPDATE roles
                 SET role_name = $2, status = $3, updated_at = $4
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, role_name, status,
                 created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(updated_role_name)
            .bind(updated_status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await?;

            Ok(role)
        } else {
            Ok(None)
        }
    }

    /// Soft deletes a role
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("UPDATE roles SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieves the menu ID list for a role
    pub async fn get_role_menu_ids(pool: &PgPool, role_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let menu_ids: Vec<(i64,)> =
            sqlx::query_as("SELECT menu_id FROM role_menus WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(pool)
                .await?;

        Ok(menu_ids.into_iter().map(|(id,)| id).collect())
    }

    /// Sets role menus
    pub async fn set_role_menus(
        pool: &PgPool,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        // Delete existing menu associations
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut *tx)
            .await?;

        // Add new menu associations
        for menu_id in menu_ids {
            sqlx::query(
                "INSERT INTO role_menus (role_id, menu_id)
                 VALUES ($1, $2)",
            )
            .bind(role_id)
            .bind(menu_id)
            .execute(&mut *tx)
            .await?;
        }

        // Commit transaction
        tx.commit().await?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn find_options(
        pool: &PgPool,
        status: Option<&str>,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let mut query = String::from("SELECT id, role_name FROM roles WHERE deleted_at IS NULL");

        // Process status
        if let Some(status) = status {
            if status == "enabled" {
                query.push_str(" AND status = 1");
            } else if status == "disabled" {
                query.push_str(" AND status = 0"); // Assuming 0 is disabled
            }
        }
        // status == "all" does not add status condition

        // Process search query
        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND role_name ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(" ORDER BY role_name ASC");

        // Process limit
        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let results: Vec<(i64, String)> = sqlx::query_as(&query).fetch_all(pool).await?;
        Ok(results)
    }
}
