use crate::common::{
    api::OptionItem,
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

use super::types::{DictItemResp, UpdateDictPayload};

pub struct DictRepository;

const DEFAULT_DICT_STATUS: i16 = 1;
const DEFAULT_DICT_SORT_ORDER: i32 = 1;

#[derive(Debug, Clone)]
pub struct DictListQuery {
    pub dict_type: Option<String>,
    pub label: Option<String>,
    pub value: Option<String>,
    pub status: Option<i16>,
}

impl DictRepository {
    /// Formats the query for the dictionary items
    fn format_query(query: &DictListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        push_ilike(query_builder, "dict_type", query.dict_type.as_deref());
        push_ilike(query_builder, "label", query.label.as_deref());
        push_ilike(query_builder, "value", query.value.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Retrieves dictionary items with pagination
    pub async fn list_dicts(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: DictListQuery,
    ) -> Result<(Vec<DictItemResp>, i64), ServiceError> {
        let total = count_with_filters(
            pool,
            "SELECT COUNT(*) FROM dicts WHERE 1=1 AND deleted_at IS NULL",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
        )
        .await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }
        let dicts = fetch_with_filters(
            pool,
            "SELECT id, dict_type, label, value, status, COALESCE(description, '') AS description, sort_order, updated_at FROM dicts WHERE 1=1 AND deleted_at IS NULL",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("dict_type ASC, sort_order ASC, id ASC"),
            Some(limit),
            Some(offset),
        )
        .await?;

        tracing::debug!("Retrieved {} dictionary items", dicts.len());
        Ok((dicts, total))
    }

    pub async fn list_dict_options(
        pool: &PgPool,
        dict_type: Option<&str>,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(String, String)>, ServiceError> {
        tracing::debug!(
            "Querying dictionary options with type: {:?}, search: {:?}, limit: {:?}",
            dict_type,
            search_query,
            limit
        );

        let results = fetch_with_filters(
            pool,
            "SELECT label, value FROM dicts WHERE deleted_at IS NULL AND status = 1",
            |query_builder| {
                if let Some(dtype) = dict_type {
                    let dtype = dtype.trim();
                    if !dtype.is_empty() {
                        query_builder.push(" AND dict_type = ").push_bind(dtype.to_string());
                    }
                }
                push_ilike(query_builder, "label", search_query);
            },
            Some("sort_order ASC, label ASC"),
            limit,
            None,
        )
        .await?;

        tracing::debug!("Found {} dictionary options", results.len());
        Ok(results)
    }

    /// Retrieves dictionary items by type
    pub async fn list_dicts_by_type(
        pool: &PgPool,
        dict_type: &str,
    ) -> Result<Vec<OptionItem<String>>, ServiceError> {
        tracing::debug!("Querying dictionary items with type: {}", dict_type);

        let dicts = fetch_with_filters(
            pool,
            "SELECT label, value FROM dicts WHERE deleted_at IS NULL AND status = 1",
            |query_builder| {
                query_builder.push(" AND dict_type = ").push_bind(dict_type.to_string());
            },
            Some("sort_order ASC, label ASC"),
            None,
            None,
        )
        .await?;

        Ok(dicts)
    }

    /// Creates a new dictionary item
    pub async fn create(
        pool: &PgPool,
        dict_type: &str,
        label: &str,
        value: &str,
        status: Option<i16>,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating new dictionary item with type: {}, label: {}", dict_type, label);
        let now = Utc::now().naive_utc();

        let dict = sqlx::query_scalar::<_, i64>(
            "INSERT INTO dicts (dict_type, label, value, status, description, sort_order, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
             RETURNING id",
        )
        .bind(dict_type)
        .bind(label)
        .bind(value)
        .bind(status.unwrap_or(DEFAULT_DICT_STATUS))
        .bind(description)
        .bind(sort_order.unwrap_or(DEFAULT_DICT_SORT_ORDER))
        .bind(now)
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
        request: &UpdateDictPayload,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating dictionary item with id: {}", id);

        let dict_id = sqlx::query_scalar::<_, i64>(
            "UPDATE dicts
             SET dict_type = $1, label = $2, value = $3, status = $4, description = $5, sort_order = $6, updated_at = $7
             WHERE id = $8 AND deleted_at IS NULL
             RETURNING id",
        )
        .bind(&request.dict_type)
        .bind(&request.label)
        .bind(&request.value)
        .bind(request.status.unwrap_or(DEFAULT_DICT_STATUS))
        .bind(request.description.as_deref())
        .bind(request.sort_order.unwrap_or(DEFAULT_DICT_SORT_ORDER))
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
