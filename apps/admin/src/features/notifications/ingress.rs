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
    keys: Arc<HashMap<(String, String), SigningKey>>,
    guard: Arc<IngressGuard>,
}

struct SigningKey {
    secret: Vec<u8>,
    previous_until: Option<i64>,
}

pub(crate) struct ProducerKeys {
    pub producer: &'static str,
    pub current_id: String,
    pub current_secret: Vec<u8>,
    pub previous: Option<(String, Vec<u8>, i64)>,
}

#[derive(Clone, Copy)]
enum AudienceKind {
    CurrentMonitor,
    #[cfg(feature = "reports-notifications")]
    Initiator(i64),
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
    #[cfg(test)]
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

    #[cfg(test)]
    async fn new_at(
        pool: SqlitePool,
        database_path: std::path::PathBuf,
        policy: AdmissionPolicy,
        key_id: String,
        secret: Vec<u8>,
        previous: Option<(String, Vec<u8>, i64)>,
        started_at: DateTime<Utc>,
    ) -> Result<Self, AdmissionError> {
        Self::new_with_keys_at(
            pool,
            database_path,
            policy,
            vec![ProducerKeys {
                producer: "monitor",
                current_id: key_id,
                current_secret: secret,
                previous,
            }],
            started_at,
        )
        .await
    }

    pub(crate) async fn new_with_keys(
        pool: SqlitePool,
        database_path: std::path::PathBuf,
        policy: AdmissionPolicy,
        keys: Vec<ProducerKeys>,
    ) -> Result<Self, AdmissionError> {
        Self::new_with_keys_at(pool, database_path, policy, keys, Utc::now()).await
    }

    async fn new_with_keys_at(
        pool: SqlitePool,
        database_path: std::path::PathBuf,
        policy: AdmissionPolicy,
        configured: Vec<ProducerKeys>,
        started_at: DateTime<Utc>,
    ) -> Result<Self, AdmissionError> {
        let mut keys = HashMap::new();
        for producer in configured {
            let current = (producer.producer.to_string(), producer.current_id);
            if keys
                .insert(
                    current,
                    SigningKey { secret: producer.current_secret, previous_until: None },
                )
                .is_some()
            {
                return Err(AdmissionError::Invalid);
            }
            if let Some((key_id, secret, expires)) = producer.previous {
                if expires <= started_at.timestamp() || expires > started_at.timestamp() + 120 {
                    return Err(AdmissionError::Invalid);
                }
                if keys
                    .insert(
                        (producer.producer.to_string(), key_id),
                        SigningKey { secret, previous_until: Some(expires) },
                    )
                    .is_some()
                {
                    return Err(AdmissionError::Invalid);
                }
            }
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
        let key = self
            .keys
            .get(&(headers.producer.clone(), headers.key_id.clone()))
            .ok_or(IngestError::Unauthorized)?;
        if key.previous_until.is_some_and(|until| now.timestamp() >= until) {
            return Err(IngestError::Unauthorized);
        }
        verify_notification(&headers, body, &headers.key_id, &key.secret)
            .map_err(|_| IngestError::Unauthorized)?;
        let now_seconds = now.timestamp();
        if headers.expires - headers.created > 60 || headers.created > now_seconds + 60 {
            return Err(IngestError::BadRequest);
        }
        if headers.expires <= now_seconds {
            return Err(IngestError::Expired);
        }
        self.guard.accept(&headers.producer, &headers.nonce, headers.expires, now_seconds)?;
        let event: NotificationEvent =
            serde_json::from_slice(body).map_err(|_| IngestError::Unprocessable)?;
        let audience = validate_event(&event, &headers.producer)?;
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
        let (required_capability, candidate_user_ids) = match audience {
            AudienceKind::CurrentMonitor => ("monitor:incident:view", Vec::new()),
            #[cfg(feature = "reports-notifications")]
            AudienceKind::Initiator(user_id) => ("reports:run:view", vec![user_id]),
        };
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
            required_capability: required_capability.into(),
            candidate_user_ids,
        };
        let result = match audience {
            AudienceKind::CurrentMonitor => {
                self.admission.admit_current_monitor_audience(&input, now.naive_utc()).await
            }
            #[cfg(feature = "reports-notifications")]
            AudienceKind::Initiator(_) => self.admission.admit(&input, now.naive_utc()).await,
        };
        match result {
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

fn validate_event(event: &NotificationEvent, producer: &str) -> Result<AudienceKind, IngestError> {
    if event.schema_version != 1 || event.producer != producer {
        return Err(IngestError::Forbidden);
    }
    match producer {
        "monitor"
            if matches!(
                event.topic.as_str(),
                "monitor.incident.opened" | "monitor.incident.resolved"
            ) && event.subject.kind == "monitor-incident"
                && event.audience.policy == "monitor-incident-readers"
                && event.audience.initiator_user_id.is_none()
                && event.subject.revision
                    == if event.topic.ends_with("opened") { 1 } else { 2 } =>
        {
            Ok(AudienceKind::CurrentMonitor)
        }
        #[cfg(feature = "reports-notifications")]
        "reports"
            if matches!(
                event.topic.as_str(),
                "reports.run.completed" | "reports.run.failed" | "reports.run.cancelled"
            ) && event.subject.kind == "reports-run"
                && event.audience.policy == "reports-run-initiator"
                && event.subject.revision == 1
                && event.audience.initiator_user_id.is_some_and(|id| id > 0) =>
        {
            Ok(AudienceKind::Initiator(
                event.audience.initiator_user_id.expect("validated initiator"),
            ))
        }
        _ => Err(IngestError::Forbidden),
    }
}

pub(crate) use super::ingress_http::start_with_keys;

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
