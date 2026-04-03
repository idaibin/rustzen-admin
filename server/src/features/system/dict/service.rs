use super::{
    repo::{DictListQuery, DictRepository},
    types::{CreateDictRequest, DictItemResp, DictQuery, UpdateDictPayload},
};
use crate::common::{
    api::OptionItem,
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
    query::parse_optional_i16_filter,
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

        let DictQuery { current, page_size, dict_type, label, value, status } = query;
        let pagination = Pagination::from_query(PaginationQuery { current, page_size });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let status = parse_optional_i16_filter(status.as_deref(), "dict status", None)?;
        let repo_query = DictListQuery { dict_type, label, value, status };

        let (dicts, total) = DictRepository::list_dicts(pool, offset, limit, repo_query).await?;

        Ok((dicts, total))
    }

    /// Creates a new dictionary item with validation
    pub async fn create_dict(
        pool: &PgPool,
        request: CreateDictRequest,
    ) -> Result<i64, ServiceError> {
        tracing::info!(
            "Creating dictionary item: type={}, key={}",
            request.dict_type,
            request.label
        );
        DictRepository::create(
            pool,
            &request.dict_type,
            &request.label,
            &request.value,
            request.status,
            request.description.as_deref(),
            request.sort_order,
        )
        .await
    }

    /// Updates an existing dictionary item with validation
    pub async fn update_dict(
        pool: &PgPool,
        id: i64,
        request: UpdateDictPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Updating dictionary item: {}", id);
        DictRepository::update(pool, id, &request).await
    }

    /// Deletes a dictionary item by ID
    pub async fn delete_dict(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Deleting dictionary item: {}", id);
        if DictRepository::soft_delete(pool, id).await? {
            Ok(())
        } else {
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
        Ok(DictRepository::list_dict_options(
            pool,
            dict_type.as_deref(),
            search_query.as_deref(),
            limit,
        )
        .await?
        .into_iter()
        .map(|(label, value)| OptionItem { label, value })
        .collect())
    }

    /// Retrieves dictionary items by type
    pub async fn get_dict_by_type(
        pool: &PgPool,
        dict_type: &str,
    ) -> Result<Vec<OptionItem<String>>, ServiceError> {
        DictRepository::list_dicts_by_type(pool, dict_type).await
    }

    /// Updates the status of a dictionary item
    pub async fn update_dict_status(
        pool: &PgPool,
        id: i64,
        status: i16,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating dictionary item {} status to: {}", id, status);

        if ![1, 2].contains(&status) {
            return Err(ServiceError::InvalidOperation(
                "Status must be 1 (active) or 2 (inactive)".to_string(),
            ));
        }

        if DictRepository::update_status(pool, id, status).await? {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Dictionary item".to_string()))
        }
    }
}
