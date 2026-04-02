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
        let LogQuery { current, page_size, username, action, description, ip_address } = query;
        let pagination = Pagination::from_query(PaginationQuery { current, page_size });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let repo_query = LogListQuery { username, action, description, ip_address };

        LogRepository::list_logs(pool, offset, limit, repo_query).await
    }

    /// Stores a structured log record.
    pub async fn record_operation(
        pool: &PgPool,
        command: LogWriteCommand,
    ) -> Result<i64, ServiceError> {
        LogRepository::insert_log_entry(pool, &command).await
    }

    pub async fn export_logs_csv(pool: &PgPool, query: LogQuery) -> Result<String, ServiceError> {
        let LogQuery { username, action, description, ip_address, .. } = query;
        let repo_query = LogListQuery { username, action, description, ip_address };
        Self::create_csv_chunk(LogRepository::list_logs_for_export(pool, repo_query).await?, true)
    }

    /// Create CSV chunk for a batch of logs
    fn create_csv_chunk(
        logs: Vec<LogItemResp>,
        include_header: bool,
    ) -> Result<String, ServiceError> {
        let mut csv_content = String::new();

        if include_header {
            csv_content
                .push_str("ID,user_id,username,action,description,status,duration_ms,ip_address,user_agent,created_at\n");
        }

        csv_content.push_str(
            &logs
                .into_iter()
                .map(|log| {
                    format!(
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
                    )
                })
                .collect::<String>(),
        );

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
