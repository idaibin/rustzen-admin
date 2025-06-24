// Business logic for system logs.

use super::repo::LogRepository;
use crate::common::error::ServiceError;
use serde_json::Value;
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
    /// * `search_query` - Optional search term for filtering log messages
    /// * `page` - Page number (1-based)
    /// * `page_size` - Number of records per page
    ///
    /// # Returns
    /// * `Result<(Vec<Value>, i64), ServiceError>` - Tuple of (logs, total_count) or service error
    pub async fn get_log_list(
        pool: &PgPool,
        search_query: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<(Vec<Value>, i64), ServiceError> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        tracing::info!(
            "Retrieving log list with search: {:?}, page: {}, page_size: {}",
            search_query,
            page,
            page_size
        );

        // Get total count for pagination
        let total_count = match LogRepository::count_all(pool, search_query.as_deref()).await {
            Ok(count) => count,
            Err(e) => {
                tracing::error!("Failed to count logs: {}", e);
                return Err(ServiceError::DatabaseQueryFailed);
            }
        };

        // Get log list
        match LogRepository::find_all(pool, search_query.as_deref(), page_size, offset).await {
            Ok(logs) => {
                tracing::info!(
                    "Successfully retrieved {} logs (total: {})",
                    logs.len(),
                    total_count
                );
                Ok((logs, total_count))
            }
            Err(e) => {
                tracing::error!("Failed to retrieve log list: {}", e);
                Err(ServiceError::DatabaseQueryFailed)
            }
        }
    }

    /// Retrieves a specific log entry by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Log entry ID
    ///
    /// # Returns
    /// * `Result<Value, ServiceError>` - Log entry or service error
    pub async fn get_log_by_id(pool: &PgPool, id: i32) -> Result<Value, ServiceError> {
        tracing::info!("Retrieving log entry with id: {}", id);

        match LogRepository::find_by_id(pool, id).await {
            Ok(Some(log)) => {
                tracing::info!("Successfully retrieved log entry with id: {}", id);
                Ok(log)
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
    ) -> Result<Value, ServiceError> {
        // Validate log level
        if !["DEBUG", "INFO", "WARN", "ERROR"].contains(&level.as_str()) {
            tracing::warn!("Invalid log level provided: {}", level);
            return Err(ServiceError::InvalidOperation("Invalid log level".to_string()));
        }

        tracing::info!("Creating new log entry with level: {}", level);

        match LogRepository::create(pool, &level, &message, user_id, ip_address.as_deref()).await {
            Ok(log) => {
                if let Some(id) = log.get("id") {
                    tracing::info!("Successfully created log entry with id: {}", id);
                }
                Ok(log)
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
    ) -> Result<Value, ServiceError> {
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
    ) -> Result<Value, ServiceError> {
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
    ) -> Result<Value, ServiceError> {
        Self::create_log(pool, "ERROR".to_string(), message, user_id, ip_address).await
    }
}
