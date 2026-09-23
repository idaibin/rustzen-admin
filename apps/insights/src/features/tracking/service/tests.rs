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
    assert_eq!(normalize_origin("http://app.example:80/"), Ok("http://app.example".to_string()));
    assert_eq!(normalize_origin("https://app.example:443/"), Ok("https://app.example".to_string()));
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
    let budget_error =
        check_storage_capacity(STORAGE_BUDGET_BYTES, u64::MAX, 1).expect_err("budget rejection");
    assert_eq!(budget_error.into_response().status(), StatusCode::INSUFFICIENT_STORAGE);

    let disk_error = check_storage_capacity(0, FREE_DISK_RESERVE_BYTES, 1)
        .expect_err("free disk reserve rejection");
    assert_eq!(disk_error.into_response().status(), StatusCode::INSUFFICIENT_STORAGE);

    assert!(check_storage_capacity(0, FREE_DISK_RESERVE_BYTES + 1, 1).is_ok());
}
