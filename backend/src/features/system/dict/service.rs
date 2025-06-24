// Business logic for dictionary items.

use super::model::DictItem;
use super::repo::DictRepository;
use crate::common::api::OptionItem;
use crate::common::error::ServiceError;
use sqlx::PgPool;

/// A service for dictionary-related operations.
pub struct DictService;

impl DictService {
    /// Retrieves a list of dictionary items
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Vec<DictItem>, ServiceError>` - List of dictionary items or service error
    pub async fn get_dict_list(pool: &PgPool) -> Result<Vec<DictItem>, ServiceError> {
        tracing::info!("Starting to retrieve dictionary list");

        match DictRepository::find_all(pool).await {
            Ok(dicts) => {
                tracing::info!("Successfully retrieved {} dictionary items", dicts.len());
                Ok(dicts)
            }
            Err(e) => {
                tracing::error!("Failed to retrieve dictionary list: {}", e);
                Err(ServiceError::DatabaseQueryFailed)
            }
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

        match DictRepository::find_options(
            pool,
            dict_type.as_deref(),
            search_query.as_deref(),
            limit,
        )
        .await
        {
            Ok(options) => {
                let result: Vec<OptionItem<String>> =
                    options.into_iter().map(|(label, value)| OptionItem { label, value }).collect();

                tracing::info!("Successfully retrieved {} dictionary options", result.len());
                Ok(result)
            }
            Err(e) => {
                tracing::error!("Failed to retrieve dictionary options: {}", e);
                Err(ServiceError::DatabaseQueryFailed)
            }
        }
    }
}
