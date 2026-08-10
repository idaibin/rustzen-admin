use chrono::{DateTime, Duration, Utc};
use rustzen_ipc::{Page, Pagination};
use rustzen_storage::SqlitePool;
use uuid::Uuid;

use crate::{common::error::AppError, features::nodes::types::NodeView};

use super::repo;
use super::types::{IncidentDetail, IncidentSummary, ListQuery};

const MAX_QUERY_RANGE: Duration = Duration::days(90);

pub(crate) async fn list(
    pool: &SqlitePool,
    query: ListQuery,
) -> Result<Page<IncidentSummary>, AppError> {
    let page = Pagination::parse(query.current, query.page_size).map_err(|_| {
        AppError::unprocessable("current must be positive and pageSize must be between 1 and 100")
    })?;
    validate_status(query.status.as_deref())?;
    validate_source_type(query.source_type.as_deref())?;
    validate_source_id(query.source_id.as_deref())?;
    let window = TimeWindow::parse(query.from.as_deref(), query.to.as_deref())?;
    let (rows, total) = repo::list(
        pool,
        repo::ListParams {
            offset: page.offset(),
            limit: page.page_size(),
            status: query.status.as_deref(),
            source_type: query.source_type.as_deref(),
            source_id: query.source_id.as_deref(),
            from: &window.from,
            to: &window.to,
        },
    )
    .await?;
    Ok(Page { data: rows.into_iter().map(IncidentSummary::from).collect(), total, success: true })
}

pub(crate) async fn get(pool: &SqlitePool, id: &str) -> Result<IncidentDetail, AppError> {
    let row = repo::get(pool, id).await?.ok_or_else(|| AppError::not_found("incident"))?;
    let details = serde_json::from_str(&row.details).map_err(|error| {
        tracing::error!(incident_id = %row.id, %error, "Stored incident details are not valid JSON");
        AppError::database()
    })?;
    let (node, check) = match row.source_type.as_str() {
        "node" | "resource" => (repo::node_context(pool, &row.source_id).await?, None),
        "check" => (None, repo::check_context(pool, &row.source_id).await?),
        _ => (None, None),
    };
    Ok(IncidentDetail { summary: IncidentSummary::from(row), details, node, check })
}

pub(crate) async fn observe(
    pool: &SqlitePool,
    source_type: &str,
    source_id: &str,
    kind: &str,
    title: &str,
    details: serde_json::Value,
    active: bool,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    if active {
        let id = Uuid::new_v4().to_string();
        let details = details.to_string();
        repo::upsert_active(
            pool,
            repo::ActiveIncident {
                id: &id,
                source_type,
                source_id,
                kind,
                title,
                details: &details,
                observed_at: &now,
            },
        )
        .await?;
    } else {
        repo::resolve_active(pool, source_type, source_id, kind, &now).await?;
    }
    Ok(())
}

pub(crate) async fn evaluate_once(pool: &SqlitePool) -> Result<(), AppError> {
    let settings = crate::features::settings::service::get(pool).await?;
    let nodes = crate::features::nodes::service::list(pool).await?;
    for node in &nodes {
        evaluate_node(pool, node).await?;
        let samples =
            repo::resource_samples(pool, &node.row.id, settings.failure_threshold).await?;
        for (kind, title, threshold, values) in [
            (
                "cpu_high",
                format!("High CPU usage on {}", node.row.hostname),
                settings.cpu_threshold_percent,
                samples.iter().map(|sample| sample.cpu_percent).collect::<Vec<_>>(),
            ),
            (
                "memory_high",
                format!("High memory usage on {}", node.row.hostname),
                settings.memory_threshold_percent,
                samples.iter().map(|sample| sample.memory_percent).collect::<Vec<_>>(),
            ),
            (
                "disk_high",
                format!("High disk usage on {}", node.row.hostname),
                settings.disk_threshold_percent,
                samples.iter().map(|sample| sample.disk_percent).collect::<Vec<_>>(),
            ),
        ] {
            let active = values.len() == settings.failure_threshold as usize
                && values.iter().all(|value| *value >= threshold);
            observe(
                pool,
                "resource",
                &node.row.id,
                kind,
                &title,
                serde_json::json!({ "threshold": threshold, "samples": values }),
                active,
            )
            .await?;
        }
    }
    Ok(())
}

