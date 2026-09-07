use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::types::{RoleListQuery, RoleWithMenusRow};

pub struct RoleRepository;

#[derive(Debug, PartialEq, Eq)]
pub enum SoftDeleteOutcome {
    Deleted,
    AssignmentBlocked(i64),
    NotFound,
}

impl RoleRepository {
    fn format_query(query: &RoleListQuery, query_builder: &mut QueryBuilder<Sqlite>) {
        push_ilike(query_builder, "name", query.role_name.as_deref());
        push_ilike(query_builder, "code", query.role_code.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Queries roles with pagination
    pub async fn list_roles(
        pool: &SqlitePool,
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
            "SELECT id, name, code, description, status, created_at, updated_at, is_system, menus,
                    (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = role_with_menus.id)
                        AS assigned_user_count
             FROM role_with_menus
             WHERE 1=1",
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
    pub async fn get_role_identity(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<(String, bool)>, ServiceError> {
        sqlx::query_as::<_, (String, bool)>(
            "SELECT code, is_system FROM roles WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching role {} identity: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// insert role_menus
    pub(super) async fn insert_role_menus(
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM role_menus WHERE role_id = ?")
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
        let mut query_builder: QueryBuilder<Sqlite> =
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
        pool: &SqlitePool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String, String, bool)>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, name, code, is_system FROM roles WHERE status = 1 AND deleted_at IS NULL",
            |query_builder| {
                push_ilike(query_builder, "name", search_query);
            },
            Some("name ASC"),
            limit,
            None,
        )
        .await
    }

    pub async fn get_role_user_count(pool: &SqlitePool, role_id: i64) -> Result<i64, ServiceError> {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_roles WHERE role_id = ?")
                .bind(role_id)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role user count: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(result)
    }

    pub async fn list_menu_codes_by_ids(
        pool: &SqlitePool,
        menu_ids: &[i64],
    ) -> Result<Vec<String>, ServiceError> {
        if menu_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT code FROM menus WHERE deleted_at IS NULL AND id IN (");
        let mut separated = query_builder.separated(", ");
        for menu_id in menu_ids {
            separated.push_bind(menu_id);
        }
        separated.push_unseparated(")");

        query_builder.build_query_scalar().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error listing menu codes by ids: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })
    }
}

#[cfg(test)]
#[path = "repo_tests.rs"]
mod tests;
