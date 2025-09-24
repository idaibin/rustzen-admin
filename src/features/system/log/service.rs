use super::{dto::LogQueryDto, entity::LogEntity, repo::LogRepository, vo::LogItemVo};
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

    pub async fn get_all_log_csv(
        pool: &PgPool,
        query: LogQueryDto,
    ) -> Result<String, ServiceError> {
        let logs = LogRepository::find_all(pool, query).await?;
        Self::create_csv_chunk(logs, true).await
    }

    /// Create CSV chunk for a batch of logs
    async fn create_csv_chunk(
        logs: Vec<LogEntity>,
        include_header: bool,
    ) -> Result<String, ServiceError> {
        let mut csv_content = String::new();

        // Add CSV header if this is the first batch
        if include_header {
            csv_content
                .push_str("ID,user_id,username,action,description,status,duration_ms,ip_address,user_agent,created_at\n");
        }

        // Add data rows
        for log in logs {
            let row = format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                log.id,
                log.user_id,
                Self::escape_csv_field(&log.username),
                Self::escape_csv_field(&log.action),
                Self::escape_csv_field(log.description.as_deref().unwrap_or("")),
                Self::escape_csv_field(&log.status),
                log.duration_ms,
                Self::escape_csv_field(&log.ip_address.to_string()),
                Self::escape_csv_field(&log.user_agent),
                log.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            csv_content.push_str(&row);
        }

        Ok(csv_content)
    }

    /// Escape CSV field to handle commas, quotes, and newlines
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            // Escape quotes by doubling them and wrap in quotes
            let escaped = field.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            field.to_string()
        }
    }
}