async fn evaluate_node(pool: &SqlitePool, node: &NodeView) -> Result<(), AppError> {
    observe(
        pool,
        "node",
        &node.row.id,
        "node_offline",
        &format!("Node {} is offline", node.row.hostname),
        serde_json::json!({ "lastSeenAt": node.row.last_seen_at }),
        node.status == "offline",
    )
    .await
}

pub(crate) async fn active_count(pool: &SqlitePool) -> Result<i64, AppError> {
    Ok(repo::active_count(pool).await?)
}

fn validate_status(status: Option<&str>) -> Result<(), AppError> {
    if status.is_some_and(|value| !matches!(value, "open" | "resolved" | "acknowledged")) {
        return Err(AppError::unprocessable("status must be open, resolved, or acknowledged"));
    }
    Ok(())
}

fn validate_source_type(source_type: Option<&str>) -> Result<(), AppError> {
    if source_type.is_some_and(|value| !matches!(value, "node" | "check" | "resource")) {
        return Err(AppError::unprocessable("sourceType must be node, check, or resource"));
    }
    Ok(())
}

fn validate_source_id(source_id: Option<&str>) -> Result<(), AppError> {
    if source_id.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 200) {
        return Err(AppError::unprocessable("sourceId must contain 1 to 200 characters"));
    }
    Ok(())
}

struct TimeWindow {
    from: String,
    to: String,
}

impl TimeWindow {
    fn parse(from: Option<&str>, to: Option<&str>) -> Result<Self, AppError> {
        let to = parse_time(to, Utc::now())?;
        let from = parse_time(from, to - MAX_QUERY_RANGE)?;
        if from > to {
            return Err(AppError::unprocessable("from must not be after to"));
        }
        if to - from > MAX_QUERY_RANGE {
            return Err(AppError::unprocessable("incident query range cannot exceed 90 days"));
        }
        Ok(Self { from: from.to_rfc3339(), to: to.to_rfc3339() })
    }
}

