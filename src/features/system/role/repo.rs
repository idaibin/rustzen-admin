use super::{dto::RoleQueryDto, entity::RoleWithMenuEntity};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

pub struct RoleRepository;

impl RoleRepository {
    fn format_query(query: &RoleQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(role_name) = &query.role_name {
            if !role_name.trim().is_empty() {
                query_builder.push(" AND role_name ILIKE  ").push_bind(format!("%{}%", role_name));
            }
        }
        if let Some(role_code) = &query.role_code {
            if !role_code.trim().is_empty() {
                query_builder.push(" AND role_code ILIKE  ").push_bind(format!("%{}%", role_code));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        }
    }

    /// Count users matching filters
    async fn count_roles(pool: &PgPool, query: &RoleQueryDto) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM role_with_menus WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(count.0)
    }

    /// Queries roles with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: RoleQueryDto,
    ) -> Result<(Vec<RoleWithMenuEntity>, i64), ServiceError> {
        let total = Self::count_roles(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM role_with_menus WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let roles = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in user_with_roles pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

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

        let role_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO roles (name, code, description, status, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(Utc::now().naive_utc())
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

        // update role
        let id_opt = sqlx::query_scalar::<_, i64>(
            "UPDATE roles
                 SET name = $1, code = $2, description = $3, status = $4
                 WHERE id = $5 AND deleted_at IS NULL
                 RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = id_opt {
            // update role_menus
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
        let mut query_builder =
            String::from("INSERT INTO role_menus (role_id, menu_id, created_at) VALUES ");
        for (i, menu_id) in menu_ids.iter().enumerate() {
            if i > 0 {
                query_builder.push_str(", ");
            }
            query_builder.push_str(&format!("({}, {}, '{}')", role_id, menu_id, now));
        }
        sqlx::query(&query_builder).execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting role_menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn find_options(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query =
            String::from("SELECT id, name FROM roles WHERE status = 1 AND deleted_at IS NULL");

        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND name ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(" ORDER BY name ASC");

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
