use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_ilike},
};

use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::types::{LogItemResp, LogListQuery, LogWriteCommand};

/// Log data access layer
pub struct LogRepository;

impl LogRepository {
    fn format_query(query: &LogListQuery, query_builder: &mut QueryBuilder<'_, Sqlite>) {
        push_ilike(query_builder, "username", query.username.as_deref());
        push_ilike(query_builder, "action", query.action.as_deref());
        push_ilike(query_builder, "description", query.description.as_deref());
        push_ilike(query_builder, "ip_address", query.ip_address.as_deref());
    }

    /// Find logs with pagination and filters
    pub async fn list_logs(
        pool: &SqlitePool,
        offset: i64,
        limit: i64,
        query: LogListQuery,
    ) -> Result<(Vec<LogItemResp>, i64), ServiceError> {
        tracing::debug!("Finding logs with pagination and filters: {:?}", query);
        let total = count_with_filters(
            pool,
            "SELECT COUNT(*) FROM operation_logs WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
        )
        .await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }
        let logs = fetch_with_filters(
            pool,
            "SELECT id, user_id, username, action, description, data, status, duration_ms, ip_address, user_agent, created_at FROM operation_logs WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("created_at DESC"),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok((logs, total))
    }

    /// Creates a new log entry with full details (for business operations)
    pub async fn insert_log_entry(
        pool: &SqlitePool,
        command: &LogWriteCommand,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating detailed log entry with action: {:?}", command.action);

        let log_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO operation_logs (
                user_id, username, action, description, data, status, duration_ms, ip_address, user_agent, created_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP
            ) RETURNING id",
        )
        .bind(command.user_id)
        .bind(command.username.as_str())
        .bind(command.action.as_str())
        .bind(command.description.as_str())
        .bind(command.data.clone()) // Option<serde_json::Value> will map to JSONB or NULL
        .bind(command.status.as_str())
        .bind(command.duration_ms)
        .bind(command.ip_address.as_str())
        .bind(command.user_agent.as_str())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating detailed log: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(log_id)
    }

    /// Ensure the current and near-future monthly partitions exist before request logging begins.
    pub async fn ensure_partitions(_pool: &SqlitePool) -> Result<(), ServiceError> {
        let _ = Utc::now();
        tracing::debug!("Skipping partition creation for SQLite backend");

        Ok(())
    }

    pub async fn list_logs_for_export(
        pool: &SqlitePool,
        query: LogListQuery,
    ) -> Result<Vec<LogItemResp>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, user_id, username, action, description, data, status, duration_ms, ip_address, user_agent, created_at FROM operation_logs WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("created_at DESC"),
            None,
            None,
        )
        .await
    }
}
