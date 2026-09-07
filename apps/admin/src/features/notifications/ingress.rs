use super::{admission::AdmissionService, admission_types::*};
use chrono::{DateTime, Utc};
use rustzen_ipc::{NotificationEvent, NotificationHeaders, verify_notification};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub(super) const MAX_BODY: usize = 16 * 1024;

#[derive(Clone)]
pub(crate) struct IngressState {
    admission: Arc<AdmissionService>,
    keys: Arc<HashMap<String, SigningKey>>,
    guard: Arc<IngressGuard>,
}

struct SigningKey {
    secret: Vec<u8>,
    previous_until: Option<i64>,
}

#[derive(Default)]
struct GuardState {
    nonces: HashMap<(String, String), i64>,
    producer_rate: HashMap<String, RateBucket>,
    aggregate_rate: RateBucket,
}

#[derive(Default)]
struct RateBucket {
    window_start: i64,
    used: usize,
}

#[derive(Default)]
struct IngressGuard(Mutex<GuardState>);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum IngestOutcome {
    Stored,
    Duplicate,
    NoRecipients,
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub(crate) enum IngestError {
    #[error("bad request")]
    BadRequest,
    #[error("unauthorized producer")]
    Unauthorized,
    #[error("forbidden topic")]
    Forbidden,
    #[error("event expired")]
    Expired,
    #[error("rate or nonce capacity")]
    RateLimited,
    #[error("event conflict")]
    Conflict,
    #[error("invalid event")]
    Unprocessable,
    #[error("notification capacity")]
    Capacity,
    #[error("internal ingest failure")]
    Internal,
}

impl IngressState {
    pub(crate) async fn new(
        pool: SqlitePool,
        database_path: std::path::PathBuf,
        policy: AdmissionPolicy,
        key_id: String,
        secret: Vec<u8>,
        previous: Option<(String, Vec<u8>, i64)>,
    ) -> Result<Self, AdmissionError> {
        Self::new_at(pool, database_path, policy, key_id, secret, previous, Utc::now()).await
    }

    async fn new_at(
        pool: SqlitePool,
        database_path: std::path::PathBuf,
        policy: AdmissionPolicy,
        key_id: String,
        secret: Vec<u8>,
        previous: Option<(String, Vec<u8>, i64)>,
        started_at: DateTime<Utc>,
    ) -> Result<Self, AdmissionError> {
        let mut keys = HashMap::from([(key_id, SigningKey { secret, previous_until: None })]);
        if let Some((key_id, secret, expires)) = previous {
            if expires <= started_at.timestamp() || expires > started_at.timestamp() + 120 {
                return Err(AdmissionError::Invalid);
            }
            keys.insert(key_id, SigningKey { secret, previous_until: Some(expires) });
        }
        Ok(Self {
            admission: Arc::new(AdmissionService::start(pool, database_path, policy).await?),
            keys: Arc::new(keys),
            guard: Arc::default(),
        })
    }

