use super::{
    repo::{DictListQuery, DictRepository},
    types::{CreateDictRequest, DictItemResp, DictQuery, UpdateDictPayload},
};
use crate::common::{
    api::OptionItem,
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
};

use sqlx::PgPool;

pub struct DictService;

impl DictService {
    /// Retrieves a list of dictionary items with optional filtering
    pub async fn list_dicts(
        pool: &PgPool,
        query: DictQuery,
    ) -> Result<(Vec<DictItemResp>, i64), ServiceError> {
        tracing::info!("Starting to retrieve dictionary list with query: {:?}", query);

        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let repo_query = DictListQuery {
            dict_type: query.dict_type,
            label: query.label,
            value: query.value,
            status: query.status,
        };

        // Get filtered items or all items
        let (dicts, total) = DictRepository::list_dicts(pool, offset, limit, repo_query).await?;

        tracing::info!("Successfully retrieved {} dictionary items", dicts.len());
        Ok((dicts, total))
    }

    /// Creates a new dictionary item with validation
    pub async fn create_dict(pool: &PgPool, request: CreateDictRequest) -> Result<i64, ServiceError> {
        tracing::info!(
            "Creating dictionary item: type={}, key={}",
            request.dict_type,
            request.label
        );

        let dict_id: i64 = DictRepository::create(
            pool,
            &request.dict_type,
            &request.label,
            &request.value,
            request.status,
            request.description.as_deref(),
            request.sort_order,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create dictionary item: {}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(dict_id)
    }

    /// Updates an existing dictionary item with validation
    pub async fn update_dict(
        pool: &PgPool,
        id: i64,
        request: UpdateDictPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Updating dictionary item: {}", id);

        let updated_dict = DictRepository::update(pool, id, &request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update dictionary item {}: {}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(updated_dict)
    }

    /// Deletes a dictionary item by ID
    pub async fn delete_dict(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Deleting dictionary item: {}", id);

        let success = DictRepository::soft_delete(pool, id).await?;

        if success {
            tracing::info!("Successfully deleted dictionary item: {}", id);
            Ok(())
        } else {
            tracing::warn!("Dictionary item not found during deletion: {}", id);
            Err(ServiceError::NotFound("Dictionary item".to_string()))
        }
    }

    /// Retrieves dictionary options for dropdown selections
    pub async fn get_dict_options(
        pool: &PgPool,
        dict_type: Option<String>,
        search_query: Option<String>,
        limit: Option<i64>,
    ) -> Result<Vec<OptionItem<String>>, ServiceError> {
        let limit = limit.unwrap_or(50);
        tracing::info!(
            "Retrieving dictionary options with type: {:?}, search: {:?}, limit: {}",
            dict_type,
            search_query,
            limit
        );

        let options = DictRepository::list_dict_options(
            pool,
            dict_type.as_deref(),
            search_query.as_deref(),
            limit,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve dictionary options: {}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let result: Vec<OptionItem<String>> =
            options.into_iter().map(|(label, value)| OptionItem { label, value }).collect();

        tracing::info!("Successfully retrieved {} dictionary options", result.len());
        Ok(result)
    }

    /// Retrieves dictionary items by type
    pub async fn get_dict_by_type(
        pool: &PgPool,
        dict_type: &str,
    ) -> Result<Vec<OptionItem<String>>, ServiceError> {
        tracing::info!("Retrieving dictionary items by type: {}", dict_type);

        let dicts = DictRepository::list_dicts_by_type(pool, dict_type).await?;

        tracing::info!(
            "Successfully retrieved {} dictionary items for type {}",
            dicts.len(),
            dict_type
        );
        Ok(dicts)
    }

    /// Updates the status of a dictionary item
    pub async fn update_dict_status(
        pool: &PgPool,
        id: i64,
        status: i16,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating dictionary item {} status to: {}", id, status);

        // Validate status value
        if ![1, 2].contains(&status) {
            return Err(ServiceError::InvalidOperation(
                "Status must be 1 (active) or 2 (inactive)".to_string(),
            ));
        }

        let success = DictRepository::update_status(pool, id, status).await?;

        if success {
            tracing::info!("Successfully updated dictionary item {} status to {}", id, status);
            Ok(())
        } else {
            tracing::warn!("Dictionary item not found for status update: {}", id);
            Err(ServiceError::NotFound("Dictionary item".to_string()))
        }
    }
}
