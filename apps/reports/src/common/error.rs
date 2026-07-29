use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("run cancelled")]
    Cancelled,
    #[error("reports database operation failed")]
    Database,
    #[error("reports service operation failed")]
    Internal,
}

impl AppError {
    pub fn database(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "Reports database operation failed");
        Self::Database
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "Reports service operation failed");
        Self::Internal
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::database(error)
    }
}
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::internal(error)
    }
}
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::internal(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Cancelled => (StatusCode::CONFLICT, "run cancelled".to_string()),
            Self::Database | Self::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "reports worker error".to_string())
            }
        };
        (
            status,
            Json(serde_json::json!({
                "code": 40002,
                "message": message,
                "data": null,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
    use serde_json::{Value, json};

    use super::AppError;

    async fn response(error: AppError) -> (StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.expect("read error body");
        (status, serde_json::from_slice(&body).expect("parse error body"))
    }

    #[tokio::test]
    async fn conflict_and_internal_errors_keep_the_existing_gateway_contract() {
        assert_eq!(
            response(AppError::Conflict("report is not ready".to_string())).await,
            (
                StatusCode::CONFLICT,
                json!({ "code": 40002, "message": "report is not ready", "data": null })
            )
        );
        assert_eq!(
            response(AppError::Internal).await,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "code": 40002, "message": "reports worker error", "data": null })
            )
        );
    }
}
