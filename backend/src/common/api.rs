use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

// --- API Response Structures ---

/// A unified structure for successful API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Business status code. 0 for success.
    pub code: i32,
    /// Response message.
    pub message: String,
    /// Response data.
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a success response.
    pub fn success(data: T) -> Json<Self> {
        Json(Self { code: 0, message: "Success".to_string(), data })
    }
}

/// A generic structure for dropdown options.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionItem<T> {
    pub label: String,
    pub value: T,
}

/// Query parameters for options endpoints
#[derive(Debug, Deserialize)]
pub struct OptionsQuery {
    pub status: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
}

/// Query parameters for dict options endpoints
#[derive(Debug, Deserialize)]
pub struct DictOptionsQuery {
    pub dict_type: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
}

// --- Business Service Errors ---

/// A unified error type for the business logic layer.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// A database query failed.
    #[error("Database query failed")]
    DatabaseQueryFailed,

    /// The requested resource was not found.
    #[error("{0} not found")]
    NotFound(String),

    /// The user's credentials were invalid.
    #[error("Invalid username or password")]
    InvalidCredentials,

    /// The provided JWT was invalid or expired.
    #[error("Invalid or expired token")]
    InvalidToken,

    /// The user does not have permission to perform this action.
    #[error("Permission denied")]
    PermissionDenied,

    /// A username that was provided already exists.
    #[error("Username already exists")]
    UsernameConflict,

    /// An email that was provided already exists.
    #[error("Email already exists")]
    EmailConflict,

    /// A role name that was provided already exists.
    #[error("Role name already exists")]
    RoleNameConflict,

    /// A menu title that was provided already exists.
    #[error("Menu title already exists")]
    MenuTitleConflict,

    /// An operation was attempted that is invalid given the current state.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Password hashing failed.
    #[error("Password hashing failed")]
    PasswordHashingFailed,

    /// User is disabled.
    #[error("User is disabled")]
    UserIsDisabled,
}

// --- Axum Error Handling ---

/// A type alias for application-level results.
pub type AppResult<T> = Result<T, AppError>;

/// A unified error type for the application layer, which can be converted into an HTTP response.
#[derive(Debug)]
pub struct AppError((StatusCode, i32, String));

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.0;
        let body = Json(serde_json::json!({
            "code": code,
            "message": message,
            "data": null,
        }));
        (status, body).into_response()
    }
}

/// Converts a `ServiceError` into an `AppError`.
/// This is the central place to map business logic errors to HTTP-level errors.
impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        let (status, code, message) = match err {
            // System-level Errors (2xxxx)
            ServiceError::DatabaseQueryFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001, // System-Common-01
                "A database query failed.".to_string(),
            ),

            // Business-level Errors (1xxxx)
            ServiceError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                10001, // Business-Common-01
                format!("{} not found.", resource),
            ),
            ServiceError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                10101, // Business-Auth-01
                "Invalid username or password.".to_string(),
            ),
            ServiceError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                10102, // Business-Auth-02
                "Invalid or expired token.".to_string(),
            ),
            ServiceError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                10103, // Business-Auth-03
                "Permission denied.".to_string(),
            ),
            ServiceError::UsernameConflict => (
                StatusCode::CONFLICT,
                10201, // Business-User-01
                "Username already exists.".to_string(),
            ),
            ServiceError::EmailConflict => (
                StatusCode::CONFLICT,
                10202, // Business-User-02
                "Email already exists.".to_string(),
            ),
            ServiceError::RoleNameConflict => (
                StatusCode::CONFLICT,
                10301, // Business-Role-01
                "Role name already exists.".to_string(),
            ),
            ServiceError::MenuTitleConflict => (
                StatusCode::CONFLICT,
                10401, // Business-Menu-01
                "Menu title already exists.".to_string(),
            ),
            ServiceError::InvalidOperation(reason) => (
                StatusCode::BAD_REQUEST,
                10002, // Business-Common-02
                reason,
            ),
            ServiceError::PasswordHashingFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, 10003, "Password processing failed".into())
            }
            ServiceError::UserIsDisabled => {
                (StatusCode::FORBIDDEN, 10004, "User account is disabled".into())
            }
        };
        AppError((status, code, message))
    }
}

/// Allows `sqlx::Error` to be converted into `AppError` for convenience in route handlers.
/// This should be used sparingly, prefer mapping to `ServiceError` in the service layer.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        let service_error = ServiceError::DatabaseQueryFailed;
        service_error.into()
    }
}
