use chrono::{DateTime, Duration, Utc};
use rustzen_storage::SqlitePool;
use sqlx::FromRow;
use std::{future::Future, pin::Pin};
use uuid::Uuid;

use super::cleanup::{expire_terminal, gap, trim_quarantine};

const LEASE_SECONDS: i64 = 30;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct Claim {
    pub event_id: String,
    pub payload_json: String,
    pub payload_sha256: String,
    pub expires_at: String,
    pub lease_token: String,
    pub attempts: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum DeliveryResult {
    Stored,
    Duplicate,
    NoRecipients,
    Conflict,
    Expired,
    Invalid,
    Unauthorized,
    RateLimited { retry_after_seconds: i64 },
    Capacity { retry_after_seconds: i64 },
    Unavailable,
    Timeout,
    Ambiguous,
}

pub(crate) trait Transport: Send + Sync {
    fn send<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>>;
}

#[cfg(test)]
pub(crate) async fn run_once_at(
    pool: &SqlitePool,
    transport: &dyn Transport,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    run_once_with_clock(pool, transport, &mut || now).await
}

#[cfg(test)]
pub(crate) async fn run_once_between(
    pool: &SqlitePool,
    transport: &dyn Transport,
    started: DateTime<Utc>,
    completed: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut times = [started, completed].into_iter();
    run_once_with_clock(pool, transport, &mut || times.next().unwrap_or(completed)).await
}

async fn run_once_with_clock(
    pool: &SqlitePool,
    transport: &dyn Transport,
    clock: &mut impl FnMut() -> DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let now = clock();
    expire_terminal(pool, now).await?;
    let Some(claim) = claim(pool, now).await? else {
        return Ok(false);
    };
    let result = transport.send(&claim).await;
    finish(pool, &claim, result, clock()).await?;
    Ok(true)
}

pub(crate) async fn run_batch(
    pool: &SqlitePool,
    transport: &dyn Transport,
    limit: usize,
) -> Result<usize, sqlx::Error> {
    run_batch_with_clock(pool, transport, limit, Utc::now).await
}

#[cfg(test)]
pub(crate) async fn run_batch_at(
    pool: &SqlitePool,
    transport: &dyn Transport,
    limit: usize,
    now: DateTime<Utc>,
) -> Result<usize, sqlx::Error> {
    run_batch_with_clock(pool, transport, limit, || now).await
}

async fn run_batch_with_clock(
    pool: &SqlitePool,
    transport: &dyn Transport,
    limit: usize,
    mut clock: impl FnMut() -> DateTime<Utc>,
) -> Result<usize, sqlx::Error> {
    let mut delivered = 0;
    for _ in 0..limit.min(25) {
        if !run_once_with_clock(pool, transport, &mut clock).await? {
            break;
        }
        delivered += 1;
    }
    Ok(delivered)
}

pub(crate) async fn claim(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<Option<Claim>, sqlx::Error> {
    let now_text = now.to_rfc3339();
    let lease_until = (now + Duration::seconds(LEASE_SECONDS)).to_rfc3339();
    let token = Uuid::new_v4().to_string();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let id = sqlx::query_scalar::<_, String>(
        "SELECT o.event_id FROM notification_outbox o
         WHERE o.state IN ('pending','reconciling') AND o.next_attempt_at<=?
           AND (o.lease_until IS NULL OR o.lease_until<=?)
           AND ((o.state='pending' AND o.expires_at>?) OR
                (o.state='reconciling' AND o.reconcile_until>?))
           AND NOT EXISTS (
             SELECT 1 FROM notification_outbox earlier
             WHERE earlier.subject_kind=o.subject_kind AND earlier.subject_id=o.subject_id
               AND earlier.subject_revision<o.subject_revision
               AND earlier.state IN ('pending','reconciling')
           )
         ORDER BY o.occurred_at,o.event_id LIMIT 1",
    )
    .bind(&now_text)
    .bind(&now_text)
    .bind(&now_text)
    .bind(&now_text)
    .fetch_optional(&mut *tx)
    .await?;
    let result = if let Some(id) = id {
        sqlx::query_as::<_, Claim>(
            "UPDATE notification_outbox SET lease_until=?,lease_token=?,attempts=attempts+1
             WHERE event_id=? AND (lease_until IS NULL OR lease_until<=?)
             RETURNING event_id,payload_json,payload_sha256,expires_at,lease_token,attempts",
        )
        .bind(lease_until)
        .bind(token)
        .bind(id)
        .bind(&now_text)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        None
    };
    tx.commit().await?;
    Ok(result)
}

pub(crate) async fn finish(
    pool: &SqlitePool,
    claim: &Claim,
    result: DeliveryResult,
    now: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let now_text = now.to_rfc3339();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let changed = match result {
        DeliveryResult::Stored | DeliveryResult::Duplicate | DeliveryResult::NoRecipients => {
            let changed = delete_claim(&mut tx, claim).await?;
            if changed == 1 {
                sqlx::query("UPDATE notification_delivery_status SET last_success_at=? WHERE id=1")
                    .bind(&now_text)
                    .execute(&mut *tx)
                    .await?;
            }
            changed
        }
        DeliveryResult::Expired => {
            let changed = delete_claim(&mut tx, claim).await?;
            if changed == 1 {
                gap(&mut tx, "expired", &now_text).await?;
            }
            changed
        }
        DeliveryResult::Conflict | DeliveryResult::Invalid | DeliveryResult::Unauthorized => {
            let code = match result {
                DeliveryResult::Conflict => "conflict",
                DeliveryResult::Invalid => "invalid",
                _ => "unauthorized",
            };
            let changed = sqlx::query(
                "UPDATE notification_outbox SET state='quarantined',lease_until=NULL,lease_token=NULL,
                 last_error_code=? WHERE event_id=? AND lease_token=? AND state IN ('pending','reconciling')",
            ).bind(code).bind(&claim.event_id).bind(&claim.lease_token)
             .execute(&mut *tx).await?.rows_affected();
            if changed == 1 {
                sqlx::query("UPDATE notification_delivery_status SET quarantined_count=quarantined_count+1, first_gap_at=COALESCE(first_gap_at,?),last_gap_at=? WHERE id=1")
                    .bind(&now_text).bind(&now_text).execute(&mut *tx).await?;
                trim_quarantine(&mut tx).await?;
            }
            changed
        }
        DeliveryResult::RateLimited { .. }
        | DeliveryResult::Capacity { .. }
        | DeliveryResult::Unavailable => {
            let retry_after = match result {
                DeliveryResult::RateLimited { retry_after_seconds }
                | DeliveryResult::Capacity { retry_after_seconds } => retry_after_seconds,
                _ => 0,
            };
            let delay = backoff_seconds(&claim.event_id, claim.attempts).max(retry_after);
            let next_attempt = (now + Duration::seconds(delay)).min(parse(&claim.expires_at)?);
            sqlx::query(
                "UPDATE notification_outbox SET state='pending',lease_until=NULL,lease_token=NULL,
                 next_attempt_at=?,last_error_code=?
                 WHERE event_id=? AND lease_token=? AND state IN ('pending','reconciling')",
            )
            .bind(next_attempt.to_rfc3339())
            .bind(match result {
                DeliveryResult::RateLimited { .. } => "rate-limited",
                DeliveryResult::Capacity { .. } => "capacity",
                _ => "connect-unavailable",
            })
            .bind(&claim.event_id)
            .bind(&claim.lease_token)
            .execute(&mut *tx)
            .await?
            .rows_affected()
        }
        DeliveryResult::Timeout | DeliveryResult::Ambiguous => {
            let delay = backoff_seconds(&claim.event_id, claim.attempts);
            sqlx::query(
                "UPDATE notification_outbox SET state='reconciling',lease_until=NULL,lease_token=NULL,
                 reconcile_until=COALESCE(reconcile_until,?),next_attempt_at=?,last_error_code=?
                 WHERE event_id=? AND lease_token=? AND state IN ('pending','reconciling')",
            ).bind((parse(&claim.expires_at)? + Duration::seconds(60)).to_rfc3339())
             .bind((now + Duration::seconds(delay)).to_rfc3339())
             .bind(if result == DeliveryResult::Timeout { "timeout" } else { "ambiguous" })
             .bind(&claim.event_id).bind(&claim.lease_token).execute(&mut *tx).await?.rows_affected()
        }
    };
    tx.commit().await?;
    super::diagnostics::warn_after_commit(pool).await;
    Ok(changed)
}

pub(crate) fn backoff_seconds(event_id: &str, attempts: i64) -> i64 {
    use sha2::{Digest, Sha256};
    let exponent = attempts.saturating_sub(1).clamp(0, 6) as u32;
    let base = i64::min(60, 1_i64 << exponent);
    let digest = Sha256::digest(format!("{event_id}:{attempts}").as_bytes());
    i64::min(60, base + i64::from(digest[0] % 3))
}

async fn delete_claim(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    claim: &Claim,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM notification_outbox WHERE event_id=? AND lease_token=? AND state IN ('pending','reconciling')")
        .bind(&claim.event_id).bind(&claim.lease_token).execute(&mut **tx).await?.rows_affected())
}

fn parse(value: &str) -> Result<DateTime<Utc>, sqlx::Error> {
    DateTime::parse_from_rfc3339(value)
        .map(|v| v.with_timezone(&Utc))
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))
}
