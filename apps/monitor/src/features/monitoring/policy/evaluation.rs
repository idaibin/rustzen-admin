use sqlx::{Sqlite, Transaction};
#[cfg(not(feature = "notifications"))]
use uuid::Uuid;

#[expect(
    clippy::too_many_arguments,
    reason = "evaluation needs the report target and configured threshold"
)]
pub(in super::super) async fn evaluate(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    value: f64,
    enabled: i64,
    threshold: f64,
    now: &str,
) -> Result<(), sqlx::Error> {
    if enabled == 0 {
        return Ok(());
    }
    let high = value >= threshold;
    sqlx::query("INSERT INTO alert_counters(node_id,kind,target,abnormal_count,normal_count) VALUES(?,?,?,?,?) ON CONFLICT(node_id,kind,target) DO UPDATE SET abnormal_count=CASE WHEN ? THEN abnormal_count+1 ELSE 0 END,normal_count=CASE WHEN ? THEN 0 ELSE normal_count+1 END").bind(node).bind(kind).bind(target).bind(if high{1}else{0}).bind(if high{0}else{1}).bind(high).bind(high).execute(&mut **tx).await?;
    let (a,n):(i64,i64)=sqlx::query_as("SELECT abnormal_count,normal_count FROM alert_counters WHERE node_id=? AND kind=? AND target=?").bind(node).bind(kind).bind(target).fetch_one(&mut **tx).await?;
    if a >= 3 {
        #[cfg(feature = "notifications")]
        crate::notifications::outbox::open(
            tx,
            node,
            kind,
            target,
            &format!("{kind} threshold exceeded"),
            Some(threshold),
            Some(value),
            now,
        )
        .await?;
        #[cfg(not(feature = "notifications"))]
        sqlx::query("INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,threshold_percent,observed_percent,opened_at,last_observed_at) VALUES(?,?,?,?, 'active', ?,?,?,?,?) ON CONFLICT(node_id,kind,target) WHERE status='active' DO UPDATE SET observed_percent=excluded.observed_percent,last_observed_at=excluded.last_observed_at").bind(Uuid::new_v4().to_string()).bind(node).bind(kind).bind(target).bind(format!("{kind} threshold exceeded")).bind(threshold).bind(value).bind(now).bind(now).execute(&mut **tx).await?;
    }
    if n >= 3 {
        resolve(tx, node, kind, target, now, "normal samples").await?;
    }
    Ok(())
}
pub(in super::super) async fn resolve(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    now: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    #[cfg(feature = "notifications")]
    return crate::notifications::outbox::resolve_exact(tx, node, kind, target, now, reason).await;
    #[cfg(not(feature = "notifications"))]
    sqlx::query("UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason=?,last_observed_at=? WHERE node_id=? AND kind=? AND target=? AND status='active'").bind(now).bind(reason).bind(now).bind(node).bind(kind).bind(target).execute(&mut **tx).await?;
    #[cfg(not(feature = "notifications"))]
    Ok(())
}
