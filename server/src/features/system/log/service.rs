use super::{
    repo::{LogListQuery, LogRepository},
    types::{LogItemResp, LogQuery, LogWriteCommand},
};
use crate::common::{
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
};

use sqlx::PgPool;

/// A service for log-related operations
pub struct LogService;

impl LogService {
    /// Retrieves a paginated list of system logs
    pub async fn list_logs(
        pool: &PgPool,
        query: LogQuery,
    ) -> Result<(Vec<LogItemResp>, i64), ServiceError> {
        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let repo_query = LogListQuery {
            username: query.username,
            action: query.action,
            description: query.description,
            ip_address: query.ip_address,
        };

        let (logs, total) = LogRepository::list_logs(pool, offset, limit, repo_query).await?;

        Ok((logs, total))
    }

    /// Stores a structured log record.
    pub async fn record_operation(
        pool: &PgPool,
        command: LogWriteCommand,
    ) -> Result<i64, ServiceError> {
        LogRepository::insert_log_entry(pool, &command).await
    }

    pub async fn export_logs_csv(pool: &PgPool, query: LogQuery) -> Result<String, ServiceError> {
        let repo_query = LogListQuery {
            username: query.username,
            action: query.action,
            description: query.description,
            ip_address: query.ip_address,
        };
        let logs = LogRepository::list_logs_for_export(pool, repo_query).await?;
        Self::create_csv_chunk(logs, true).await
    }

    /// Create CSV chunk for a batch of logs
    async fn create_csv_chunk(
        logs: Vec<LogItemResp>,
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
