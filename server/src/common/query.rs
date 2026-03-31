use crate::common::error::ServiceError;

use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder};

/// Apply a case-insensitive LIKE filter when the value is present and non-empty.
pub fn push_ilike(
    query_builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        let value = value.trim();
        if !value.is_empty() {
            query_builder
                .push(" AND ")
                .push(column)
                .push(" ILIKE ")
                .push_bind(format!("%{}%", value));
        }
    }
}

/// Apply an equality filter when the value is present.
pub fn push_eq<'a, T>(
    query_builder: &mut QueryBuilder<'a, Postgres>,
    column: &str,
    value: Option<T>,
) where
    T: for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + 'a,
{
    if let Some(value) = value {
        query_builder.push(" AND ").push(column).push(" = ").push_bind(value);
    }
}

/// Count rows for a filtered query.
pub async fn count_with_filters<F>(
    pool: &PgPool,
    base_sql: &'static str,
    apply_filters: F,
) -> Result<i64, ServiceError>
where
    F: for<'qb> FnOnce(&mut QueryBuilder<'qb, Postgres>),
{
    let mut query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(base_sql);
    apply_filters(&mut query_builder);

    let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
        tracing::error!("Database error counting rows: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(count.0)
}

/// Fetch rows for a filtered query with optional ordering and pagination.
pub async fn fetch_with_filters<T, F>(
    pool: &PgPool,
    base_sql: &'static str,
    apply_filters: F,
    order_by: Option<&'static str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<T>, ServiceError>
where
    F: for<'qb> FnOnce(&mut QueryBuilder<'qb, Postgres>),
    T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    let mut query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(base_sql);
    apply_filters(&mut query_builder);

    if let Some(order_by) = order_by {
        query_builder.push(" ORDER BY ").push(order_by);
    }

    if let Some(limit) = limit {
        query_builder.push(" LIMIT ").push_bind(limit);
    }

    if let Some(offset) = offset {
        query_builder.push(" OFFSET ").push_bind(offset);
    }

    let rows = query_builder.build_query_as::<T>().fetch_all(pool).await.map_err(|e| {
        tracing::error!("Database error fetching rows: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(rows)
}
