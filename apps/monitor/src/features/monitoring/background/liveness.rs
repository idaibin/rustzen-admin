use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use sqlx::Row;
#[cfg(not(feature = "notifications"))]
use uuid::Uuid;

use super::super::acceptance::LockHook;

pub(crate) async fn offline_scan_at(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    offline_scan_at_inner(pool, now, None).await
}

#[cfg(test)]
pub(in super::super) async fn offline_scan_at_with_lock_hook(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    lock_hook: LockHook,
) -> Result<(), sqlx::Error> {
    offline_scan_at_inner(pool, now, Some(lock_hook)).await
}

async fn offline_scan_at_inner(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    lock_hook: Option<LockHook>,
) -> Result<(), sqlx::Error> {
    // Liveness must be decided against a snapshot that cannot be changed by a
    // concurrent report between the read and the incident insert.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if let Some(lock_hook) = lock_hook {
        lock_hook.acquired.notify_one();
        lock_hook.release.notified().await;
    }
    for node in sqlx::query(
        "SELECT n.node_id,n.last_received_at,
                COALESCE(s.offline_enabled,g.offline_enabled) AS offline_enabled,
                COALESCE(s.offline_after_seconds,g.offline_after_seconds)
                    AS offline_after_seconds
         FROM monitor_nodes n CROSS JOIN alert_settings g
         LEFT JOIN node_alert_settings s ON s.node_id=n.node_id
         WHERE g.id=1",
    )
    .fetch_all(&mut *tx)
    .await?
    {
        if node.get::<i64, _>("offline_enabled") == 0 {
            continue;
        }
        let id: String = node.get(0);
        let last: DateTime<Utc> = DateTime::parse_from_rfc3339(&node.get::<String, _>(1))
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(now);
        if (now - last).num_seconds() > node.get::<i64, _>("offline_after_seconds") {
            let t = now.to_rfc3339();
            #[cfg(feature = "notifications")]
            crate::notifications::outbox::open(
                &mut tx,
                &id,
                "nodeOffline",
                "node",
                "node offline",
                None,
                None,
                &t,
            )
            .await?;
            #[cfg(not(feature = "notifications"))]
            sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at) VALUES(?,?, 'nodeOffline','node','active','node offline',?,?) ON CONFLICT(node_id,kind,target) WHERE status='active' DO UPDATE SET last_observed_at=excluded.last_observed_at").bind(Uuid::new_v4().to_string()).bind(id).bind(&t).bind(&t).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    #[cfg(feature = "notifications")]
    crate::notifications::diagnostics::warn_after_commit(pool).await;
    Ok(())
}
