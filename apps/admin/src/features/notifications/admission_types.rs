use chrono::NaiveDateTime;
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct AdmissionEvent {
    pub producer: String,
    pub event_id: String,
    pub payload_sha256: String,
    pub expires_at: NaiveDateTime,
    pub topic: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub subject_revision: i64,
    pub occurred_at: NaiveDateTime,
    pub title: String,
    pub summary: String,
    pub required_capability: String,
    pub candidate_user_ids: Vec<i64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AdmissionPolicy {
    pub message_limit: i64,
    pub recipient_limit: i64,
    pub receipt_limit: i64,
    pub charged_bytes_limit: i64,
    pub free_space_reserve_bytes: u64,
    pub wal_pressure_frames: u32,
    pub wal_pressure_observations: u32,
    pub cleanup_time_limit: Duration,
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            message_limit: 100_000,
            recipient_limit: 1_000_000,
            receipt_limit: 1_000_000,
            charged_bytes_limit: 512 * 1024 * 1024,
            free_space_reserve_bytes: 128 * 1024 * 1024,
            wal_pressure_frames: 1024,
            wal_pressure_observations: 3,
            cleanup_time_limit: Duration::from_millis(50),
        }
    }
}

impl AdmissionPolicy {
    pub(crate) fn from_config(
        config: &rustzen_config::AdminConfig,
    ) -> Result<Self, AdmissionError> {
        let (messages, recipients, receipts, charged_bytes) = config.notification_limits();
        let (reserve, frames, observations) = config.notification_pressure_limits();
        Ok(Self {
            message_limit: i64::try_from(messages).map_err(|_| AdmissionError::Invalid)?,
            recipient_limit: i64::try_from(recipients).map_err(|_| AdmissionError::Invalid)?,
            receipt_limit: i64::try_from(receipts).map_err(|_| AdmissionError::Invalid)?,
            charged_bytes_limit: i64::try_from(charged_bytes)
                .map_err(|_| AdmissionError::Invalid)?,
            free_space_reserve_bytes: reserve,
            wal_pressure_frames: frames,
            wal_pressure_observations: observations,
            cleanup_time_limit: Duration::from_millis(50),
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CapacityReason {
    Messages,
    Recipients,
    Receipts,
    ChargedBytes,
    FreeSpace,
    Checkpoint,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AdmissionError {
    #[error("invalid notification event")]
    Invalid,
    #[error("notification event has expired")]
    Expired,
    #[error("event ID is already bound to different content")]
    Conflict,
    #[error("eligible audience exceeds 1000 recipients")]
    AudienceTooLarge,
    #[error("notification-capacity: {reason:?}")]
    Capacity { reason: CapacityReason, retry_after_seconds: u64 },
    #[error("notification accounting validation failed")]
    Accounting,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum AdmissionResult {
    Stored { notification_id: String, recipients: usize },
    NoRecipients,
    Duplicate { result: String },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct AdmissionStatus {
    pub accepting: bool,
    pub reason: Option<CapacityReason>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum Failpoint {
    AfterReceipt,
    AfterMessage,
    DuringRecipients,
    AfterRecipients,
}
