// Business logic for system logs.

use super::{
    model::{LogQueryParams, LogResponse},
    repo::LogRepository,
};
use crate::common::error::ServiceError;
use sqlx::PgPool;

/// A service for log-related operations
///
/// Provides business logic for system log management including
/// querying, pagination, filtering, and log creation.
pub struct LogService;

impl LogService {
    /// Retrieves a paginated list of system logs
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `params` - Log query parameters
    ///
    /// # Returns
    /// * `Result<Json<ApiResponse<Vec<LogResponse>>>, ServiceError>` - Paginated log list or service error
    pub async fn get_log_list(
        pool: &PgPool,
        params: LogQueryParams,
    ) -> Result<(Vec<LogResponse>, i64), ServiceError> {
        let current = params.current.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(10);
        let offset = (current - 1) * page_size;

        // Get total count
        let total =
            LogRepository::count_logs(pool, params.search.as_deref()).await.map_err(|e| {
                tracing::error!("Failed to count logs: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        if total == 0 {
            return Ok((Vec::new(), total));
        }

        // Get log data
        let log_entities =
            LogRepository::find_all(pool, params.search.as_deref(), page_size, offset)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to retrieve logs: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;

        // Convert to response format
        let log_responses: Vec<LogResponse> =
            log_entities.into_iter().map(LogResponse::from).collect();

        Ok((log_responses, total))
    }

    /// Retrieves a specific log entry by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Log entry ID
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Log entry or service error
    pub async fn get_log_by_id(pool: &PgPool, id: i32) -> Result<LogResponse, ServiceError> {
        tracing::info!("Retrieving log entry with id: {}", id);

        match LogRepository::find_by_id(pool, id).await {
            Ok(Some(log)) => {
                tracing::info!("Successfully retrieved log entry with id: {}", id);
                Ok(LogResponse::from(log))
            }
            Ok(None) => {
                tracing::warn!("Log entry with id {} not found", id);
                Err(ServiceError::NotFound(format!("Log with id {} not found", id)))
            }
            Err(e) => {
                tracing::error!("Failed to retrieve log entry with id {}: {}", id, e);
                Err(ServiceError::DatabaseQueryFailed)
            }
        }
    }

    /// Creates a new system log entry
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `level` - Log level (INFO, WARN, ERROR, DEBUG)
    /// * `message` - Log message content
    /// * `user_id` - Optional user ID associated with the log
    /// * `ip_address` - Optional IP address
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Created log entry or service error
    pub async fn create_log(
        pool: &PgPool,
        level: String,
        message: String,
        user_id: Option<i32>,
        ip_address: Option<String>,
    ) -> Result<LogResponse, ServiceError> {
        // Validate log level
        if !["DEBUG", "INFO", "WARN", "ERROR"].contains(&level.as_str()) {
            tracing::warn!("Invalid log level provided: {}", level);
            return Err(ServiceError::InvalidOperation("Invalid log level".to_string()));
        }

        tracing::info!("Creating new log entry with level: {}", level);

        match LogRepository::create(pool, &level, &message, user_id, ip_address.as_deref()).await {
            Ok(log) => {
                tracing::info!("Successfully created log entry with id: {}", log.id);
                Ok(LogResponse::from(log))
            }
            Err(e) => {
                tracing::error!("Failed to create log entry: {}", e);
                Err(ServiceError::DatabaseQueryFailed)
            }
        }
    }

    /// Creates an informational log entry
    ///
    /// Convenience method for creating INFO level logs.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `message` - Log message content
    /// * `user_id` - Optional user ID associated with the log
    /// * `ip_address` - Optional IP address
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Created log entry or service error
    pub async fn log_info(
        pool: &PgPool,
        message: String,
        user_id: Option<i32>,
        ip_address: Option<String>,
    ) -> Result<LogResponse, ServiceError> {
        Self::create_log(pool, "INFO".to_string(), message, user_id, ip_address).await
    }

    /// Creates a warning log entry
    ///
    /// Convenience method for creating WARN level logs.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `message` - Log message content
    /// * `user_id` - Optional user ID associated with the log
    /// * `ip_address` - Optional IP address
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Created log entry or service error
    pub async fn log_warn(
        pool: &PgPool,
        message: String,
        user_id: Option<i32>,
        ip_address: Option<String>,
    ) -> Result<LogResponse, ServiceError> {
        Self::create_log(pool, "WARN".to_string(), message, user_id, ip_address).await
    }

    /// Creates an error log entry
    ///
    /// Convenience method for creating ERROR level logs.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `message` - Log message content
    /// * `user_id` - Optional user ID associated with the log
    /// * `ip_address` - Optional IP address
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Created log entry or service error
    pub async fn log_error(
        pool: &PgPool,
        message: String,
        user_id: Option<i32>,
        ip_address: Option<String>,
    ) -> Result<LogResponse, ServiceError> {
        Self::create_log(pool, "ERROR".to_string(), message, user_id, ip_address).await
    }

    /// Get logs by user ID
    pub async fn get_user_logs(
        pool: &PgPool,
        user_id: i64,
        current: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<(Vec<LogResponse>, i64), ServiceError> {
        let current = current.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let offset = (current - 1) * page_size;

        let log_entities = LogRepository::find_by_user_id(pool, user_id, page_size, offset)
            .await
            .map_err(|e| {
            tracing::error!("Failed to retrieve user logs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let log_responses: Vec<LogResponse> =
            log_entities.into_iter().map(LogResponse::from).collect();
        let total = LogRepository::count_logs(pool, None).await.map_err(|e| {
            tracing::error!("Failed to count user logs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok((log_responses, total))
    }
}
