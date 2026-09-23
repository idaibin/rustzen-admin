use axum::http::{HeaderMap, StatusCode, header};
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::Value;

use crate::common::error::AppError;

use super::{
    super::types::{NewEvent, TrackInput},
    normalize_origin,
};

const MAX_FUTURE_CLOCK_SKEW: ChronoDuration = ChronoDuration::minutes(5);
const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1000;
const REGISTERED_EVENTS: &[&str] = &["page_view", "api_request", "custom_export"];
const REGISTERED_PROPERTY_KEYS: &[&str] = &["feature", "format", "result", "status"];

pub(super) fn header_text(headers: &HeaderMap, name: &str) -> Result<Option<String>, AppError> {
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

pub(super) fn request_origin(headers: &HeaderMap) -> Result<String, AppError> {
    let origin = header_text(headers, header::ORIGIN.as_str())?
        .ok_or_else(|| AppError::input_rejection(StatusCode::FORBIDDEN, "origin is required"))?;
    normalize_origin(&origin)
        .map_err(|_| AppError::input_rejection(StatusCode::FORBIDDEN, "origin is invalid"))
}

pub(super) fn origin_allowed(raw_origins: &str, origin: &str) -> Result<bool, AppError> {
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

pub(super) fn validate_event(
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
