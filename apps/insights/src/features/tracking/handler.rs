use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{
    app::AppState,
    common::{api::ApiResponse, error::AppError},
};

use super::service::TrackingService;

const ALLOWED_METHODS: &str = "POST";
const ALLOWED_HEADERS: &str = "content-type, x-rustzen-project-key";

pub async fn track(State(state): State<AppState>, request: Request<Body>) -> Response {
    let (parts, body) = request.into_parts();
    let outcome = TrackingService::track(&state.pool, &state.ingestion, &parts.headers, body).await;
    let mut response = match outcome.result {
        Ok(result) => ApiResponse::success(result).into_response(),
        Err(error) => error.into_response(),
    };
    apply_origin_cors(&mut response, outcome.cors_origin.as_deref());
    response
}

pub async fn preflight(State(state): State<AppState>, request: Request<Body>) -> Response {
    if !preflight_request_is_supported(request.headers()) {
        let mut response = AppError::input_rejection(
            StatusCode::FORBIDDEN,
            "CORS preflight request is not supported",
        )
        .into_response();
        apply_origin_cors(&mut response, None);
        return response;
    }

    match TrackingService::preflight_origin(&state.pool, &state.ingestion, request.headers()).await
    {
        Ok(origin) => {
            let mut response = StatusCode::NO_CONTENT.into_response();
            apply_preflight_cors(&mut response, &origin);
            response
        }
        Err(error) => {
            let mut response = error.into_response();
            apply_origin_cors(&mut response, None);
            response
        }
    }
}

pub async fn tracker() -> Response {
    const SCRIPT: &str = include_str!("tracker.js");
    let mut response = SCRIPT.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    response
}

fn preflight_request_is_supported(headers: &HeaderMap) -> bool {
    let Some(method) = headers
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    else {
        return false;
    };
    if !method.eq_ignore_ascii_case("POST") {
        return false;
    }
    headers
        .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
        .map(|value| {
            value.to_str().ok().is_some_and(|value| {
                value.split(',').map(str::trim).filter(|name| !name.is_empty()).all(|name| {
                    matches!(
                        name.to_ascii_lowercase().as_str(),
                        "content-type" | "x-rustzen-project-key"
                    )
                })
            })
        })
        .unwrap_or(true)
}

fn apply_origin_cors(response: &mut Response, origin: Option<&str>) {
    response.headers_mut().insert(header::VARY, HeaderValue::from_static("Origin"));
    let Some(origin) = origin else { return };
    let Ok(origin) = HeaderValue::try_from(origin) else { return };
    response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
}

fn apply_preflight_cors(response: &mut Response, origin: &str) {
    apply_origin_cors(response, Some(origin));
    response
        .headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(ALLOWED_METHODS));
    response
        .headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(ALLOWED_HEADERS));
}
