use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

use super::types::RoleWithMenusRow;

pub struct RoleRepository;

#[derive(Debug, Clone)]
pub struct RoleListQuery {
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub status: Option<i16>,
}

impl RoleRepository {
    fn format_query(query: &RoleListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        push_ilike(query_builder, "role_name", query.role_name.as_deref());
        push_ilike(query_builder, "role_code", query.role_code.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Queries roles with pagination
    pub async fn list_roles(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: RoleListQuery,
    ) -> Result<(Vec<RoleWithMenusRow>, i64), ServiceError> {
        let total = count_with_filters(
            pool,
            "SELECT COUNT(*) FROM role_with_menus WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
        )
        .await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }
        let roles = fetch_with_filters(
            pool,
            "SELECT id, name, code, description, status, created_at, updated_at, is_system, menus FROM role_with_menus WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("created_at DESC"),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok((roles, total))
    }

    /// Creates a new role
    pub async fn create(
        pool: &PgPool,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let now = Utc::now().naive_utc();

        let role_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO roles (name, code, description, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $5)
             RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_role_menus(&mut tx, role_id, menu_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role_id)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &PgPool,
        id: i64,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let id_opt = sqlx::query_scalar::<_, i64>(
            "UPDATE roles
                 SET name = $1, code = $2, description = $3, status = $4, updated_at = $6
                 WHERE id = $5 AND deleted_at IS NULL
                 RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(id)
        .bind(Utc::now().naive_utc())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = id_opt {
            Self::insert_role_menus(&mut tx, id, menu_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing role update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("Role id: {}", id)))
        }
    }

    /// Soft deletes a role
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE roles SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL",
        )
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

    /// Returns whether the role is a system built-in role.
    pub async fn is_system_role(pool: &PgPool, id: i64) -> Result<Option<bool>, ServiceError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT is_system FROM roles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching role {} system flag: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// insert role_menus
    async fn insert_role_menus(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing role_menus: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        if menu_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc();
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("INSERT INTO role_menus (role_id, menu_id, created_at) ");
        query_builder.push_values(menu_ids.iter(), |mut builder, menu_id| {
            builder.push_bind(role_id).push_bind(menu_id).push_bind(now);
        });

        query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting role_menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn list_role_options(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, name FROM roles WHERE status = 1 AND deleted_at IS NULL",
            |query_builder| {
                push_ilike(query_builder, "name", search_query);
            },
            Some("name ASC"),
            limit,
            None,
        )
        .await
    }

    pub async fn get_role_user_count(pool: &PgPool, role_id: i64) -> Result<i64, ServiceError> {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_roles WHERE role_id = $1")
                .bind(role_id)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role user count: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(result)
    }
}
