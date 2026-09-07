use chrono::{DateTime, Duration, Utc};
use rustzen_ipc::{
    NotificationAudience, NotificationContent, NotificationEvent, NotificationSubject,
};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Sqlite, Transaction};
use uuid::Uuid;

const PENDING_ROWS: i64 = 100_000;
const PENDING_BYTES: i64 = 64 * 1024 * 1024;

#[derive(FromRow)]
struct IncidentTransition {
    id: String,
    title: String,
}

#[expect(clippy::too_many_arguments, reason = "incident transition owns its persisted fields")]
pub(crate) async fn open(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    title: &str,
    threshold: Option<f64>,
    observed: Option<f64>,
    now: &str,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let inserted = sqlx::query_as::<_, IncidentTransition>(
        "INSERT INTO monitor_incidents
         (id,node_id,kind,target,status,title,threshold_percent,observed_percent,opened_at,last_observed_at)
         VALUES(?,?,?,?, 'active', ?,?,?,?,?)
         ON CONFLICT(node_id,kind,target) WHERE status='active' DO NOTHING
         RETURNING id,title",
    )
    .bind(&id).bind(node).bind(kind).bind(target).bind(title).bind(threshold).bind(observed)
    .bind(now).bind(now).fetch_optional(&mut **tx).await?;
    if let Some(incident) = inserted {
        enqueue(tx, &incident, 1, "monitor.incident.opened", now, "Incident opened").await?;
    } else {
        sqlx::query(
            "UPDATE monitor_incidents SET observed_percent=COALESCE(?,observed_percent),
             last_observed_at=? WHERE node_id=? AND kind=? AND target=? AND status='active'",
        )
        .bind(observed)
        .bind(now)
        .bind(node)
        .bind(kind)
        .bind(target)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(crate) async fn resolve_exact(
    tx: &mut Transaction<'_, Sqlite>,
    node: &str,
    kind: &str,
    target: &str,
    now: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    let incident = sqlx::query_as::<_, IncidentTransition>(
        "UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason=?,last_observed_at=?
         WHERE node_id=? AND kind=? AND target=? AND status='active' RETURNING id,title",
    ).bind(now).bind(reason).bind(now).bind(node).bind(kind).bind(target)
      .fetch_optional(&mut **tx).await?;
    if let Some(incident) = incident {
        enqueue(tx, &incident, 2, "monitor.incident.resolved", now, "Incident resolved").await?;
    }
    Ok(())
}

pub(crate) async fn resolve_id(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    now: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    let incident = sqlx::query_as::<_, IncidentTransition>(
        "UPDATE monitor_incidents SET status='resolved',resolved_at=?,resolution_reason=?,last_observed_at=?
         WHERE id=? AND status='active' RETURNING id,title",
    ).bind(now).bind(reason).bind(now).bind(id).fetch_optional(&mut **tx).await?;
    if let Some(incident) = incident {
        enqueue(tx, &incident, 2, "monitor.incident.resolved", now, "Incident resolved").await?;
    }
    Ok(())
}

async fn enqueue(
    tx: &mut Transaction<'_, Sqlite>,
    incident: &IncidentTransition,
    revision: i64,
    topic: &str,
    occurred_at: &str,
    summary: &str,
) -> Result<(), sqlx::Error> {
    let occurred = DateTime::parse_from_rfc3339(occurred_at)
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?
        .with_timezone(&Utc);
    delete_expired(tx, occurred).await?;
    let event = NotificationEvent {
        schema_version: 1,
        event_id: Uuid::new_v4().to_string(),
        producer: "monitor".into(),
        topic: topic.into(),
        occurred_at: occurred.to_rfc3339(),
        expires_at: (occurred + Duration::days(7)).to_rfc3339(),
        subject: NotificationSubject {
            kind: "monitor-incident".into(),
            id: incident.id.clone(),
            revision,
        },
        audience: NotificationAudience {
            policy: "monitor-incident-readers".into(),
            initiator_user_id: None,
        },
        content: NotificationContent { title: incident.title.clone(), summary: summary.into() },
    };
    let payload =
        serde_json::to_string(&event).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let digest = hex::encode(Sha256::digest(payload.as_bytes()));
    let charge = 512.max(payload.len() as i64 + 256);
    let (count, bytes) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT pending_count,pending_bytes FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&mut **tx)
    .await?;
    if count >= PENDING_ROWS || bytes.saturating_add(charge) > PENDING_BYTES {
        record_gap(tx, occurred_at, "omitted_count").await?;
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO notification_outbox
         (event_id,topic,subject_kind,subject_id,subject_revision,payload_json,payload_sha256,
          occurred_at,expires_at,state,next_attempt_at,charged_bytes)
         VALUES(?,?,?,?,?,?,?,?,?,'pending',?,?)",
    )
    .bind(&event.event_id)
    .bind(topic)
    .bind(&event.subject.kind)
    .bind(&event.subject.id)
    .bind(revision)
    .bind(payload)
    .bind(digest)
    .bind(&event.occurred_at)
    .bind(&event.expires_at)
    .bind(&event.occurred_at)
    .bind(charge)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn delete_expired(
    tx: &mut Transaction<'_, Sqlite>,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let now = now.to_rfc3339();
    let expired = sqlx::query_scalar::<_, String>(
        "SELECT event_id FROM notification_outbox WHERE state='pending' AND expires_at<=?
         AND lease_token IS NULL ORDER BY expires_at LIMIT 100",
    )
    .bind(&now)
    .fetch_all(&mut **tx)
    .await?;
    for id in expired {
        let deleted =
            sqlx::query("DELETE FROM notification_outbox WHERE event_id=? AND lease_token IS NULL")
                .bind(id)
                .execute(&mut **tx)
                .await?
                .rows_affected();
        if deleted == 1 {
            record_gap(tx, &now, "expired_count").await?;
        }
    }
    Ok(())
}

async fn record_gap(
    tx: &mut Transaction<'_, Sqlite>,
    at: &str,
    counter: &str,
) -> Result<(), sqlx::Error> {
    let sql = match counter {
        "omitted_count" => {
            "UPDATE notification_delivery_status SET omitted_count=omitted_count+1, first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
        }
        "expired_count" => {
            "UPDATE notification_delivery_status SET expired_count=expired_count+1, first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
        }
        _ => return Err(sqlx::Error::Protocol("unsupported notification gap counter".into())),
    };
    sqlx::query(sql).bind(at).bind(at).execute(&mut **tx).await?;
    Ok(())
}
