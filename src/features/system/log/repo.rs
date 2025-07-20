use super::model::{LogEntity, LogQueryDto};
use crate::common::error::ServiceError;
use sqlx::{PgPool, QueryBuilder};

/// Log data access layer
pub struct LogRepository;

impl LogRepository {
    fn format_query(query: &LogQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(username) = &query.username {
            if !username.trim().is_empty() {
                query_builder.push(" AND username ILIKE ").push_bind(format!("%{}%", username));
            }
        }
        if let Some(action) = &query.action {
            if !action.trim().is_empty() {
                query_builder.push(" AND action ILIKE ").push_bind(format!("%{}%", action));
            }
        }
        if let Some(description) = &query.description {
            if !description.trim().is_empty() {
                query_builder
                    .push(" AND description ILIKE ")
                    .push_bind(format!("%{}%", description));
            }
        }
        if let Some(ip_address) = &query.ip_address {
            if !ip_address.trim().is_empty() {
                query_builder
                    .push(" AND ip_address::text ILIKE ")
                    .push_bind(format!("%{}%", ip_address));
            }
        }
    }

    /// Count logs matching filters
    async fn count_logs(pool: &PgPool, query: &LogQueryDto) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM operation_logs WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        tracing::info!("user count: {:?}", count);

        Ok(count.0)
    }

    /// Find logs with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: LogQueryDto,
    ) -> Result<(Vec<LogEntity>, i64), ServiceError> {
        tracing::debug!("Finding users with pagination and filters: {:?}", query);
        let total = Self::count_logs(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM operation_logs WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let logs = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in operation_logs pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok((logs, total))
    }

    /// Creates a new log entry with full details (for business operations)
    pub async fn create_with_details(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        action: Option<&str>,
        description: Option<&str>,
        data: Option<serde_json::Value>,
        status: Option<&str>,
        duration_ms: Option<i32>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating detailed log entry with action: {:?}", action);

        let log_id = sqlx::query_scalar::<_, i64>(
            "SELECT log_operation($1, $2, $3, $4, $5, $6::inet, $7, $8, $9)",
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(description)
        .bind(data) // Option<serde_json::Value> will map to JSONB or NULL
        .bind(ip_address)
        .bind(user_agent)
        .bind(status)
        .bind(duration_ms)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating detailed log: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(log_id)
    }
}
