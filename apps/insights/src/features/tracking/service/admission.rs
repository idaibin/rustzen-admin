use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::http::StatusCode;
use tokio::sync::{Mutex as AsyncMutex, Semaphore, TryAcquireError};

use crate::common::error::AppError;

const MAX_RATE_REQUESTS: u32 = 30;
const MAX_RATE_EVENTS: u32 = 300;
const RATE_WINDOW: Duration = Duration::from_secs(60);
pub(super) const INGESTION_CONCURRENCY: usize = 8;

pub(crate) type StorageCapacityChecker =
    Arc<dyn Fn() -> Result<(), AppError> + Send + Sync + 'static>;

#[derive(Debug, Eq, Hash, PartialEq)]
struct RateKey {
    project_id: String,
    origin: String,
}

#[derive(Debug)]
struct RateWindow {
    started_at: Instant,
    requests: u32,
    events: u32,
}

/// Serializes the small ingestion critical section and owns per-source quotas.
pub struct IngestionState {
    rate_windows: Mutex<HashMap<RateKey, RateWindow>>,
    write_guard: AsyncMutex<()>,
    admission: Arc<Semaphore>,
    storage_capacity_checker: Option<StorageCapacityChecker>,
}

impl IngestionState {
    pub fn new() -> Arc<Self> {
        Self::with_storage_capacity_checker(None)
    }

    pub(crate) fn with_storage_capacity_checker(
        storage_capacity_checker: Option<StorageCapacityChecker>,
    ) -> Arc<Self> {
        Arc::new(Self {
            rate_windows: Mutex::new(HashMap::new()),
            write_guard: AsyncMutex::new(()),
            admission: Arc::new(Semaphore::new(INGESTION_CONCURRENCY)),
            storage_capacity_checker,
        })
    }

    pub(super) fn storage_capacity_checker(&self) -> Option<StorageCapacityChecker> {
        self.storage_capacity_checker.clone()
    }

    pub(super) fn reserve_request(&self, project_id: &str, origin: &str) -> Result<(), AppError> {
        let mut windows = self
            .rate_windows
            .lock()
            .map_err(|_| AppError::internal("Insights rate limiter lock poisoned"))?;
        let key = RateKey { project_id: project_id.to_string(), origin: origin.to_string() };
        let now = Instant::now();
        let window =
            windows.entry(key).or_insert(RateWindow { started_at: now, requests: 0, events: 0 });
        if now.duration_since(window.started_at) >= RATE_WINDOW {
            window.started_at = now;
            window.requests = 0;
            window.events = 0;
        }
        if window.requests.saturating_add(1) > MAX_RATE_REQUESTS {
            return Err(AppError::input_rejection(
                StatusCode::TOO_MANY_REQUESTS,
                "collection rate limit exceeded",
            ));
        }
        window.requests += 1;
        Ok(())
    }

    pub(super) fn reserve_events(
        &self,
        project_id: &str,
        origin: &str,
        event_count: usize,
    ) -> Result<(), AppError> {
        let event_count = u32::try_from(event_count).map_err(|_| {
            AppError::input_rejection(StatusCode::PAYLOAD_TOO_LARGE, "batch is too large")
        })?;
        let mut windows = self
            .rate_windows
            .lock()
            .map_err(|_| AppError::internal("Insights rate limiter lock poisoned"))?;
        let key = RateKey { project_id: project_id.to_string(), origin: origin.to_string() };
        let window = windows.get_mut(&key).ok_or_else(|| {
            AppError::internal("Insights request quota was not reserved before event quota")
        })?;
        if window.events.saturating_add(event_count) > MAX_RATE_EVENTS {
            return Err(AppError::input_rejection(
                StatusCode::TOO_MANY_REQUESTS,
                "collection rate limit exceeded",
            ));
        }
        window.events += event_count;
        Ok(())
    }

    pub(super) fn try_acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
        self.admission.clone().try_acquire_owned().map_err(|error| {
            let message = match error {
                TryAcquireError::NoPermits => "Insights ingestion concurrency limit reached",
                TryAcquireError::Closed => "Insights ingestion is shutting down",
            };
            AppError::input_rejection(StatusCode::TOO_MANY_REQUESTS, message)
        })
    }

    pub(crate) async fn policy_guard(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.write_guard.lock().await
    }
}
