use super::{accounting::Accounting, admission_types::*};
use chrono::NaiveDateTime;
use sqlx::SqliteConnection;

pub(super) fn validate_key(event: &AdmissionEvent) -> Result<(), AdmissionError> {
    if !matches!(event.producer.as_str(), "monitor" | "reports")
        || !(1..=128).contains(&event.event_id.len())
        || event.payload_sha256.len() != 64
        || !event
            .payload_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(AdmissionError::Invalid);
    }
    Ok(())
}

pub(super) fn validate_new(
    event: &AdmissionEvent,
    accepted_at: NaiveDateTime,
) -> Result<(), AdmissionError> {
    if event.expires_at <= accepted_at {
        return Err(AdmissionError::Expired);
    }
    let bounded = [
        (&event.topic, 1, 128),
        (&event.subject_kind, 1, 64),
        (&event.subject_id, 1, 256),
        (&event.title, 1, 256),
        (&event.summary, 0, 1024),
        (&event.required_capability, 1, 128),
    ];
    if event.subject_revision <= 0
        || event.candidate_user_ids.len() > 10_000
        || event.candidate_user_ids.iter().any(|id| *id <= 0)
        || bounded.iter().any(|(value, min, max)| value.len() < *min || value.len() > *max)
    {
        return Err(AdmissionError::Invalid);
    }
    Ok(())
}

pub(super) async fn duplicate(
    connection: &mut SqliteConnection,
    event: &AdmissionEvent,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT payload_sha256, result FROM notification_receipts
         WHERE producer = ? AND event_id = ?",
    )
    .bind(&event.producer)
    .bind(&event.event_id)
    .fetch_optional(connection)
    .await
}

pub(super) async fn eligible(
    connection: &mut SqliteConnection,
    event: &AdmissionEvent,
) -> Result<Vec<i64>, sqlx::Error> {
    let candidates = serde_json::to_string(&event.candidate_user_ids)
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
    sqlx::query_scalar(
        "WITH candidate(user_id) AS (
            SELECT DISTINCT CAST(value AS INTEGER) FROM json_each(?)
        )
        SELECT u.id FROM candidate c
        JOIN users u ON u.id = c.user_id AND u.status = 1 AND u.deleted_at IS NULL
        WHERE EXISTS (SELECT 1 FROM modules m WHERE m.id = ? AND m.enabled = 1)
          AND EXISTS (
            SELECT 1 FROM user_permissions up WHERE up.user_id = u.id
              AND (up.menu_code = '*' OR up.menu_code = ?
                   OR (substr(up.menu_code, -2) = ':*'
                       AND ? LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
          ) ORDER BY u.id LIMIT 1001",
    )
    .bind(candidates)
    .bind(&event.producer)
    .bind(&event.required_capability)
    .bind(&event.required_capability)
    .fetch_all(connection)
    .await
}

pub(super) async fn missing_states(
    connection: &mut SqliteConnection,
    recipients: &[i64],
) -> Result<i64, sqlx::Error> {
    let recipients = serde_json::to_string(recipients)
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM json_each(?) candidate
         LEFT JOIN notification_user_state state
           ON state.user_id = CAST(candidate.value AS INTEGER)
         WHERE state.user_id IS NULL",
    )
    .bind(recipients)
    .fetch_one(connection)
    .await
}

pub(super) async fn insert_message(
    connection: &mut SqliteConnection,
    id: &str,
    event: &AdmissionEvent,
    accepted_at: NaiveDateTime,
    charge: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO notifications
         (id, producer, event_id, topic, subject_kind, subject_id, subject_revision,
          occurred_at, accepted_at, title, summary, required_capability, charged_bytes)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&event.producer)
    .bind(&event.event_id)
    .bind(&event.topic)
    .bind(&event.subject_kind)
    .bind(&event.subject_id)
    .bind(event.subject_revision)
    .bind(event.occurred_at)
    .bind(accepted_at)
    .bind(&event.title)
    .bind(&event.summary)
    .bind(&event.required_capability)
    .bind(charge)
    .execute(connection)
    .await?;
    Ok(())
}

pub(super) async fn accounting(
    connection: &mut SqliteConnection,
) -> Result<Accounting, sqlx::Error> {
    Accounting::current(connection).await
}
