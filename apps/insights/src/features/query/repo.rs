use rustzen_storage::SqlitePool;

use super::types::EventRow;

pub struct Window<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub offset: i64,
    pub limit: i64,
}

pub async fn events(
    pool: &SqlitePool,
    window: &Window<'_>,
    event_name: Option<&str>,
    event_kind: Option<&str>,
    path: Option<&str>,
    visitor_id: Option<&str>,
    platform: Option<&str>,
) -> Result<(Vec<EventRow>, i64), sqlx::Error> {
    let escaped_path = path.map(escape_like_pattern);
    let path = escaped_path.as_deref();
    let rows = sqlx::query_as(
        "SELECT id, event_name, visitor_id, user_id, session_id, platform, page_path,
                referrer, api_path, api_method, status_code, duration_ms, is_error,
                properties, occurred_at, received_at FROM insights_events
         WHERE occurred_at >= ? AND occurred_at <= ?
           AND (? IS NULL OR event_name = ?) AND (? IS NULL OR visitor_id = ?)
           AND (? IS NULL OR platform = ?)
           AND (? IS NULL OR CASE ?
                WHEN 'page' THEN event_name = 'page_view'
                WHEN 'api' THEN event_name = 'api_request'
                WHEN 'other' THEN event_name NOT IN ('page_view', 'api_request')
                ELSE 0 END)
           AND (? IS NULL OR page_path LIKE '%' || ? || '%' ESCAPE char(92)
                OR api_path LIKE '%' || ? || '%' ESCAPE char(92))
         ORDER BY occurred_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(window.from)
    .bind(window.to)
    .bind(event_name)
    .bind(event_name)
    .bind(visitor_id)
    .bind(visitor_id)
    .bind(platform)
    .bind(platform)
    .bind(event_kind)
    .bind(event_kind)
    .bind(path)
    .bind(path)
    .bind(path)
    .bind(window.limit)
    .bind(window.offset)
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM insights_events
         WHERE occurred_at >= ? AND occurred_at <= ?
           AND (? IS NULL OR event_name = ?) AND (? IS NULL OR visitor_id = ?)
           AND (? IS NULL OR platform = ?)
           AND (? IS NULL OR CASE ?
                WHEN 'page' THEN event_name = 'page_view'
                WHEN 'api' THEN event_name = 'api_request'
                WHEN 'other' THEN event_name NOT IN ('page_view', 'api_request')
                ELSE 0 END)
           AND (? IS NULL OR page_path LIKE '%' || ? || '%' ESCAPE char(92)
                OR api_path LIKE '%' || ? || '%' ESCAPE char(92))",
    )
    .bind(window.from)
    .bind(window.to)
    .bind(event_name)
    .bind(event_name)
    .bind(visitor_id)
    .bind(visitor_id)
    .bind(platform)
    .bind(platform)
    .bind(event_kind)
    .bind(event_kind)
    .bind(path)
    .bind(path)
    .bind(path)
    .fetch_one(pool)
    .await?;
    Ok((rows, total))
}

fn escape_like_pattern(value: &str) -> String {
    value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}
