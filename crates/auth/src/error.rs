use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Missing auth context")]
    MissingAuthContext,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Authorization authority unavailable")]
    AuthorityUnavailable,
}

impl IntoResponse for CoreError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            CoreError::InvalidToken => (StatusCode::UNAUTHORIZED, 401, "Invalid or expired token"),
            CoreError::MissingAuthContext => {
                (StatusCode::UNAUTHORIZED, 401, "Missing auth context")
            }
            CoreError::PermissionDenied => (StatusCode::FORBIDDEN, 403, "Permission denied"),
            CoreError::AuthorityUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, 50302, "Authorization authority unavailable")
            }
        };

        let mut response =
            (status, Json(CoreErrorResponse { code, message, data: None })).into_response();
        response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        if status == StatusCode::SERVICE_UNAVAILABLE {
            response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        }
        response
    }
}

#[derive(Serialize)]
struct CoreErrorResponse {
    code: i32,
    message: &'static str,
    data: Option<()>,
}
