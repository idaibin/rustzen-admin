use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    code: i32,
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl AppError {
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self { status: StatusCode::UNPROCESSABLE_ENTITY, code: 40002, message: message.into() }
    }

    pub fn not_found(resource: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: 40002,
            message: format!("{resource} not found"),
        }
    }

    pub fn invalid_agent_token() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: 30001,
            message: "Invalid monitor agent token.".to_string(),
        }
    }

    pub(crate) fn database() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: 40002,
            message: "monitor worker error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "code": self.code,
            "message": self.message,
            "data": null,
        }));
        (self.status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "Monitor database operation failed");
        Self::database()
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    use super::AppError;

    async fn response(error: AppError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.expect("error body");
        (status, serde_json::from_slice(&body).expect("error JSON"))
    }

    #[tokio::test]
    async fn module_errors_preserve_the_existing_admin_proxy_contract() {
        let (status, body) = response(AppError::not_found("node")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            serde_json::json!({ "code": 40002, "message": "node not found", "data": null })
        );

        let (status, body) = response(AppError::database()).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            body,
            serde_json::json!({ "code": 40002, "message": "monitor worker error", "data": null })
        );
    }

    #[tokio::test]
    async fn agent_token_error_keeps_its_dedicated_contract() {
        let (status, body) = response(AppError::invalid_agent_token()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            body,
            serde_json::json!({
                "code": 30001,
                "message": "Invalid monitor agent token.",
                "data": null
            })
        );
    }
}
