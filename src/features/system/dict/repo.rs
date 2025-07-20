use super::{dto::DictQueryDto, entity::DictEntity};
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

pub struct DictRepository;

impl DictRepository {
    /// Formats the query for the dictionary items
    fn format_query(query: &DictQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(dict_type) = &query.dict_type {
            if !dict_type.trim().is_empty() {
                query_builder.push(" AND dict_type ILIKE  ").push_bind(format!("%{}%", dict_type));
            }
        }
        if let Some(label) = &query.label {
            if !label.trim().is_empty() {
                query_builder.push(" AND label ILIKE  ").push_bind(format!("%{}%", label));
            }
        }
        if let Some(value) = &query.value {
            if !value.trim().is_empty() {
                query_builder.push(" AND value ILIKE  ").push_bind(format!("%{}%", value));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        }
    }

    /// Count dicts matching filters
    async fn count_dicts(pool: &PgPool, query: &DictQueryDto) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM dicts WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(count.0)
    }

    /// Retrieves dictionary items with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: DictQueryDto,
    ) -> Result<(Vec<DictEntity>, i64), ServiceError> {
        let total = Self::count_dicts(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, dict_type, label, value, status, description, sort_order, updated_at
             FROM dicts WHERE 1=1
            ",
        );

        Self::format_query(&query, &mut query_builder);
        query_builder.push(" AND deleted_at IS NULL AND status = 1");
        query_builder.push(" ORDER BY id ASC, dict_type ASC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let dicts = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding dictionary items: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::debug!("Retrieved {} dictionary items", dicts.len());
        Ok((dicts, total))
    }

    pub async fn find_options(
        pool: &PgPool,
        dict_type: Option<&str>,
        search_query: Option<&str>,
        limit: i64,
    ) -> Result<Vec<(String, String)>, ServiceError> {
        tracing::debug!(
            "Querying dictionary options with type: {:?}, search: {:?}, limit: {}",
            dict_type,
            search_query,
            limit
        );

        let mut query = String::from(
            "SELECT key, value
             FROM dicts
             WHERE deleted_at IS NULL AND status = 1",
        );

        // Add type filter
        if let Some(dtype) = dict_type {
            query.push_str(&format!(" AND type = '{}'", dtype.replace("'", "''")));
        }

        // Add search filter
        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND key ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(&format!(" ORDER BY sort_order ASC, key ASC LIMIT {}", limit));

        let results: Vec<(String, String)> =
            sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
                tracing::error!("Database error finding dictionary options: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        tracing::debug!("Found {} dictionary options", results.len());
        Ok(results)
    }

    /// Retrieves dictionary items by type
    pub async fn find_by_type(
        pool: &PgPool,
        dict_type: &str,
    ) -> Result<Vec<DictEntity>, ServiceError> {
        tracing::debug!("Querying dictionary items with type: {}", dict_type);

        let dicts = sqlx::query_as::<_, DictEntity>(
            "SELECT id, type as dict_type, key as label, value,
                    CASE WHEN sort_order = 0 THEN true ELSE false END as is_default
             FROM dicts
             WHERE type = $1 AND deleted_at IS NULL AND status = 1
             ORDER BY sort_order ASC, key ASC",
        )
        .bind(dict_type)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error finding dictionary items by type '{}': {:?}",
                dict_type,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(dicts)
    }

    /// Creates a new dictionary item
    pub async fn create(
        pool: &PgPool,
        dict_type: &str,
        label: &str,
        value: &str,
        status: i16,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating new dictionary item with type: {}, label: {}", dict_type, label);

        let dict = sqlx::query_scalar::<_, i64>(
            "INSERT INTO dicts (dict_type, label, value, status, description, sort_order, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id",
        )
        .bind(dict_type)
        .bind(label)
        .bind(value)
        .bind(status)
        .bind(description)
        .bind(sort_order)
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error creating dictionary item type '{}', label '{}': {:?}",
                dict_type,
                label,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(dict)
    }

    /// Updates an existing dictionary item
    pub async fn update(
        pool: &PgPool,
        id: i64,
        dict_type: &str,
        label: &str,
        value: &str,
        status: i16,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating dictionary item with id: {}", id);

        let dict_id = sqlx::query_scalar::<_, i64>(
            "UPDATE dicts
             SET dict_type = $1, label = $2, value = $3, status = $4, description = $5, sort_order = $6, updated_at = $7
             WHERE id = $8 AND deleted_at IS NULL
             RETURNING id",
        )
            .bind(dict_type)
            .bind(label)
            .bind(value)
            .bind(status)
            .bind(description)
            .bind(sort_order)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
            tracing::error!("Database error updating dictionary item {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(dict_id) = dict_id {
            Ok(dict_id)
        } else {
            Err(ServiceError::NotFound("Dictionary item".to_string()))
        }
    }

    /// Soft deletes a dictionary item by ID
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        tracing::debug!("Soft deleting dictionary item with id: {}", id);

        let result = sqlx::query(
            "UPDATE dicts
             SET deleted_at = $1, updated_at = $1
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting dictionary item {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            tracing::info!("Soft deleted dictionary item with id: {}", id);
        } else {
            tracing::warn!("Dictionary item with id {} not found for deletion", id);
        }

        Ok(deleted)
    }

    /// Updates the status of a dictionary item
    pub async fn update_status(pool: &PgPool, id: i64, status: i16) -> Result<bool, ServiceError> {
        tracing::debug!("Updating dictionary item {} status to: {}", id, status);

        let result = sqlx::query(
            "UPDATE dicts
             SET status = $1, updated_at = $2
             WHERE id = $3 AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating dictionary item {} status: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let updated = result.rows_affected() > 0;
        if updated {
            tracing::info!("Updated dictionary item {} status to {}", id, status);
        } else {
            tracing::warn!("Dictionary item with id {} not found for status update", id);
        }

        Ok(updated)
    }
}
