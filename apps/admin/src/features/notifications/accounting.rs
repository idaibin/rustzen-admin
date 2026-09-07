use sqlx::{FromRow, SqliteConnection};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, FromRow)]
pub(crate) struct Accounting {
    pub message_count: i64,
    pub recipient_count: i64,
    pub receipt_count: i64,
    pub charged_bytes: i64,
}

impl Accounting {
    pub async fn current(connection: &mut SqliteConnection) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            "SELECT message_count, recipient_count, receipt_count, charged_bytes
             FROM notification_accounting WHERE id = 1",
        )
        .fetch_one(connection)
        .await
    }

    pub async fn validate(connection: &mut SqliteConnection) -> Result<(), sqlx::Error> {
        let stored = Self::current(connection).await?;
        let actual = sqlx::query_as::<_, Self>(
            "SELECT
                (SELECT COUNT(*) FROM notifications) AS message_count,
                (SELECT COUNT(*) FROM notification_recipients) AS recipient_count,
                (SELECT COUNT(*) FROM notification_receipts) AS receipt_count,
                COALESCE((SELECT SUM(charged_bytes) FROM notifications), 0)
                  + COALESCE((SELECT SUM(charged_bytes) FROM notification_recipients), 0)
                  + COALESCE((SELECT SUM(charged_bytes) FROM notification_receipts), 0)
                  + COALESCE((SELECT SUM(charged_bytes) FROM notification_user_state), 0)
                    AS charged_bytes",
        )
        .fetch_one(connection)
        .await?;
        if stored != actual {
            return Err(sqlx::Error::Protocol(
                "notification accounting differs from durable rows".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn receipt_charge(producer: &str, event_id: &str, digest: &str) -> i64 {
    512.max((producer.len() + event_id.len() + digest.len() + 3 * 24 + 16) as i64)
}

pub(crate) fn message_charge(input: &super::admission_types::AdmissionEvent) -> i64 {
    1024.max(
        (input.producer.len()
            + input.event_id.len()
            + input.topic.len()
            + input.subject_kind.len()
            + input.subject_id.len()
            + input.title.len()
            + input.summary.len()
            + input.required_capability.len()
            + 3 * 24
            + 24) as i64,
    )
}

pub(crate) fn recipient_charge(notification_id: &str) -> i64 {
    256.max((notification_id.len() + 24) as i64)
}

pub(crate) const USER_STATE_CHARGE: i64 = 128;
