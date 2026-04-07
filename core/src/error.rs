use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Missing auth context")]
    MissingAuthContext,
    #[error("Permission denied")]
    PermissionDenied,
}

impl IntoResponse for CoreError {
    fn into_response(self) -> Response {
        let status = match self {
            CoreError::InvalidToken | CoreError::MissingAuthContext => StatusCode::UNAUTHORIZED,
            CoreError::PermissionDenied => StatusCode::FORBIDDEN,
        };
        status.into_response()
    }
}
