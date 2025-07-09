// Business logic for dictionary items.

use super::dto::{CreateDictDto, DictQueryDto, UpdateDictDto};
use super::repo::DictRepository;
use super::vo::DictDetailVo;
use crate::common::api::OptionItem;
use crate::common::error::ServiceError;
use sqlx::PgPool;

/// A service for dictionary-related operations.
pub struct DictService;

impl DictService {
    /// Retrieves a list of dictionary items with optional filtering
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `params` - Query parameters for filtering
    ///
    /// # Returns
    /// * `Result<(Vec<DictDetailVo>, i64), ServiceError>` - List of dictionary items and total count
    pub async fn get_dict_list(
        pool: &PgPool,
        params: Option<DictQueryDto>,
    ) -> Result<(Vec<DictDetailVo>, i64), ServiceError> {
        tracing::info!("Starting to retrieve dictionary list with params: {:?}", params);

        let dict_type = params.as_ref().and_then(|p| p.dict_type.as_deref());

        // Get filtered items or all items
        let dicts = if let Some(dtype) = dict_type {
            DictRepository::find_by_type(pool, dtype).await?
        } else {
            DictRepository::find_all(pool).await?
        };

        // Apply search filter if provided
        let filtered_dicts = if let Some(params) = params {
            if let Some(search) = params.q {
                dicts
                    .into_iter()
                    .filter(|dict| {
                        dict.label.to_lowercase().contains(&search.to_lowercase())
                            || dict.value.to_lowercase().contains(&search.to_lowercase())
                    })
                    .collect()
            } else {
                dicts
            }
        } else {
            dicts
        };

        let total = filtered_dicts.len() as i64;
        let dict_vos: Vec<DictDetailVo> =
            filtered_dicts.into_iter().map(DictDetailVo::from).collect();

        tracing::info!("Successfully retrieved {} dictionary items", dict_vos.len());
        Ok((dict_vos, total))
    }

