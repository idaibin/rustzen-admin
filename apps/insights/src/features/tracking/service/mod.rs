use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, StatusCode, header},
};
use chrono::{Duration as ChronoDuration, Utc};
use rustzen_storage::{SqliteMaintenancePlan, SqlitePool, run_sqlite_maintenance};
use serde_json::Value;
use sqlx::Row;
use sysinfo::Disks;

use crate::common::error::AppError;

use super::{
    repo,
    types::{NewEvent, TrackAccepted, TrackInput},
};

mod admission;

#[cfg(test)]
use admission::INGESTION_CONCURRENCY;
pub use admission::IngestionState;
#[cfg(test)]
pub(crate) use admission::StorageCapacityChecker;

const MAX_FUTURE_CLOCK_SKEW: ChronoDuration = ChronoDuration::minutes(5);
const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1000;
pub const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_BATCH_EVENTS: usize = 50;
const STORAGE_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
const FREE_DISK_RESERVE_BYTES: u64 = 512 * 1024 * 1024;
const STORAGE_PAGE_AMPLIFICATION: u64 = 4;
const STORAGE_EVENT_OVERHEAD_BYTES: u64 = 2048;
const STORAGE_WAL_RESERVE_BYTES: u64 = 64 * 1024;
const REGISTERED_EVENTS: &[&str] = &["page_view", "api_request", "custom_export"];
const REGISTERED_PROPERTY_KEYS: &[&str] = &["feature", "format", "result", "status"];
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

fn header_text(headers: &HeaderMap, name: &str) -> Result<Option<String>, AppError> {
    headers
        .get(name)
        .map(|value| {
            value
                .to_str()
                .map(str::trim)
                .map(str::to_owned)
                .map_err(|_| AppError::input_rejection(StatusCode::FORBIDDEN, "header is invalid"))
        })
        .transpose()
}

fn request_origin(headers: &HeaderMap) -> Result<String, AppError> {
    let origin = header_text(headers, header::ORIGIN.as_str())?
        .ok_or_else(|| AppError::input_rejection(StatusCode::FORBIDDEN, "origin is required"))?;
    normalize_origin(&origin)
        .map_err(|_| AppError::input_rejection(StatusCode::FORBIDDEN, "origin is invalid"))
}

fn origin_allowed(raw_origins: &str, origin: &str) -> Result<bool, AppError> {
    let configured: Vec<String> = serde_json::from_str(raw_origins).map_err(AppError::internal)?;
    configured.into_iter().try_fold(false, |matched, configured| {
        if matched {
            return Ok(true);
        }
        let normalized = normalize_origin(&configured)
            .map_err(|_| AppError::internal("Insights origin policy is invalid"))?;
        Ok(normalized == origin)
    })
}

async fn ensure_storage_capacity(pool: &SqlitePool, events: &[NewEvent]) -> Result<(), AppError> {
    let Some(database_path) = database_path(pool).await? else {
        // In-memory SQLite has no filesystem budget to measure; production paths
        // always resolve to a file and are checked below.
        return Ok(());
    };
    let projected_bytes = events
        .iter()
        .map(estimated_event_bytes)
        .map(|bytes| bytes.saturating_mul(STORAGE_PAGE_AMPLIFICATION))
        .map(|bytes| bytes.saturating_add(STORAGE_EVENT_OVERHEAD_BYTES))
        .sum::<u64>()
        .saturating_add(STORAGE_WAL_RESERVE_BYTES);
    tokio::task::spawn_blocking(move || {
        if filesystem_capacity_ok(&database_path, projected_bytes) {
            Ok(())
        } else {
            Err(AppError::input_rejection(
                StatusCode::INSUFFICIENT_STORAGE,
                "Insights storage protection rejected the batch",
            ))
        }
    })
    .await
    .map_err(AppError::internal)?
}

