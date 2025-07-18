// Business logic for system logs.

use super::{
    model::{LogListVo, LogQueryDto},
    repo::LogRepository,
};
use crate::common::error::ServiceError;
use sqlx::PgPool;

/// A service for log-related operations
pub struct LogService;

impl LogService {
    /// Retrieves a paginated list of system logs
    pub async fn get_log_list(
        pool: &PgPool,
        query: LogQueryDto,
    ) -> Result<(Vec<LogListVo>, i64), ServiceError> {
        let current = query.current.unwrap_or(1);
        let limit = query.page_size.unwrap_or(10);
        let offset = (current - 1) * limit;

        // Get total count
        let total = LogRepository::count_logs(pool, query.keyword.as_deref()).await?;

        if total == 0 {
            return Ok((Vec::new(), total));
        }

        // Get log data
        let log_entities =
            LogRepository::find_all(pool, limit, offset, query.keyword.as_deref()).await?;

        // Convert to response format
        let log_responses: Vec<LogListVo> = log_entities.into_iter().map(LogListVo::from).collect();

        Ok((log_responses, total))
    }

    /// Logs an HTTP request (for middleware use only)
    /// This should be called by the logging middleware, not by business logic.
    pub async fn log_http_request(
        pool: &PgPool,
        method: &str,
        uri: &str,
        user_id: Option<i64>,
        username: Option<&str>,
        ip_address: &str,
        user_agent: &str,
        status_code: u16,
        duration_ms: i32,
    ) -> Result<(), ServiceError> {
        let action = format!("HTTP_{}", method);
        let status = if status_code < 400 { "SUCCESS" } else { "ERROR" };
        let description = format!("{} {} - {}", method, uri, status_code);

        // Only log write operations or errors
        let is_write = matches!(method, "POST" | "PUT" | "DELETE" | "PATCH");
        let is_error = status_code >= 400;

        if !is_write && !is_error {
            return Ok(()); // Skip logging for non-write/read-successful requests
        }

        tracing::info!("Logging HTTP request: {} {} - {}", method, uri, status_code);

        // Use the detailed method for HTTP requests
        let _ = LogRepository::create_with_details(
            pool,
            user_id.unwrap_or(0), // Use 0 for anonymous users
            username.unwrap_or("anonymous"),
            &action,
            &description,
            Some(ip_address),
            Some(user_agent),
            None,         // request_id
            Some("HTTP"), // resource_type
            None,         // resource_id
            status,
            Some(duration_ms),
        )
        .await?;

        Ok(())
    }

    /// Logs a business operation (for explicit CRUD, not for HTTP middleware)
    pub async fn log_business_operation(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        operation: &str, // CREATE, UPDATE, DELETE, etc.
        resource_type: &str,
        resource_id: Option<i64>,
        ip_address: &str,
        user_agent: &str,
        success: bool,
        details: Option<&str>,
        duration_ms: Option<i32>,
    ) -> Result<(), ServiceError> {
        let action = format!("{}_{}", resource_type.to_uppercase(), operation);
        let status = if success { "SUCCESS" } else { "FAILED" };

        let description: String = if let Some(details) = details {
            details.to_string()
        } else {
            format!(
                "User {} {} {} {}",
                username,
                operation.to_lowercase(),
                resource_type,
                resource_id.map(|id| format!("(ID: {})", id)).unwrap_or_default()
            )
        };

        let _ = LogRepository::create_with_details(
            pool,
            user_id,
            username,
            &action,
            description.as_str(),
            Some(ip_address),
            Some(user_agent),
            None, // request_id
            Some(resource_type),
            resource_id,
            status,
            duration_ms,
        )
        .await?;

        Ok(())
    }
}
