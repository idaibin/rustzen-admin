// Database operations related to the `sys_role` table go here.

use super::entity::RoleEntity;
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::PgPool;

/// Role data access layer
pub struct RoleRepository;

impl RoleRepository {
    /// Retrieves a role by its ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<RoleEntity>, ServiceError> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, role_code, description, status,
             created_at, updated_at, deleted_at, is_system
             FROM roles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding role by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role)
    }

    /// Retrieves a role by role name
    pub async fn find_by_role_name(
        pool: &PgPool,
        role_name: &str,
    ) -> Result<Option<RoleEntity>, ServiceError> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, role_code, description, status,
             created_at, updated_at, deleted_at, is_system
             FROM roles WHERE role_name = $1 AND deleted_at IS NULL",
        )
        .bind(role_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding role by name {}: {:?}", role_name, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role)
    }

    /// Queries roles with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        role_name_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<RoleEntity>, ServiceError> {
        let roles = if role_name_filter.is_none() && status_filter.is_none() {
            // No filtering conditions
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, role_code, description, status,
                 created_at, updated_at, deleted_at, is_system
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error finding roles with pagination: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
        } else {
            // With filtering conditions, implement as needed
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, role_code, description, status,
                 created_at, updated_at, deleted_at, is_system
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error finding roles with pagination: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
        };

        Ok(roles)
    }

    /// Gets the total count of roles
    pub async fn count_roles(
        pool: &PgPool,
        _role_name_filter: Option<&str>,
        _status_filter: Option<i16>,
    ) -> Result<i64, ServiceError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error counting roles: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(count.0)
    }

    /// Creates a new role
    pub async fn create(
        pool: &PgPool,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
    ) -> Result<RoleEntity, ServiceError> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "INSERT INTO roles (role_name, role_code, description, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, role_name, role_code, description, status,
             created_at, updated_at, deleted_at, is_system",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &PgPool,
        id: i64,
        role_name: Option<&str>,
        role_code: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> Result<Option<RoleEntity>, ServiceError> {
        // Simplified implementation: first query existing role
        let existing = Self::find_by_id(pool, id).await?;
        if let Some(existing_role) = existing {
            let updated_role_name = role_name.unwrap_or(&existing_role.role_name);
            let updated_role_code = role_code.unwrap_or(&existing_role.role_code);
            let updated_description =
                description.unwrap_or_else(|| existing_role.description.as_deref().unwrap_or(""));
            let updated_status = status.unwrap_or(existing_role.status);

            let role = sqlx::query_as::<_, RoleEntity>(
                "UPDATE roles
                 SET role_name = $2, role_code = $3, description = $4, status = $5, updated_at = $6
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, role_name, role_code, description, status,
                 created_at, updated_at, deleted_at, is_system",
            )
            .bind(id)
            .bind(updated_role_name)
            .bind(updated_role_code)
            .bind(updated_description)
            .bind(updated_status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating role: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

            Ok(role)
        } else {
            Ok(None)
        }
    }

    /// Soft deletes a role
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result =
            sqlx::query("UPDATE roles SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error soft deleting role {}: {:?}", id, e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieves the menu ID list for a role
    pub async fn get_role_menu_ids(pool: &PgPool, role_id: i64) -> Result<Vec<i64>, ServiceError> {
        let menu_ids: Vec<(i64,)> =
            sqlx::query_as("SELECT menu_id FROM role_menus WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role menu IDs: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(menu_ids.into_iter().map(|(id,)| id).collect())
    }

    /// Sets role menus
    pub async fn set_role_menus(
        pool: &PgPool,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        // Delete existing menu associations
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing role menus: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        // Add new menu associations
        for menu_id in menu_ids {
            sqlx::query(
                "INSERT INTO role_menus (role_id, menu_id)
                 VALUES ($1, $2)",
            )
            .bind(role_id)
            .bind(menu_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error inserting new role menu: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        }

        // Commit transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing transaction for role menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn find_options(
        pool: &PgPool,
        status: Option<&str>,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
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

        let results: Vec<(i64, String)> =
            sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
                tracing::error!("Database error finding role options: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(results)
    }

    pub async fn is_system_role(pool: &PgPool, role_id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1 AND is_system = true)",
            role_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking if role is system: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(result.unwrap_or(false))
    }

    pub async fn get_role_user_count(pool: &PgPool, role_id: i64) -> Result<i64, ServiceError> {
        let result =
            sqlx::query_scalar!("SELECT COUNT(*) FROM user_roles WHERE role_id = $1", role_id,)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role user count: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(result.unwrap_or(0))
    }
}