async fn database_path(pool: &SqlitePool) -> Result<Option<PathBuf>, AppError> {
    let rows =
        sqlx::query("PRAGMA database_list").fetch_all(pool).await.map_err(AppError::internal)?;
    let row = rows
        .into_iter()
        .find(|row| row.try_get::<String, _>("name").ok().is_some_and(|name| name == "main"));
    let Some(row) = row else { return Ok(None) };
    let file = row.try_get::<String, _>("file").map_err(AppError::internal)?;
    if file.is_empty() { Ok(None) } else { Ok(Some(PathBuf::from(file))) }
}

fn estimated_event_bytes(event: &NewEvent) -> u64 {
    (event.event_name.len()
        + event.visitor_id.len()
        + event.user_id.as_deref().map_or(0, str::len)
        + event.session_id.as_deref().map_or(0, str::len)
        + event.platform.as_deref().map_or(0, str::len)
        + event.page_path.as_deref().map_or(0, str::len)
        + event.referrer.as_deref().map_or(0, str::len)
        + event.api_path.as_deref().map_or(0, str::len)
        + event.api_method.as_deref().map_or(0, str::len)
        + event.properties.len()
        + event.occurred_at.len()
        + event.received_at.len()) as u64
        + 256
}

fn filesystem_capacity_ok(database_path: &Path, projected_bytes: u64) -> bool {
    let Ok(main_size) = fs::metadata(database_path).map(|metadata| metadata.len()) else {
        return false;
    };
    let sidecar_size = [
        PathBuf::from(format!("{}-wal", database_path.display())),
        PathBuf::from(format!("{}-shm", database_path.display())),
    ]
    .into_iter()
    .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
    .sum::<u64>();
    let current_bytes = main_size.saturating_add(sidecar_size);
    let Ok(candidate) = database_path.canonicalize() else { return false };
    let disks = Disks::new_with_refreshed_list();
    let Some(disk) = disks
        .list()
        .iter()
        .filter(|disk| candidate.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
    else {
        return false;
    };
    check_storage_capacity(current_bytes, disk.available_space(), projected_bytes).is_ok()
}

fn check_storage_capacity(
    current_bytes: u64,
    available_bytes: u64,
    projected_bytes: u64,
) -> Result<(), AppError> {
    if current_bytes.saturating_add(projected_bytes) > STORAGE_BUDGET_BYTES
        || available_bytes < FREE_DISK_RESERVE_BYTES.saturating_add(projected_bytes)
    {
        return Err(AppError::input_rejection(
            StatusCode::INSUFFICIENT_STORAGE,
            "Insights storage protection rejected the batch",
        ));
    }
    Ok(())
}

pub fn spawn_retention(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            let result = async {
                let settings = crate::features::settings::service::get(&pool).await?;
                let cutoff =
                    (Utc::now() - ChronoDuration::days(settings.event_retention_days)).to_rfc3339();
                cleanup_before(&pool, &cutoff).await
            }
            .await;
            match result {
                Ok(0) => {}
                Ok(deleted) => {
                    tracing::info!(deleted, "Insights retention completed");
                    if let Err(error) =
                        run_sqlite_maintenance(&pool, SqliteMaintenancePlan::reclaim()).await
                    {
                        tracing::error!(%error, "Insights SQLite maintenance failed");
                    }
                }
                Err(error) => tracing::error!(%error, "Insights retention failed"),
            }
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });
}

async fn cleanup_before(pool: &SqlitePool, cutoff: &str) -> Result<u64, AppError> {
    let mut transaction = pool.begin().await.map_err(AppError::internal)?;
    let deleted = repo::delete_events_before(&mut transaction, cutoff).await?;
    transaction.commit().await.map_err(AppError::internal)?;
    Ok(deleted)
}

