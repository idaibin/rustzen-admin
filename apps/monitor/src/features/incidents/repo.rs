use rustzen_storage::SqlitePool;

use super::types::{CheckContext, IncidentRow, NodeContext, ResourceSample};

pub(super) struct ListParams<'a> {
    pub offset: i64,
    pub limit: i64,
    pub status: Option<&'a str>,
    pub source_type: Option<&'a str>,
    pub source_id: Option<&'a str>,
    pub from: &'a str,
    pub to: &'a str,
}

pub(super) async fn list(
    pool: &SqlitePool,
    params: ListParams<'_>,
) -> Result<(Vec<IncidentRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as(
        "SELECT id, source_type, source_id, kind, title, status, details,
                opened_at, acknowledged_at, resolved_at, last_observed_at
         FROM monitor_incidents
         WHERE (? IS NULL OR status = ?)
           AND (? IS NULL OR source_type = ?)
           AND (? IS NULL OR source_id = ?)
           AND last_observed_at >= ?
           AND last_observed_at <= ?
         ORDER BY last_observed_at DESC, opened_at DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(params.status)
    .bind(params.status)
    .bind(params.source_type)
    .bind(params.source_type)
    .bind(params.source_id)
    .bind(params.source_id)
    .bind(params.from)
    .bind(params.to)
    .bind(params.limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monitor_incidents
         WHERE (? IS NULL OR status = ?)
           AND (? IS NULL OR source_type = ?)
           AND (? IS NULL OR source_id = ?)
           AND last_observed_at >= ?
           AND last_observed_at <= ?",
    )
    .bind(params.status)
    .bind(params.status)
    .bind(params.source_type)
    .bind(params.source_type)
    .bind(params.source_id)
    .bind(params.source_id)
    .bind(params.from)
    .bind(params.to)
    .fetch_one(pool)
    .await?;
    Ok((rows, total))
}

pub(super) async fn get(pool: &SqlitePool, id: &str) -> Result<Option<IncidentRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, source_type, source_id, kind, title, status, details,
                opened_at, acknowledged_at, resolved_at, last_observed_at
         FROM monitor_incidents WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(super) async fn node_context(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<NodeContext>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, agent_id, hostname, agent_version, last_seen_at
         FROM monitor_nodes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(super) async fn check_context(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<CheckContext>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, host, port, last_status, last_checked_at, consecutive_failures
         FROM monitor_checks WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(super) struct ActiveIncident<'a> {
    pub id: &'a str,
    pub source_type: &'a str,
    pub source_id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
    pub details: &'a str,
    pub observed_at: &'a str,
}

pub(super) async fn upsert_active(
    pool: &SqlitePool,
    incident: ActiveIncident<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO monitor_incidents (
           id, source_type, source_id, kind, title, status, details,
           opened_at, last_observed_at
         ) VALUES (?, ?, ?, ?, ?, 'open', ?, ?, ?)
         ON CONFLICT(source_type, source_id, kind)
           WHERE status IN ('open', 'acknowledged')
         DO UPDATE SET title = excluded.title, details = excluded.details,
                       last_observed_at = excluded.last_observed_at",
    )
    .bind(incident.id)
    .bind(incident.source_type)
    .bind(incident.source_id)
    .bind(incident.kind)
    .bind(incident.title)
    .bind(incident.details)
    .bind(incident.observed_at)
    .bind(incident.observed_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn resolve_active(
    pool: &SqlitePool,
    source_type: &str,
    source_id: &str,
    kind: &str,
    resolved_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE monitor_incidents
         SET status = 'resolved', resolved_at = ?, last_observed_at = ?
         WHERE source_type = ? AND source_id = ? AND kind = ?
           AND status IN ('open', 'acknowledged')",
    )
    .bind(resolved_at)
    .bind(resolved_at)
    .bind(source_type)
    .bind(source_id)
    .bind(kind)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn resource_samples(
    pool: &SqlitePool,
    node_id: &str,
    limit: i64,
) -> Result<Vec<ResourceSample>, sqlx::Error> {
    sqlx::query_as(
        "SELECT CAST(cpu_percent AS REAL) AS cpu_percent,
                CASE WHEN memory_total_bytes > 0
                     THEN memory_used_bytes * 100.0 / memory_total_bytes ELSE 0 END AS memory_percent,
                CASE WHEN disk_total_bytes > 0
                     THEN disk_used_bytes * 100.0 / disk_total_bytes ELSE 0 END AS disk_percent
         FROM monitor_metrics WHERE node_id = ?
         ORDER BY collected_at DESC LIMIT ?",
    )
    .bind(node_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub(super) async fn active_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM monitor_incidents WHERE status IN ('open', 'acknowledged')",
    )
    .fetch_one(pool)
    .await
}
