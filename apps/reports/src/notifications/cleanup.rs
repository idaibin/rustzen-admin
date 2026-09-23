use chrono::{DateTime, Duration, Utc};
use rustzen_storage::SqlitePool;

const QUARANTINE_ROWS: i64 = 1_000;
const QUARANTINE_BYTES: i64 = 4 * 1024 * 1024;

pub(super) async fn expire_terminal(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let now_text = now.to_rfc3339();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query(
        "DELETE FROM notification_outbox WHERE event_id IN (
           SELECT event_id FROM notification_outbox
           WHERE state='quarantined' AND occurred_at<=?
           ORDER BY occurred_at,event_id LIMIT 100
         )",
    )
    .bind((now - Duration::hours(24)).to_rfc3339())
    .execute(&mut *tx)
    .await?;
    for (id, kind) in sqlx::query_as::<_, (String, String)>(
        "SELECT event_id,CASE WHEN state='reconciling' THEN 'unconfirmed' ELSE 'expired' END
         FROM notification_outbox WHERE (lease_token IS NULL OR lease_until<=?) AND
         ((state='pending' AND expires_at<=?) OR (state='reconciling' AND reconcile_until<=?))
         ORDER BY expires_at LIMIT 100",
    )
    .bind(&now_text)
    .bind(&now_text)
    .bind(&now_text)
    .fetch_all(&mut *tx)
    .await?
    {
        if sqlx::query(
            "DELETE FROM notification_outbox WHERE event_id=?
             AND (lease_token IS NULL OR lease_until<=?)",
        )
        .bind(id)
        .bind(&now_text)
        .execute(&mut *tx)
        .await?
        .rows_affected()
            == 1
        {
            gap(&mut tx, &kind, &now_text).await?;
        }
    }
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(())
}

pub(super) async fn gap(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    kind: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    let sql = if kind == "expired" {
        "UPDATE notification_delivery_status SET expired_count=expired_count+1,first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
    } else {
        "UPDATE notification_delivery_status SET unconfirmed_count=unconfirmed_count+1,first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1"
    };
    sqlx::query(sql).bind(now).bind(now).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn trim_quarantine(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), sqlx::Error> {
    loop {
        let (count, bytes) = sqlx::query_as::<_, (i64, i64)>(
            "SELECT quarantine_count,quarantine_bytes FROM notification_delivery_status WHERE id=1",
        )
        .fetch_one(&mut **tx)
        .await?;
        if count <= QUARANTINE_ROWS && bytes <= QUARANTINE_BYTES {
            break;
        }
        let deleted = sqlx::query("DELETE FROM notification_outbox WHERE event_id=(SELECT event_id FROM notification_outbox WHERE state='quarantined' ORDER BY occurred_at,event_id LIMIT 1)")
            .execute(&mut **tx).await?.rows_affected();
        if deleted == 0 {
            break;
        }
        sqlx::query("UPDATE notification_delivery_status SET quarantine_evicted_count=quarantine_evicted_count+1 WHERE id=1")
            .execute(&mut **tx).await?;
    }
    Ok(())
}
