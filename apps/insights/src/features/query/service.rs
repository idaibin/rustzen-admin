use chrono::{DateTime, Duration, Utc};
use rustzen_ipc::Pagination;
use rustzen_storage::SqlitePool;

use crate::common::{api::Page, error::AppError};

use super::{
    repo::{self, Window},
    types::{Event, EventQuery},
};

pub async fn events(pool: &SqlitePool, query: EventQuery) -> Result<Page<Event>, AppError> {
    let (from, to) = time_range(pool, query.from, query.to).await?;
    let (offset, limit) = pagination(query.current, query.page_size)?;
    let (rows, total) = repo::events(
        pool,
        &Window { from: &from, to: &to, offset, limit },
        clean_filter(query.event_name).as_deref(),
        query.event_kind.map(|kind| kind.as_str()),
        clean_filter(query.path).as_deref(),
        clean_filter(query.visitor_id).as_deref(),
        clean_filter(query.platform).as_deref(),
    )
    .await?;
    let data = rows.into_iter().map(Event::try_from).collect::<Result<Vec<_>, _>>()?;
    Ok(Page { data, total, success: true })
}

async fn time_range(
    pool: &SqlitePool,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Result<(String, String), AppError> {
    let settings = crate::features::settings::service::get(pool).await?;
    let to = to.unwrap_or_else(Utc::now);
    let from = from.unwrap_or_else(|| to - Duration::days(settings.default_query_days));
    if from > to || to - from > Duration::days(settings.max_query_days) {
        return Err(AppError::bad_request(format!(
            "query range must not exceed {} days",
            settings.max_query_days
        )));
    }
    Ok((from.to_rfc3339(), to.to_rfc3339()))
}

fn pagination(current: Option<i64>, page_size: Option<i64>) -> Result<(i64, i64), AppError> {
    let page = Pagination::parse(current, page_size)
        .map_err(|_| AppError::bad_request("invalid pagination"))?;
    Ok((page.offset(), page.page_size()))
}

fn clean_filter(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}
