use chrono::NaiveDateTime;
use sqlx::SqliteConnection;
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

pub(crate) const RETENTION_DAYS: i64 = 30;
const ROW_LIMIT: i64 = 500;
const CHARGE_LIMIT: i64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CleanupLimits {
    pub rows: i64,
    pub charged_bytes: i64,
    pub time: Duration,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct CleanupResult {
    pub recipients: u64,
    pub messages: u64,
    pub receipts: u64,
    pub states: u64,
    pub charged_bytes: i64,
}

impl CleanupResult {
    pub(crate) fn made_progress(&self) -> bool {
        self.recipients + self.messages + self.receipts + self.states > 0
    }
}

pub(crate) async fn cleanup(
    connection: &mut SqliteConnection,
    now: NaiveDateTime,
    time_limit: Duration,
) -> Result<CleanupResult, sqlx::Error> {
    cleanup_with_limits(
        connection,
        now,
        CleanupLimits { rows: ROW_LIMIT, charged_bytes: CHARGE_LIMIT, time: time_limit },
    )
    .await
}

pub(crate) async fn cleanup_with_limits(
    connection: &mut SqliteConnection,
    now: NaiveDateTime,
    limits: CleanupLimits,
) -> Result<CleanupResult, sqlx::Error> {
    let deadline = Instant::now() + limits.time;
    let cutoff = now - chrono::Duration::days(RETENTION_DAYS);
    let mut result = CleanupResult::default();
    let recipients = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT nr.notification_id, nr.user_id, nr.charged_bytes
         FROM notification_recipients nr
         JOIN notifications n ON n.id = nr.notification_id
         WHERE n.accepted_at <= ? ORDER BY n.accepted_at, nr.notification_id, nr.user_id LIMIT ?",
    )
    .bind(cutoff)
    .bind(limits.rows)
    .fetch_all(&mut *connection)
    .await?;
    let mut selected = Vec::new();
    for row @ (_, _, charge) in recipients {
        if Instant::now() >= deadline || result.charged_bytes + charge > limits.charged_bytes {
            break;
        }
        result.charged_bytes += charge;
        selected.push(row);
    }
    let affected_users: BTreeSet<_> = selected.iter().map(|(_, user_id, _)| *user_id).collect();
    for user_id in affected_users {
        sqlx::query("UPDATE notification_user_state SET revision = revision + 1 WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *connection)
            .await?;
    }
    for (notification_id, user_id, charge) in selected {
        let deleted = sqlx::query(
            "DELETE FROM notification_recipients WHERE notification_id = ? AND user_id = ?",
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&mut *connection)
        .await?
        .rows_affected();
        result.recipients += deleted;
        if deleted == 0 {
            result.charged_bytes -= charge;
        }
    }
    if Instant::now() < deadline {
        let messages = sqlx::query_as::<_, (String, i64)>(
            "SELECT n.id, n.charged_bytes FROM notifications n
             WHERE n.accepted_at <= ?
               AND NOT EXISTS (SELECT 1 FROM notification_recipients nr WHERE nr.notification_id = n.id)
             ORDER BY n.accepted_at, n.inbox_seq LIMIT ?",
        )
        .bind(cutoff)
        .bind(limits.rows)
        .fetch_all(&mut *connection)
        .await?;
        for (id, charge) in messages {
            if Instant::now() >= deadline || result.charged_bytes + charge > limits.charged_bytes {
                break;
            }
            let deleted = sqlx::query("DELETE FROM notifications WHERE id = ?")
                .bind(id)
                .execute(&mut *connection)
                .await?
                .rows_affected();
            result.messages += deleted;
            result.charged_bytes += charge * deleted as i64;
        }
    }
    if Instant::now() < deadline {
        let receipts = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT r.producer, r.event_id, r.charged_bytes FROM notification_receipts r
             WHERE r.retain_until <= ?
               AND NOT EXISTS (SELECT 1 FROM notifications n
                               WHERE n.producer = r.producer AND n.event_id = r.event_id)
             ORDER BY r.retain_until, r.producer, r.event_id LIMIT ?",
        )
        .bind(now)
        .bind(limits.rows)
        .fetch_all(&mut *connection)
        .await?;
        for (producer, event_id, charge) in receipts {
            if Instant::now() >= deadline || result.charged_bytes + charge > limits.charged_bytes {
                break;
            }
            let deleted = sqlx::query(
                "DELETE FROM notification_receipts WHERE producer = ? AND event_id = ?",
            )
            .bind(producer)
            .bind(event_id)
            .execute(&mut *connection)
            .await?
            .rows_affected();
            result.receipts += deleted;
            result.charged_bytes += charge * deleted as i64;
        }
    }
    if Instant::now() < deadline {
        let states = sqlx::query_as::<_, (i64, i64)>(
            "SELECT s.user_id, s.charged_bytes FROM notification_user_state s
             WHERE NOT EXISTS (
                 SELECT 1 FROM notification_recipients nr WHERE nr.user_id = s.user_id
             ) ORDER BY s.user_id LIMIT ?",
        )
        .bind(limits.rows)
        .fetch_all(&mut *connection)
        .await?;
        for (user_id, charge) in states {
            if Instant::now() >= deadline || result.charged_bytes + charge > limits.charged_bytes {
                break;
            }
            let deleted = sqlx::query("DELETE FROM notification_user_state WHERE user_id = ?")
                .bind(user_id)
                .execute(&mut *connection)
                .await?
                .rows_affected();
            result.states += deleted;
            result.charged_bytes += charge * deleted as i64;
        }
    }
    Ok(result)
}