    /// Retrieves a single dictionary item by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Dictionary item ID
    ///
    /// # Returns
    /// * `Result<DictDetailVo, ServiceError>` - Dictionary item or service error
    pub async fn get_dict_by_id(pool: &PgPool, id: i64) -> Result<DictDetailVo, ServiceError> {
        tracing::info!("Retrieving dictionary item by ID: {}", id);

        let dict = DictRepository::find_by_id(pool, id).await.map_err(|e| {
            tracing::error!("Failed to retrieve dictionary item {}: {}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        match dict {
            Some(dict) => {
                tracing::info!("Successfully retrieved dictionary item: {}", id);
                Ok(DictDetailVo::from(dict))
            }
            None => {
                tracing::warn!("Dictionary item not found: {}", id);
                Err(ServiceError::NotFound("Dictionary item".to_string()))
            }
        }
    }

    /// Creates a new dictionary item with validation
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Create dictionary request data
    ///
    /// # Returns
    /// * `Result<DictDetailVo, ServiceError>` - Created dictionary item or service error
    pub async fn create_dict(
        pool: &PgPool,
        request: CreateDictDto,
    ) -> Result<DictDetailVo, ServiceError> {
        tracing::info!(
            "Creating dictionary item: type={}, key={}",
            request.dict_type,
            request.label
        );

        // Check for duplicate type+key combination
        if let Ok(Some(_)) =
            DictRepository::find_by_type_and_key(pool, &request.dict_type, &request.label).await
        {
            tracing::warn!(
                "Dictionary item already exists: type={}, key={}",
                request.dict_type,
                request.label
            );
            return Err(ServiceError::InvalidOperation(
                "Dictionary item with this type and key already exists".to_string(),
            ));
        }

        // If this is marked as default, we need to unset other defaults in the same type
        if request.is_default.unwrap_or(false) {
            if let Err(e) = Self::unset_other_defaults(pool, &request.dict_type).await {
                tracing::error!(
                    "Failed to unset other defaults for type {}: {:?}",
                    request.dict_type,
                    e
                );
                return Err(ServiceError::DatabaseQueryFailed);
            }
        }

        let dict = DictRepository::create(
            pool,
            &request.dict_type,
            &request.label,
            &request.value,
            request.is_default.unwrap_or(false),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create dictionary item: {}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let dict_vo = DictDetailVo::from(dict);
        tracing::info!("Successfully created dictionary item: {}", dict_vo.id);
        Ok(dict_vo)
    }

    /// Updates an existing dictionary item with validation
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Dictionary item ID to update
    /// * `request` - Update dictionary request data
    ///
    /// # Returns
    /// * `Result<DictDetailVo, ServiceError>` - Updated dictionary item or service error
    pub async fn update_dict(
        pool: &PgPool,
        id: i64,
        request: UpdateDictDto,
    ) -> Result<DictDetailVo, ServiceError> {
        tracing::info!("Updating dictionary item: {}", id);

        // Check if item exists
        let existing = DictRepository::find_by_id(pool, id).await?;

        let existing_dict =
            existing.ok_or(ServiceError::NotFound("Dictionary item".to_string()))?;

        // Check for duplicate type+key combination if either is being changed
        let new_type = request.dict_type.as_deref().unwrap_or(&existing_dict.dict_type);
        let new_key = request.label.as_deref().unwrap_or(&existing_dict.label);

        if request.dict_type.is_some() || request.label.is_some() {
            if let Ok(Some(existing_with_key)) =
                DictRepository::find_by_type_and_key(pool, new_type, new_key).await
            {
                if existing_with_key.id != id {
                    tracing::warn!(
                        "Dictionary item already exists with type={}, key={}",
                        new_type,
                        new_key
                    );
                    return Err(ServiceError::InvalidOperation(
                        "Dictionary item with this type and key already exists".to_string(),
                    ));
                }
            }
        }

        // If setting as default, unset other defaults in the same type
        if request.is_default == Some(true) {
            let target_type = request.dict_type.as_deref().unwrap_or(&existing_dict.dict_type);
            if let Err(e) = Self::unset_other_defaults_except(pool, target_type, id).await {
                tracing::error!("Failed to unset other defaults for type {}: {:?}", target_type, e);
                return Err(ServiceError::DatabaseQueryFailed);
            }
        }

        let updated_dict = DictRepository::update(
            pool,
            id,
            request.dict_type.as_deref(),
            request.label.as_deref(),
            request.value.as_deref(),
            request.is_default,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to update dictionary item {}: {}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        match updated_dict {
            Some(dict) => {
                let dict_vo = DictDetailVo::from(dict);
                tracing::info!("Successfully updated dictionary item: {}", id);
                Ok(dict_vo)
            }
            None => {
                tracing::warn!("Dictionary item not found after update: {}", id);
                Err(ServiceError::NotFound("Dictionary item".to_string()))
            }
        }
    }

    /// Deletes a dictionary item by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Dictionary item ID to delete
    ///
    /// # Returns
    /// * `Result<(), ServiceError>` - Success or service error
    pub async fn delete_dict(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Deleting dictionary item: {}", id);

        // Check if item exists
        let existing = DictRepository::find_by_id(pool, id).await?;

        if existing.is_none() {
            tracing::warn!("Dictionary item not found for deletion: {}", id);
            return Err(ServiceError::NotFound("Dictionary item".to_string()));
        }

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
    ///
    /// Returns simplified dictionary data optimized for frontend dropdown components.
    /// Supports filtering by type, search term, and result limiting.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `dict_type` - Optional dictionary type filter
    /// * `search_query` - Optional search term for filtering labels
    /// * `limit` - Maximum number of results to return (default: 50)
    ///
    /// # Returns
    /// * `Result<Vec<OptionItem<String>>, ServiceError>` - List of option items or service error
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

        let options = DictRepository::find_options(
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

    /// Retrieves all dictionary types
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Vec<String>, ServiceError>` - List of dictionary types or service error
    pub async fn get_dict_types(pool: &PgPool) -> Result<Vec<String>, ServiceError> {
        tracing::info!("Retrieving all dictionary types");

        let types = DictRepository::find_all_types(pool).await?;

        tracing::info!("Successfully retrieved {} dictionary types", types.len());
        Ok(types)
    }

    /// Retrieves dictionary items by type
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `dict_type` - Dictionary type to filter by
    ///
    /// # Returns
    /// * `Result<Vec<DictDetailVo>, ServiceError>` - List of dictionary items or service error
    pub async fn get_dict_by_type(
        pool: &PgPool,
        dict_type: &str,
    ) -> Result<Vec<DictDetailVo>, ServiceError> {
        tracing::info!("Retrieving dictionary items by type: {}", dict_type);

        let dicts = DictRepository::find_by_type(pool, dict_type).await?;

        let dict_vos: Vec<DictDetailVo> = dicts.into_iter().map(DictDetailVo::from).collect();
        tracing::info!(
            "Successfully retrieved {} dictionary items for type {}",
            dict_vos.len(),
            dict_type
        );
        Ok(dict_vos)
    }

    /// Updates the status of a dictionary item
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Dictionary item ID
    /// * `status` - New status (1=active, 2=inactive)
    ///
    /// # Returns
    /// * `Result<(), ServiceError>` - Success or service error
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

    /// Helper method to unset default flag for other items in the same type
    async fn unset_other_defaults(pool: &PgPool, dict_type: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE dicts
             SET sort_order = 1, updated_at = CURRENT_TIMESTAMP
             WHERE type = $1 AND sort_order = 0 AND deleted_at IS NULL",
        )
        .bind(dict_type)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Helper method to unset default flag for other items in the same type except the specified ID
    async fn unset_other_defaults_except(
        pool: &PgPool,
        dict_type: &str,
        except_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE dicts
             SET sort_order = 1, updated_at = CURRENT_TIMESTAMP
             WHERE type = $1 AND sort_order = 0 AND id != $2 AND deleted_at IS NULL",
        )
        .bind(dict_type)
        .bind(except_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