fn parse_time(value: Option<&str>, default: DateTime<Utc>) -> Result<DateTime<Utc>, AppError> {
    value
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map(|value| value.map(|value| value.with_timezone(&Utc)).unwrap_or(default))
        .map_err(|_| AppError::unprocessable("from and to must be RFC3339 timestamps"))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use crate::infra::db::migrated_test_pool;

    use super::{active_count, get, list, observe};
    use crate::features::incidents::types::ListQuery;

    #[tokio::test]
    async fn incident_lifecycle_deduplicates_and_resolves() {
        let pool = migrated_test_pool().await;
        for _ in 0..2 {
            observe(&pool, "check", "check-1", "tcp_down", "TCP down", serde_json::json!({}), true)
                .await
                .expect("observe active");
        }
        assert_eq!(active_count(&pool).await.expect("active count"), 1);
        observe(&pool, "check", "check-1", "tcp_down", "TCP down", serde_json::json!({}), false)
            .await
            .expect("resolve");
        assert_eq!(active_count(&pool).await.expect("resolved count"), 0);
    }

    #[tokio::test]
    async fn list_filters_incidents_by_status_source_and_last_observed_window() {
        let pool = migrated_test_pool().await;
        insert_node(&pool).await;
        insert_check(&pool).await;
        insert_incident(
            &pool,
            IncidentFixture {
                id: "resource-open",
                source_type: "resource",
                source_id: "node-1",
                kind: "cpu_high",
                title: "High CPU usage",
                status: "open",
                opened_at: "2026-08-01T00:00:00+00:00",
                last_observed_at: "2026-08-02T00:00:00+00:00",
                resolved_at: None,
                details: serde_json::json!({ "threshold": 90, "samples": [91, 92, 93] }),
            },
        )
        .await;
        insert_incident(
            &pool,
            IncidentFixture {
                id: "check-resolved",
                source_type: "check",
                source_id: "check-1",
                kind: "tcp_down",
                title: "TCP check down",
                status: "resolved",
                opened_at: "2026-07-01T00:00:00+00:00",
                last_observed_at: "2026-07-02T00:00:00+00:00",
                resolved_at: Some("2026-07-03T00:00:00+00:00"),
                details: serde_json::json!({ "host": "127.0.0.1", "port": 80 }),
            },
        )
        .await;

        let page = list(
            &pool,
            ListQuery {
                current: Some(1),
                page_size: Some(10),
                status: Some("open".to_string()),
                source_type: Some("resource".to_string()),
                source_id: None,
                from: Some("2026-08-01T00:00:00+00:00".to_string()),
                to: Some("2026-08-03T00:00:00+00:00".to_string()),
            },
        )
        .await
        .expect("filtered incidents");
        assert_eq!(page.total, 1);
        assert_eq!(page.data[0].id, "resource-open");
        assert_eq!(page.data[0].last_observed_at, "2026-08-02T00:00:00+00:00");
    }

    #[tokio::test]
    async fn detail_exposes_structured_evidence_and_associated_node() {
        let pool = migrated_test_pool().await;
        insert_node(&pool).await;
        insert_incident(
            &pool,
            IncidentFixture {
                id: "resource-open",
                source_type: "resource",
                source_id: "node-1",
                kind: "cpu_high",
                title: "High CPU usage",
                status: "open",
                opened_at: "2026-08-01T00:00:00+00:00",
                last_observed_at: "2026-08-02T00:00:00+00:00",
                resolved_at: None,
                details: serde_json::json!({ "threshold": 90, "samples": [91, 92, 93] }),
            },
        )
        .await;

        let detail = get(&pool, "resource-open").await.expect("incident detail");
        let value = serde_json::to_value(detail).expect("serialize detail");
        assert_eq!(value["status"], "open");
        assert_eq!(value["lastObservedAt"], "2026-08-02T00:00:00+00:00");
        assert_eq!(value["details"]["threshold"], 90);
        assert_eq!(value["details"]["samples"][2], 93);
        assert_eq!(value["node"]["hostname"], "monitor-node");
        assert!(value["check"].is_null());
    }

    #[tokio::test]
    async fn invalid_filters_are_unprocessable_and_missing_detail_is_not_found() {
        let pool = migrated_test_pool().await;
        let error = list(
            &pool,
            ListQuery {
                current: Some(1),
                page_size: Some(10),
                status: Some("suppressed".to_string()),
                source_type: None,
                source_id: None,
                from: None,
                to: None,
            },
        )
        .await
        .expect_err("invalid status");
        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY);

        let error = get(&pool, "missing").await.expect_err("missing detail");
        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    async fn insert_node(pool: &rustzen_storage::SqlitePool) {
        sqlx::query(
            "INSERT INTO monitor_nodes
             (id, agent_id, hostname, agent_version, last_seen_at, created_at, updated_at)
             VALUES ('node-1', 'agent-1', 'monitor-node', '0.5.0',
                     '2026-08-02T00:00:00+00:00', '2026-08-01T00:00:00+00:00',
                     '2026-08-02T00:00:00+00:00')",
        )
        .execute(pool)
        .await
        .expect("insert monitor node");
    }

    async fn insert_check(pool: &rustzen_storage::SqlitePool) {
        sqlx::query(
            "INSERT INTO monitor_checks
             (id, name, host, port, interval_seconds, timeout_ms, failure_threshold,
              enabled, created_at, updated_at)
             VALUES ('check-1', 'HTTP check', '127.0.0.1', 80, 60, 5000, 3,
                     TRUE, '2026-07-01T00:00:00+00:00', '2026-07-01T00:00:00+00:00')",
        )
        .execute(pool)
        .await
        .expect("insert monitor check");
    }

    struct IncidentFixture<'a> {
        id: &'a str,
        source_type: &'a str,
        source_id: &'a str,
        kind: &'a str,
        title: &'a str,
        status: &'a str,
        opened_at: &'a str,
        last_observed_at: &'a str,
        resolved_at: Option<&'a str>,
        details: serde_json::Value,
    }

    async fn insert_incident(pool: &rustzen_storage::SqlitePool, incident: IncidentFixture<'_>) {
        sqlx::query(
            "INSERT INTO monitor_incidents
             (id, source_type, source_id, kind, title, status, details,
              opened_at, resolved_at, last_observed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(incident.id)
        .bind(incident.source_type)
        .bind(incident.source_id)
        .bind(incident.kind)
        .bind(incident.title)
        .bind(incident.status)
        .bind(incident.details.to_string())
        .bind(incident.opened_at)
        .bind(incident.resolved_at)
        .bind(incident.last_observed_at)
        .execute(pool)
        .await
        .expect("insert monitor incident");
    }
}
