use super::entity::DictEntity;
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::PgPool;

pub struct DictRepository;

impl DictRepository {
    /// Retrieves dictionary items with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        _dict_type: Option<&str>,
    ) -> Result<Vec<DictEntity>, ServiceError> {
        tracing::debug!("Querying all dictionary items from database");

        let dicts = sqlx::query_as::<_, DictEntity>(
            "SELECT id, type as dict_type, key as label, value,
                    CASE WHEN sort_order = 0 THEN true ELSE false END as is_default
             FROM dicts
             WHERE deleted_at IS NULL AND status = 1
             ORDER BY type ASC, sort_order ASC, id ASC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding all dictionary items: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::debug!("Retrieved {} dictionary items", dicts.len());
        Ok(dicts)
    }

    /// Retrieves dictionary options for dropdown selections
    pub async fn count_dicts(pool: &PgPool, dict_type: Option<&str>) -> Result<i64, ServiceError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM dicts WHERE deleted_at IS NULL AND status = 1")
                .bind(dict_type.as_deref())
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error counting dictionary items: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(count.0)
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

    /// Retrieves a dictionary item by ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<DictEntity>, ServiceError> {
        tracing::debug!("Querying dictionary item with id: {}", id);

        let dict = sqlx::query_as::<_, DictEntity>(
            "SELECT id, type as dict_type, key as label, value,
                    CASE WHEN sort_order = 0 THEN true ELSE false END as is_default
             FROM dicts
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding dictionary item by id {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(dict)
    }

    /// Retrieves a dictionary item by type and key
    pub async fn find_by_type_and_key(
        pool: &PgPool,
        dict_type: &str,
        key: &str,
    ) -> Result<Option<DictEntity>, ServiceError> {
        tracing::debug!("Querying dictionary item with type: {} and key: {}", dict_type, key);

        let dict = sqlx::query_as::<_, DictEntity>(
            "SELECT id, type as dict_type, key as label, value,
                    CASE WHEN sort_order = 0 THEN true ELSE false END as is_default
             FROM dicts
             WHERE type = $1 AND key = $2 AND deleted_at IS NULL",
        )
        .bind(dict_type)
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error finding dictionary item by type '{}' and key '{}': {:?}",
                dict_type,
                key,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(dict)
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
        key: &str,
        value: &str,
        is_default: bool,
    ) -> Result<DictEntity, ServiceError> {
        tracing::debug!("Creating new dictionary item with type: {}, key: {}", dict_type, key);

        let dict = sqlx::query_as::<_, DictEntity>(
            "INSERT INTO dicts (type, key, value, sort_order, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 1, $5, $5)
             RETURNING id, type as dict_type, key as label, value,
                       CASE WHEN sort_order = 0 THEN true ELSE false END as is_default",
        )
        .bind(dict_type)
        .bind(key)
        .bind(value)
        .bind(if is_default { 0 } else { 1 }) // 0 for default, 1+ for others
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error creating dictionary item type '{}', key '{}': {:?}",
                dict_type,
                key,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::info!("Created dictionary item with id: {}", dict.id);
        Ok(dict)
    }

    /// Updates an existing dictionary item
    pub async fn update(
        pool: &PgPool,
        id: i64,
        dict_type: Option<&str>,
        key: Option<&str>,
        value: Option<&str>,
        is_default: Option<bool>,
    ) -> Result<Option<DictEntity>, ServiceError> {
        tracing::debug!("Updating dictionary item with id: {}", id);

        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut param_count = 0;

        if dict_type.is_some() {
            param_count += 1;
            set_clauses.push(format!("type = ${}", param_count));
        }

        if key.is_some() {
            param_count += 1;
            set_clauses.push(format!("key = ${}", param_count));
        }

        if value.is_some() {
            param_count += 1;
            set_clauses.push(format!("value = ${}", param_count));
        }

        if is_default.is_some() {
            param_count += 1;
            set_clauses.push(format!("sort_order = ${}", param_count));
        }

        if set_clauses.is_empty() {
            return Self::find_by_id(pool, id).await;
        }

        param_count += 1;
        set_clauses.push(format!("updated_at = ${}", param_count));

        param_count += 1;
        let query = format!(
            "UPDATE dicts SET {} WHERE id = ${} AND deleted_at IS NULL
             RETURNING id, type as dict_type, key as label, value,
                       CASE WHEN sort_order = 0 THEN true ELSE false END as is_default",
            set_clauses.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, DictEntity>(&query);

        // Bind parameters in order
        if let Some(dt) = dict_type {
            query_builder = query_builder.bind(dt);
        }
        if let Some(k) = key {
            query_builder = query_builder.bind(k);
        }
        if let Some(v) = value {
            query_builder = query_builder.bind(v);
        }
        if let Some(def) = is_default {
            query_builder = query_builder.bind(if def { 0 } else { 1 });
        }

        let dict = query_builder
            .bind(Utc::now().naive_utc())
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
            tracing::error!("Database error updating dictionary item {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if dict.is_some() {
            tracing::info!("Updated dictionary item with id: {}", id);
        } else {
            tracing::warn!("Dictionary item with id {} not found for update", id);
        }

        Ok(dict)
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

    /// Retrieves all dictionary types
    pub async fn find_all_types(pool: &PgPool) -> Result<Vec<String>, ServiceError> {
        tracing::debug!("Querying all dictionary types");

        let types: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT type
             FROM dicts
             WHERE deleted_at IS NULL AND status = 1
             ORDER BY type ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding dictionary types: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let result: Vec<String> = types.into_iter().map(|(t,)| t).collect();
        tracing::debug!("Found {} dictionary types", result.len());
        Ok(result)
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
