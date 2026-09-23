use chrono::{DateTime, Utc};
use rustzen_storage::SqlitePool;
use std::sync::atomic::{AtomicI64, Ordering};

const WARNING_INTERVAL_SECONDS: i64 = 60;
static LIMITER: GapWarningLimiter = GapWarningLimiter::new();

pub(crate) async fn warn_after_commit(pool: &SqlitePool) {
    if let Err(error) = inspect(pool, &LIMITER, Utc::now()).await {
        tracing::warn!(%error, "Reports notification gap diagnostic failed after commit");
    }
}

pub(super) struct GapWarningLimiter {
    observed_total: AtomicI64,
    next_warning_at: AtomicI64,
}

impl GapWarningLimiter {
    const fn new() -> Self {
        Self { observed_total: AtomicI64::new(0), next_warning_at: AtomicI64::new(0) }
    }

    fn observe(&self, total: i64, now: i64) -> bool {
        let previous = self.observed_total.swap(total, Ordering::AcqRel);
        if total <= previous {
            return false;
        }
        let mut next = self.next_warning_at.load(Ordering::Acquire);
        loop {
            if now < next {
                return false;
            }
            match self.next_warning_at.compare_exchange(
                next,
                now + WARNING_INTERVAL_SECONDS,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(observed) => next = observed,
            }
        }
    }
}

async fn inspect(
    pool: &SqlitePool,
    limiter: &GapWarningLimiter,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let (omitted, expired, unconfirmed, quarantined, evicted) =
        sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            "SELECT omitted_count,expired_count,unconfirmed_count,quarantined_count,
                quarantine_evicted_count
         FROM notification_delivery_status WHERE id=1",
        )
        .fetch_one(pool)
        .await?;
    let total = omitted
        .saturating_add(expired)
        .saturating_add(unconfirmed)
        .saturating_add(quarantined)
        .saturating_add(evicted);
    let emit = limiter.observe(total, now.timestamp());
    if emit {
        tracing::warn!(
            omitted_count = omitted,
            expired_count = expired,
            unconfirmed_count = unconfirmed,
            quarantined_count = quarantined,
            quarantine_evicted_count = evicted,
            "Reports notification delivery gap committed"
        );
    }
    Ok(emit)
}

#[cfg(test)]
pub(super) async fn inspect_after_commit_at(
    pool: &SqlitePool,
    limiter: &GapWarningLimiter,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    inspect(pool, limiter, now).await
}

#[cfg(test)]
pub(super) fn test_limiter() -> GapWarningLimiter {
    GapWarningLimiter::new()
}