fn validate_event(
    input: TrackInput,
    received_at: &chrono::DateTime<Utc>,
    retention_days: i64,
) -> Result<NewEvent, AppError> {
    let event_name = input
        .event_name
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty() && value.len() <= 100)
        .ok_or_else(|| AppError::bad_request("eventName is required"))?;
    if !REGISTERED_EVENTS.contains(&event_name.as_str()) {
        return Err(AppError::input_rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            "eventName is not registered",
        ));
    }
    let visitor_id = required(input.visitor_id, "visitorId", 200)?;
    if input.user_id.is_some() {
        return Err(AppError::input_rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            "userId collection is not enabled",
        ));
    }
    validate_properties(&input.properties)?;
    let page_path = clean_path(input.page_path, "pagePath")?;
    let api_path = clean_path(input.api_path, "apiPath")?;
    if event_name == "page_view" && page_path.is_none() {
        return Err(AppError::bad_request("pagePath is required for page_view"));
    }
    if event_name == "api_request" && api_path.is_none() {
        return Err(AppError::bad_request("apiPath is required for api_request"));
    }
    if event_name == "page_view" && api_path.is_some() {
        return Err(AppError::bad_request("page_view must not contain apiPath"));
    }
    if event_name == "api_request" && page_path.is_some() {
        return Err(AppError::bad_request("api_request must not contain pagePath"));
    }
    if input.status_code.is_some_and(|code| !(100..=599).contains(&code)) {
        return Err(AppError::bad_request("statusCode must be between 100 and 599"));
    }
    if input.duration_ms.is_some_and(|duration| duration > MAX_DURATION_MS) {
        return Err(AppError::bad_request("durationMs must not exceed 86400000"));
    }
    let occurred_at = input.occurred_at.unwrap_or(*received_at);
    if occurred_at > *received_at + MAX_FUTURE_CLOCK_SKEW {
        return Err(AppError::bad_request("occurredAt is too far in the future"));
    }
    if occurred_at < *received_at - ChronoDuration::days(retention_days) {
        return Err(AppError::bad_request("occurredAt is outside the retention window"));
    }
    let properties = serde_json::to_string(&input.properties).map_err(AppError::internal)?;
    if properties.len() > 16_384 {
        return Err(AppError::bad_request("properties exceed 16 KiB"));
    }
    Ok(NewEvent {
        project_id: String::new(),
        event_name,
        visitor_id,
        user_id: None,
        session_id: clean_optional(input.session_id, 200)?,
        platform: clean_optional(input.platform, 50)?,
        page_path,
        referrer: clean_referrer(input.referrer)?,
        api_path,
        api_method: clean_optional(input.api_method, 20)?.map(|value| value.to_ascii_uppercase()),
        status_code: input.status_code,
        duration_ms: input.duration_ms.map(|value| value as i64),
        is_error: i64::from(input.is_error),
        properties,
        occurred_at: occurred_at.to_rfc3339(),
        received_at: received_at.to_rfc3339(),
    })
}

fn validate_properties(properties: &Value) -> Result<(), AppError> {
    let Some(properties) = properties.as_object() else {
        return Err(AppError::bad_request("properties must be an object"));
    };
    for (key, value) in properties {
        if !REGISTERED_PROPERTY_KEYS.contains(&key.as_str()) {
            return Err(AppError::input_rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                "property key is not registered",
            ));
        }
        match value {
            Value::String(value) if value.len() <= 256 => {}
            Value::Bool(_) | Value::Number(_) => {}
            Value::String(_) => {
                return Err(AppError::bad_request("property value is too long"));
            }
            _ => return Err(AppError::bad_request("property value must be scalar")),
        }
    }
    Ok(())
}

fn clean_path(value: Option<String>, name: &str) -> Result<Option<String>, AppError> {
    let value = clean_optional(value, 2000)?;
    if value.as_deref().is_some_and(|value| {
        !value.starts_with('/')
            || value.contains(['?', '#', '\n', '\r'])
            || value.bytes().any(|byte| byte.is_ascii_control())
    }) {
        return Err(AppError::input_rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("{name} must be a pathname without query or hash"),
        ));
    }
    Ok(value)
}

fn clean_referrer(value: Option<String>) -> Result<Option<String>, AppError> {
    clean_path(value, "referrer")
}

fn required(value: String, name: &str, max: usize) -> Result<String, AppError> {
    clean_optional(Some(value), max)?
        .ok_or_else(|| AppError::bad_request(format!("{name} is required")))
}

