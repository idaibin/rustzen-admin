use super::{dto::LogQueryDto, repo::LogRepository, vo::LogItemVo};
use crate::common::{error::ServiceError, pagination::Pagination};

use sqlx::PgPool;

/// A service for log-related operations
pub struct LogService;

impl LogService {
    /// Retrieves a paginated list of system logs
    pub async fn get_log_list(
        pool: &PgPool,
        query: LogQueryDto,
    ) -> Result<(Vec<LogItemVo>, i64), ServiceError> {
        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);

        let (logs, total) = LogRepository::find_with_pagination(pool, offset, limit, query).await?;
        let list: Vec<LogItemVo> = logs.into_iter().map(LogItemVo::from).collect();

        Ok((list, total))
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

        // // Only log write operations or errors
        // let is_write = matches!(method, "POST" | "PUT" | "DELETE" | "PATCH");

        // if !is_write {
        //     return Ok(()); // Skip logging for non-write/read-successful requests
        // }

        tracing::info!("Logging HTTP request: {} {} - {}", method, uri, status_code);

        // Use the detailed method for HTTP requests
        let _ = LogRepository::create_with_details(
            pool,
            user_id.unwrap_or(0), // Use 0 for anonymous users
            username.unwrap_or("anonymous"),
            Some(&action),
            Some(&description),
            None,
            Some(status),
            Some(duration_ms),
            Some(ip_address),
            Some(user_agent),
        )
        .await?;

        Ok(())
    }

    /// Logs a business operation (for explicit CRUD, not for HTTP middleware)
    pub async fn log_business_operation(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        action: &str,
        description: &str,
        data: serde_json::Value,
        status: &str,
        duration_ms: i32,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<(), ServiceError> {
        let _ = LogRepository::create_with_details(
            pool,
            user_id,
            username,
            Some(action),
            Some(description),
            Some(data),
            Some(status),
            Some(duration_ms),
            Some(ip_address),
            Some(user_agent),
        )
        .await?;

        Ok(())
    }
}
