use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use rustzen_storage::SqlitePool;
use serde_json::Value;

use crate::common::error::AppError;

use super::{
    repo,
    types::{TrackAccepted, TrackInput},
};

mod admission;
mod storage;
mod validation;

#[cfg(test)]
use admission::INGESTION_CONCURRENCY;
pub use admission::IngestionState;
#[cfg(test)]
pub(crate) use admission::StorageCapacityChecker;
use storage::ensure_storage_capacity;
pub use storage::spawn_retention;
#[cfg(test)]
use storage::{FREE_DISK_RESERVE_BYTES, STORAGE_BUDGET_BYTES, check_storage_capacity};
use validation::{header_text, origin_allowed, request_origin, validate_event};

pub const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_BATCH_EVENTS: usize = 50;
const PROJECT_KEY_HEADER: &str = "x-rustzen-project-key";

pub(crate) use crate::features::settings::service::{hash_project_key, normalize_origin};

pub struct TrackingService;

pub(crate) struct TrackOutcome {
    pub result: Result<TrackAccepted, AppError>,
    pub cors_origin: Option<String>,
}

impl TrackOutcome {
    fn rejected(error: AppError) -> Self {
        Self { result: Err(error), cors_origin: None }
    }
}

impl TrackingService {
    pub async fn track(
        pool: &SqlitePool,
        ingestion: &IngestionState,
        headers: &HeaderMap,
        body: Body,
    ) -> TrackOutcome {
        let project_key = match header_text(headers, PROJECT_KEY_HEADER).and_then(|value| {
            value.ok_or_else(|| {
                AppError::input_rejection(StatusCode::FORBIDDEN, "project key is required")
            })
        }) {
            Ok(project_key) => project_key,
            Err(error) => return TrackOutcome::rejected(error),
        };
        if project_key.len() > 256 {
            return TrackOutcome::rejected(AppError::input_rejection(
                StatusCode::FORBIDDEN,
                "project key is invalid",
            ));
        }
        let origin = match request_origin(headers) {
            Ok(origin) => origin,
            Err(error) => return TrackOutcome::rejected(error),
        };
        let _admission = match ingestion.try_acquire() {
            Ok(admission) => admission,
            Err(error) => return TrackOutcome::rejected(error),
        };
        // Buckets are process-local by design: one Insights process owns admission;
        // a restart clears the in-memory quota window and semaphore state.
        let _write_guard = ingestion.policy_guard().await;
        let project_key_hash = match hash_project_key(&project_key) {
            Ok(project_key_hash) => project_key_hash,
            Err(error) => return TrackOutcome::rejected(error),
        };
        let policy = match repo::find_project_policy(pool, &project_key_hash)
            .await
            .map_err(AppError::internal)
        {
            Ok(Some(policy)) => policy,
            Ok(None) => {
                return TrackOutcome::rejected(AppError::input_rejection(
                    StatusCode::FORBIDDEN,
                    "collection policy rejected",
                ));
            }
            Err(error) => return TrackOutcome::rejected(error),
        };
        if !policy.collection_enabled {
            return TrackOutcome::rejected(AppError::input_rejection(
                StatusCode::FORBIDDEN,
                "collection policy rejected",
            ));
        }
        match origin_allowed(&policy.allowed_origins, &origin) {
            Ok(true) => {}
            Ok(false) => {
                return TrackOutcome::rejected(AppError::input_rejection(
                    StatusCode::FORBIDDEN,
                    "collection policy rejected",
                ));
            }
            Err(error) => return TrackOutcome::rejected(error),
        }
        if let Err(error) = ingestion.reserve_request(&policy.project_id, &origin) {
            return TrackOutcome { result: Err(error), cors_origin: Some(origin) };
        }

        let result = async {
            let bytes = to_bytes(body, MAX_BODY_BYTES).await.map_err(|_| {
                AppError::input_rejection(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "request body exceeds 64 KiB",
                )
            })?;
            let body: Value = serde_json::from_slice(&bytes)
                .map_err(|_| AppError::bad_request("event body must be valid JSON"))?;
            let values = match body {
                Value::Array(values) => values,
                value @ Value::Object(_) => vec![value],
                _ => return Err(AppError::bad_request("event body must be an object or array")),
            };
            let inputs = values
                .into_iter()
                .map(serde_json::from_value::<TrackInput>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| {
                    AppError::input_rejection(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        format!(
                            "Failed to deserialize the JSON body into the target type: {error}"
                        ),
                    )
                })?;
            let settings = crate::features::settings::service::get(pool).await?;
            let max_batch_events =
                settings.max_batch_events.clamp(1, MAX_BATCH_EVENTS as i64) as usize;
            if inputs.is_empty() || inputs.len() > max_batch_events {
                return Err(AppError::input_rejection(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("batch must contain 1 to {max_batch_events} events"),
                ));
            }
            ingestion.reserve_events(&policy.project_id, &origin, inputs.len())?;
            let now = Utc::now();
            let events = inputs
                .into_iter()
                .map(|input| validate_event(input, &now, settings.event_retention_days))
                .collect::<Result<Vec<_>, _>>()?;

            // A single writer critical section makes the capacity check and commit one
            // bounded operation; a rejected batch never opens a write transaction.
            if let Some(checker) = ingestion.storage_capacity_checker() {
                checker()?;
            } else {
                ensure_storage_capacity(pool, &events).await?;
            }

            let mut transaction = pool.begin().await.map_err(AppError::internal)?;
            let accepted = events.len();
            for mut event in events {
                event.project_id.clone_from(&policy.project_id);
                repo::insert_event(&mut transaction, &event).await?;
            }
            transaction.commit().await.map_err(AppError::internal)?;
            Ok(TrackAccepted { accepted })
        }
        .await;

        TrackOutcome { result, cors_origin: Some(origin) }
    }

    pub async fn preflight_origin(
        pool: &SqlitePool,
        ingestion: &IngestionState,
        headers: &HeaderMap,
    ) -> Result<String, AppError> {
        let origin = request_origin(headers)?;
        let _write_guard = ingestion.policy_guard().await;
        let policy = crate::features::settings::service::get_collection_policy(pool).await?;
        if !policy.collection_enabled
            || !policy.project_configured
            || !policy.allowed_origins.iter().any(|allowed| allowed == &origin)
        {
            return Err(AppError::input_rejection(
                StatusCode::FORBIDDEN,
                "collection policy rejected",
            ));
        }
        Ok(origin)
    }
}

#[cfg(test)]
mod tests;