fn clean_optional(value: Option<String>, max: usize) -> Result<Option<String>, AppError> {
    value
        .map(|value| {
            let value = value.trim().to_string();
            if value.len() > max {
                Err(AppError::bad_request("event field is too long"))
            } else if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        })
        .transpose()
        .map(Option::flatten)
}
#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};
    use chrono::{TimeDelta, Utc};
    use serde_json::json;

    use super::{
        FREE_DISK_RESERVE_BYTES, INGESTION_CONCURRENCY, IngestionState, STORAGE_BUDGET_BYTES,
        check_storage_capacity, normalize_origin, validate_event,
    };
    use crate::features::tracking::types::TrackInput;

    fn event() -> TrackInput {
        TrackInput {
            event_name: Some("api_request".to_string()),
            visitor_id: "visitor".to_string(),
            user_id: None,
            session_id: None,
            platform: None,
            page_path: None,
            referrer: None,
            api_path: Some("/api/items".to_string()),
            api_method: Some("GET".to_string()),
            status_code: Some(200),
            duration_ms: Some(40),
            is_error: false,
            properties: json!({}),
            occurred_at: None,
        }
    }

    #[test]
    fn event_time_duration_and_status_code_are_bounded() {
        let now = Utc::now();

        let mut future = event();
        future.occurred_at = Some(now + TimeDelta::minutes(6));
        assert!(validate_event(future, &now, 30).is_err());

        let mut expired = event();
        expired.occurred_at = Some(now - TimeDelta::days(31));
        assert!(validate_event(expired, &now, 30).is_err());

        let mut duration = event();
        duration.duration_ms = Some(86_400_001);
        assert!(validate_event(duration, &now, 30).is_err());

        let mut status = event();
        status.status_code = Some(999);
        assert!(validate_event(status, &now, 30).is_err());
    }

    #[test]
    fn event_properties_and_paths_are_strictly_collection_safe() {
        let now = Utc::now();

        let mut properties = event();
        properties.properties = json!({"text": "copied button label"});
        assert!(validate_event(properties, &now, 30).is_err());

        let mut path = event();
        path.api_path = Some("/api/items?user=secret".to_string());
        assert!(validate_event(path, &now, 30).is_err());
    }

    #[test]
    fn normalizes_browser_origins_without_paths() {
        assert_eq!(normalize_origin("https://app.example"), Ok("https://app.example".to_string()));
        assert_eq!(normalize_origin("HTTPS://APP.EXAMPLE"), Ok("https://app.example".to_string()));
        assert_eq!(
            normalize_origin("http://app.example:80/"),
            Ok("http://app.example".to_string())
        );
        assert_eq!(
            normalize_origin("https://app.example:443/"),
            Ok("https://app.example".to_string())
        );
        assert_eq!(
            normalize_origin("https://app.example:8443"),
            Ok("https://app.example:8443".to_string())
        );
    }

    #[test]
    fn ingestion_admission_is_bounded_and_process_local() {
        let state = IngestionState::new();
        let permits = (0..INGESTION_CONCURRENCY)
            .map(|_| state.try_acquire().expect("admission permit"))
            .collect::<Vec<_>>();
        assert!(state.try_acquire().is_err());

        drop(permits);
        assert!(state.try_acquire().is_ok());
        let restarted = IngestionState::new();
        assert!(restarted.try_acquire().is_ok());
    }

    #[test]
    fn storage_capacity_check_maps_budget_and_disk_limits_to_507() {
        let budget_error = check_storage_capacity(STORAGE_BUDGET_BYTES, u64::MAX, 1)
            .expect_err("budget rejection");
        assert_eq!(budget_error.into_response().status(), StatusCode::INSUFFICIENT_STORAGE);

        let disk_error = check_storage_capacity(0, FREE_DISK_RESERVE_BYTES, 1)
            .expect_err("free disk reserve rejection");
        assert_eq!(disk_error.into_response().status(), StatusCode::INSUFFICIENT_STORAGE);

        assert!(check_storage_capacity(0, FREE_DISK_RESERVE_BYTES + 1, 1).is_ok());
    }
}
