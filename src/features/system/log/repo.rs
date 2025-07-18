use super::model::LogEntity;
use crate::common::error::ServiceError;
use sqlx::PgPool;

/// Log data access layer
pub struct LogRepository;

impl LogRepository {
    /// Retrieves all system logs with optional filtering
    pub async fn find_all(
        pool: &PgPool,
        limit: i64,
        offset: i64,
        keyword: Option<&str>,
    ) -> Result<Vec<LogEntity>, ServiceError> {
        tracing::debug!(
            "Querying logs with search: {:?}, limit: {}, offset: {}",
            keyword,
            limit,
            offset
        );

        let logs = sqlx::query_as::<_, LogEntity>(
            "SELECT id, user_id, username, action, description, ip_address,
                    user_agent, request_id, resource_type, resource_id,
                    status, duration_ms, created_at
             FROM operation_logs
             WHERE ($1::text IS NULL OR
                    description ILIKE $1 OR
                    action ILIKE $1 OR
                    username ILIKE $1)
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(keyword.map(|s| format!("%{}%", s)))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding logs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::debug!("Retrieved {} log entries", logs.len());
        Ok(logs)
    }

    /// Counts total number of logs with optional filtering
    pub async fn count_logs(
        pool: &PgPool,
        search_query: Option<&str>,
    ) -> Result<i64, ServiceError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM operation_logs
             WHERE ($1::text IS NULL OR
                    description ILIKE $1 OR
                    action ILIKE $1 OR
                    username ILIKE $1)",
        )
        .bind(search_query.map(|s| format!("%{}%", s)))
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error counting logs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(count)
    }

    /// Creates a new log entry with full details (for business operations)
    pub async fn create_with_details(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        action: &str,
        description: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_id: Option<&str>,
        resource_type: Option<&str>,
        resource_id: Option<i64>,
        status: &str,
        duration_ms: Option<i32>,
    ) -> Result<LogEntity, ServiceError> {
        tracing::debug!("Creating detailed log entry with action: {}", action);

        let log_id = sqlx::query_scalar::<_, i64>(
            "SELECT log_operation($1, $2, $3, $4, $5::inet, $6, $7, $8, $9, $10, $11)",
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(description)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(status)
        .bind(duration_ms)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating detailed log: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Fetch the created log entry
        let log = sqlx::query_as::<_, LogEntity>(
            "SELECT id, user_id, username, action, description, ip_address,
                    user_agent, request_id, resource_type, resource_id,
                    status, duration_ms, created_at
             FROM operation_logs
             WHERE id = $1",
        )
        .bind(log_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching created detailed log: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(log)
    }
}
