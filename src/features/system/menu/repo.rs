use super::{dto::MenuQueryDto, entity::MenuEntity};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

/// Menu data access layer
pub struct MenuRepository;

impl MenuRepository {
    fn format_query(query: &MenuQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(name) = &query.name {
            if !name.trim().is_empty() {
                query_builder.push(" AND name ILIKE  ").push_bind(format!("%{}%", name));
            }
        }
        if let Some(code) = &query.code {
            if !code.trim().is_empty() {
                query_builder.push(" AND code ILIKE  ").push_bind(format!("%{}%", code));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        } else {
            query_builder.push(" AND status = 1");
        }
    }

    /// Queries menus based on conditions
    pub async fn find_all(
        pool: &PgPool,
        query: MenuQueryDto,
    ) -> Result<Vec<MenuEntity>, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, parent_id, name, code, menu_type, status, is_system, sort_order, created_at, updated_at FROM menus WHERE 1=1",
        );

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" AND deleted_at IS NULL");
        query_builder.push(" ORDER BY sort_order ASC, id ASC");

        let menus = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menus with conditions: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
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
        let menu_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id",
        )
        .bind(parent_id)
        .bind(name)
        .bind(code)
        .bind(menu_type)
        .bind(sort_order)
        .bind(status)
        .bind(Utc::now().naive_utc())
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
        parent_id: i64,
        name: &str,
        code: &str,
        menu_type: i16,
        sort_order: i16,
        status: i16,
    ) -> Result<i64, ServiceError> {
        let menu_id = sqlx::query_scalar::<_, i64>(
                "UPDATE menus
                 SET parent_id = $2, name = $3, code = $4, menu_type = $5, sort_order = $6, status = $7, updated_at = $8
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, parent_id, name, code, menu_type, sort_order, status,
                 created_at, updated_at",
            )
            .bind(id)
            .bind(parent_id)
            .bind(name)
            .bind(code)
            .bind(menu_type)
            .bind(sort_order)
            .bind(status)
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

    /// Soft deletes a menu
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE menus SET deleted_at = $1, updated_at = $1 WHERE (id = $2 OR parent_id = $2) AND is_system = false AND deleted_at IS NULL"
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

    /// Retrieves menu list for Options API
    pub async fn find_options(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query =
            String::from("SELECT id, name FROM menus WHERE status = 1 AND deleted_at IS NULL");

        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND name ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(" ORDER BY sort_order ASC, name ASC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let menus = sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menu options: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }
}