    pub(crate) async fn ingest(
        &self,
        headers: NotificationHeaders,
        body: &[u8],
        now: DateTime<Utc>,
    ) -> Result<IngestOutcome, IngestError> {
        if body.len() > MAX_BODY {
            return Err(IngestError::BadRequest);
        }
        let key = self.keys.get(&headers.key_id).ok_or(IngestError::Unauthorized)?;
        if key.previous_until.is_some_and(|until| now.timestamp() >= until) {
            return Err(IngestError::Unauthorized);
        }
        verify_notification(&headers, body, &headers.key_id, &key.secret)
            .map_err(|_| IngestError::Unauthorized)?;
        let now_seconds = now.timestamp();
        if headers.producer != "monitor" {
            return Err(IngestError::Unauthorized);
        }
        if headers.expires - headers.created > 60 || headers.created > now_seconds + 60 {
            return Err(IngestError::BadRequest);
        }
        if headers.expires <= now_seconds {
            return Err(IngestError::Expired);
        }
        self.guard.accept(&headers.producer, &headers.nonce, headers.expires, now_seconds)?;
        let event: NotificationEvent =
            serde_json::from_slice(body).map_err(|_| IngestError::Unprocessable)?;
        validate_event(&event, &headers.producer)?;
        let occurred_at = DateTime::parse_from_rfc3339(&event.occurred_at)
            .map_err(|_| IngestError::Unprocessable)?
            .naive_utc();
        let expires_at = DateTime::parse_from_rfc3339(&event.expires_at)
            .map_err(|_| IngestError::Unprocessable)?
            .naive_utc();
        if occurred_at > (now + chrono::Duration::seconds(60)).naive_utc()
            || expires_at <= occurred_at
            || expires_at > occurred_at + chrono::Duration::days(7)
        {
            return Err(IngestError::Unprocessable);
        }
        let input = AdmissionEvent {
            producer: event.producer,
            event_id: event.event_id,
            payload_sha256: hex::encode(Sha256::digest(body)),
            expires_at,
            topic: event.topic,
            subject_kind: event.subject.kind,
            subject_id: event.subject.id,
            subject_revision: event.subject.revision,
            occurred_at,
            title: event.content.title,
            summary: event.content.summary,
            required_capability: "monitor:incident:view".into(),
            candidate_user_ids: Vec::new(),
        };
        match self.admission.admit_current_monitor_audience(&input, now.naive_utc()).await {
            Ok(AdmissionResult::Stored { .. }) => Ok(IngestOutcome::Stored),
            Ok(AdmissionResult::Duplicate { .. }) => Ok(IngestOutcome::Duplicate),
            Ok(AdmissionResult::NoRecipients) => Ok(IngestOutcome::NoRecipients),
            Err(AdmissionError::Conflict) => Err(IngestError::Conflict),
            Err(AdmissionError::Expired) => Err(IngestError::Expired),
            Err(AdmissionError::Invalid | AdmissionError::AudienceTooLarge) => {
                Err(IngestError::Unprocessable)
            }
            Err(AdmissionError::Capacity { .. }) => Err(IngestError::Capacity),
            Err(_) => Err(IngestError::Internal),
        }
    }
}

impl IngressGuard {
    fn accept(
        &self,
        producer: &str,
        nonce: &str,
        _expires: i64,
        now: i64,
    ) -> Result<(), IngestError> {
        let mut state = self.0.lock().expect("notification ingress guard");
        state.nonces.retain(|_, until| *until > now);
        if state.nonces.contains_key(&(producer.into(), nonce.into())) {
            return Err(IngestError::Unauthorized);
        }
        let nonce_count = state.nonces.len();
        let producer_nonce_count =
            state.nonces.keys().filter(|(owner, _)| owner == producer).count();
        let producer_allowed =
            state.producer_rate.entry(producer.into()).or_default().accept(now, 100);
        if !producer_allowed
            || !state.aggregate_rate.accept(now, 150)
            || nonce_count >= 30_000
            || producer_nonce_count >= 20_000
        {
            return Err(IngestError::RateLimited);
        }
        state.nonces.insert((producer.into(), nonce.into()), now + 120);
        Ok(())
    }
}

impl RateBucket {
    fn accept(&mut self, now: i64, limit: usize) -> bool {
        if self.window_start != now {
            self.window_start = now;
            self.used = 0;
        }
        if self.used >= limit {
            return false;
        }
        self.used += 1;
        true
    }
}

fn validate_event(event: &NotificationEvent, producer: &str) -> Result<(), IngestError> {
    if event.schema_version != 1
        || event.producer != producer
        || !matches!(event.topic.as_str(), "monitor.incident.opened" | "monitor.incident.resolved")
        || event.subject.kind != "monitor-incident"
        || event.audience.policy != "monitor-incident-readers"
        || event.subject.revision != if event.topic.ends_with("opened") { 1 } else { 2 }
    {
        return Err(IngestError::Forbidden);
    }
    Ok(())
}

pub(crate) use super::ingress_http::start;

#[cfg(test)]
mod rate_tests {
    use super::{IngestError, IngressGuard};

    #[test]
    fn fixed_window_limits_are_sustained_across_seconds() {
        let guard = IngressGuard::default();
        for second in [10, 11] {
            for index in 0..100 {
                assert!(
                    guard
                        .accept("monitor", &format!("{second}-{index}"), second + 60, second)
                        .is_ok()
                );
            }
            assert_eq!(
                guard.accept("monitor", &format!("{second}-overflow"), second + 60, second),
                Err(IngestError::RateLimited)
            );
        }

        let aggregate = IngressGuard::default();
        for index in 0..100 {
            aggregate.accept("monitor", &format!("a-{index}"), 80, 20).unwrap();
        }
        for index in 0..50 {
            aggregate.accept("reports", &format!("b-{index}"), 80, 20).unwrap();
        }
        assert_eq!(
            aggregate.accept("reports", "aggregate-overflow", 80, 20),
            Err(IngestError::RateLimited)
        );
    }
}
