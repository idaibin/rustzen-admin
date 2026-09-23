use chrono::{DateTime, Duration, Utc};
use rustzen_ipc::{
    NotificationAudience, NotificationContent, NotificationEvent, NotificationSubject,
};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Sqlite, Transaction};
use uuid::Uuid;

use rustzen_storage::SqlitePool;

const PENDING_ROWS: i64 = 100_000;
const PENDING_BYTES: i64 = 64 * 1024 * 1024;

#[derive(FromRow)]
struct Transition {
    initiator_user_id: Option<i64>,
}

pub(crate) async fn finish(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    error: Option<&str>,
    now: &str,
) -> Result<bool, sqlx::Error> {
    let topic = match status {
        "succeeded" => "reports.run.completed",
        "failed" => "reports.run.failed",
        _ => return Err(sqlx::Error::Protocol("invalid report terminal status".into())),
    };
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let transition = sqlx::query_as::<_, Transition>(
        "UPDATE automation_runs SET status=?,error=?,finished_at=?
         WHERE id=? AND status='running' AND cancel_requested_at IS NULL
         RETURNING initiator_user_id",
    )
    .bind(status)
    .bind(error)
    .bind(now)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(transition) = &transition {
        enqueue(&mut tx, id, transition.initiator_user_id, topic, now).await?;
    }
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(transition.is_some())
}

pub(crate) async fn cancel(pool: &SqlitePool, id: &str, now: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let transition = sqlx::query_as::<_, Transition>(
        "UPDATE automation_runs SET status='cancelled',finished_at=?
         WHERE id=? AND status='queued' RETURNING initiator_user_id",
    )
    .bind(now)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let changed = if let Some(transition) = &transition {
        enqueue(&mut tx, id, transition.initiator_user_id, "reports.run.cancelled", now).await?;
        true
    } else {
        sqlx::query(
            "UPDATE automation_runs SET cancel_requested_at=?
             WHERE id=? AND status='running' AND cancel_requested_at IS NULL",
        )
        .bind(now)
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
            == 1
    };
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(changed)
}

pub(crate) async fn finish_cancelled(
    pool: &SqlitePool,
    id: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let transition = sqlx::query_as::<_, Transition>(
        "UPDATE automation_runs SET status='cancelled',finished_at=?
         WHERE id=? AND status='running' AND cancel_requested_at IS NOT NULL
         RETURNING initiator_user_id",
    )
    .bind(now)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(transition) = &transition {
        enqueue(&mut tx, id, transition.initiator_user_id, "reports.run.cancelled", now).await?;
    }
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(transition.is_some())
}

pub(crate) async fn recover(pool: &SqlitePool, now: &str) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let runs = sqlx::query_as::<_, (String, Option<i64>, bool)>(
        "SELECT id,initiator_user_id,cancel_requested_at IS NOT NULL
         FROM automation_runs WHERE status='running' ORDER BY created_at,id",
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut changed = 0;
    for (id, initiator, cancelled) in runs {
        let status = if cancelled { "cancelled" } else { "failed" };
        let topic = if cancelled { "reports.run.cancelled" } else { "reports.run.failed" };
        let rows = sqlx::query(
            "UPDATE automation_runs SET status=?,error=CASE WHEN ?='failed'
             THEN 'Interrupted by service restart' ELSE error END,finished_at=?
             WHERE id=? AND status='running'",
        )
        .bind(status)
        .bind(status)
        .bind(now)
        .bind(&id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if rows == 1 {
            changed += 1;
            enqueue(&mut tx, &id, initiator, topic, now).await?;
        }
    }
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(changed)
}

async fn enqueue(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    initiator: Option<i64>,
    topic: &str,
    occurred_at: &str,
) -> Result<(), sqlx::Error> {
    let Some(initiator_user_id) = initiator else {
        return Ok(());
    };
    let occurred = DateTime::parse_from_rfc3339(occurred_at)
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?
        .with_timezone(&Utc);
    delete_expired(tx, occurred).await?;
    let label = match topic {
        "reports.run.completed" => "completed",
        "reports.run.failed" => "failed",
        _ => "cancelled",
    };
    let event = NotificationEvent {
        schema_version: 1,
        event_id: Uuid::new_v4().to_string(),
        producer: "reports".into(),
        topic: topic.into(),
        occurred_at: occurred.to_rfc3339(),
        expires_at: (occurred + Duration::days(7)).to_rfc3339(),
        subject: NotificationSubject { kind: "reports-run".into(), id: run_id.into(), revision: 1 },
        audience: NotificationAudience {
            policy: "reports-run-initiator".into(),
            initiator_user_id: Some(initiator_user_id),
        },
        content: NotificationContent {
            title: format!("Report run {label}"),
            summary: format!("Your report run {label}."),
        },
    };
    let payload =
        serde_json::to_string(&event).map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
    let digest = hex::encode(Sha256::digest(payload.as_bytes()));
    let charge = 512.max(payload.len() as i64 + 256);
    let (count, bytes) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT pending_count,pending_bytes FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&mut **tx)
    .await?;
    if count >= PENDING_ROWS || bytes.saturating_add(charge) > PENDING_BYTES {
        gap(tx, occurred_at, "omitted_count").await?;
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
    .bind(run_id)
    .bind(1_i64)
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
         AND lease_token IS NULL ORDER BY expires_at,event_id LIMIT 100",
    )
    .bind(&now)
    .fetch_all(&mut **tx)
    .await?;
    for id in expired {
        if sqlx::query("DELETE FROM notification_outbox WHERE event_id=? AND lease_token IS NULL")
            .bind(id)
            .execute(&mut **tx)
            .await?
            .rows_affected()
            == 1
        {
            gap(tx, &now, "expired_count").await?;
        }
    }
    Ok(())
}

async fn gap(tx: &mut Transaction<'_, Sqlite>, at: &str, counter: &str) -> Result<(), sqlx::Error> {
    let sql = match counter {
        "omitted_count" => {
            "UPDATE notification_delivery_status SET omitted_count=omitted_count+1,first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
        }
        "expired_count" => {
            "UPDATE notification_delivery_status SET expired_count=expired_count+1,first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
        }
        _ => return Err(sqlx::Error::Protocol("unsupported notification gap counter".into())),
    };
    sqlx::query(sql).bind(at).bind(at).execute(&mut **tx).await?;
    Ok(())
}
