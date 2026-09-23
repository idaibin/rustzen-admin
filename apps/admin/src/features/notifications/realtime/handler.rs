use super::{OpenError, RealtimeHub};
use axum::{
    Extension,
    extract::OriginalUri,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response, Sse},
};
use futures::stream;
use rustzen_auth::auth::AuthClaims;
use serde_json::json;

const RETRY_AFTER: &str = "60";

pub(crate) async fn stream(
    Extension(claims): Extension<AuthClaims>,
    Extension(realtime): Extension<RealtimeHub>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    if uri.query().is_some() {
        return error(OpenError::BadRequest);
    }
    let _continuity_hint = headers.get("last-event-id").and_then(|value| value.to_str().ok());
    match realtime.subscribe(claims).await {
        Ok(stream_state) => {
            let body = stream::unfold(stream_state, |mut state| async move {
                state.next_event().await.map(|event| (event, state))
            });
            let mut response = Sse::new(body).into_response();
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            );
            response.headers_mut().insert("x-accel-buffering", HeaderValue::from_static("no"));
            response
        }
        Err(failure) => error(failure),
    }
}

fn error(error: OpenError) -> Response {
    let (status, code, message, retry) = match error {
        OpenError::BadRequest => {
            (StatusCode::BAD_REQUEST, 400, "SSE query parameters are forbidden", false)
        }
        OpenError::Unauthorized => {
            (StatusCode::UNAUTHORIZED, 401, "Invalid or expired token", false)
        }
        OpenError::Forbidden => {
            (StatusCode::FORBIDDEN, 403, "Notification stream unavailable", false)
        }
        OpenError::Draining => return no_content(),
        OpenError::UserLimit => {
            (StatusCode::TOO_MANY_REQUESTS, 42901, "Notification stream user limit", true)
        }
        OpenError::GlobalLimit => {
            (StatusCode::SERVICE_UNAVAILABLE, 50301, "Notification stream capacity", true)
        }
        OpenError::Unavailable => {
            (StatusCode::SERVICE_UNAVAILABLE, 50302, "Notification authority unavailable", true)
        }
    };
    let mut response =
        (status, axum::Json(json!({"code":code,"message":message,"data":null}))).into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if retry {
        response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER));
    }
    response
}

fn no_content() -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
