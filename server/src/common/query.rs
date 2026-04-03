use crate::common::error::ServiceError;

use sqlx::{PgPool, Postgres, QueryBuilder, postgres::PgRow};

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

/// Parse an optional `i16` filter from query text.
///
/// `default_on_empty` is used when the value is missing or empty.
/// `all` always disables the filter.
pub fn parse_optional_i16_filter(
    value: Option<&str>,
    field_name: &str,
    default_on_empty: Option<i16>,
) -> Result<Option<i16>, ServiceError> {
    match value.map(str::trim) {
        None | Some("") => Ok(default_on_empty),
        Some("all") => Ok(None),
        Some(raw) => raw.parse::<i16>().map(Some).map_err(|_| {
            ServiceError::InvalidOperation(format!("Invalid {} value: {}", field_name, raw))
        }),
    }
}

fn map_db_error(context: &str, err: sqlx::Error) -> ServiceError {
    tracing::error!("Database error {}: {:?}", context, err);
    ServiceError::DatabaseQueryFailed
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

    let count: (i64,) = query_builder
        .build_query_as()
        .fetch_one(pool)
        .await
        .map_err(|e| map_db_error("counting rows", e))?;

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

    let rows = query_builder
        .build_query_as::<T>()
        .fetch_all(pool)
        .await
        .map_err(|e| map_db_error("fetching rows", e))?;

    Ok(rows)
}
