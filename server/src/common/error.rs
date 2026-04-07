use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A unified error type for the business logic layer.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// User is disabled.
    #[error("User is disabled")]
    UserIsDisabled,

    /// User is pending.
    #[error("User is pending")]
    UserIsPending,

    /// User is locked.
    #[error("User is locked")]
    UserIsLocked,

    /// User status is invalid.
    #[error("User status is invalid")]
    InvalidUserStatus,

    /// User is admin.
    #[error("User is admin")]
    UserIsAdmin,

    /// Role is system built-in.
    #[error("Role is system built-in")]
    RoleIsSystem,

    /// Menu is system built-in.
    #[error("Menu is system built-in")]
    MenuIsSystem,

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

    /// Failed to generate token.
    #[error("Failed to generate token")]
    TokenCreationFailed,

    /// A username that was provided already exists.
    #[error("Username already exists")]
    UsernameConflict,

    /// An email that was provided already exists.
    #[error("Email already exists")]
    EmailConflict,

    /// An operation was attempted that is invalid given the current state.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Password hashing failed.
    #[error("Password hashing failed")]
    PasswordHashingFailed,

    /// Failed to upload file.
    #[error("Failed to create avatar folder")]
    CreateAvatarFolderFailed,

    /// Failed to create avatar file.
    #[error("Failed to create avatar file")]
    CreateAvatarFileFailed,
}

/// A unified error type for the application layer, which can be converted into an HTTP response.
#[derive(Debug)]
pub struct AppError((StatusCode, i32, String));

fn app_error(status: StatusCode, code: i32, message: impl Into<String>) -> AppError {
    AppError((status, code, message.into()))
}

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
        match err {
            ServiceError::NotFound(resource) => {
                app_error(StatusCode::NOT_FOUND, 10001, format!("{} not found.", resource))
            }
            ServiceError::InvalidOperation(reason) => {
                app_error(StatusCode::BAD_REQUEST, 10002, reason)
            }
            ServiceError::PasswordHashingFailed => app_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                10003,
                "Password processing failed. Please try again.",
            ),
            ServiceError::UserIsDisabled => {
                app_error(StatusCode::FORBIDDEN, 10004, "User account is disabled.")
            }
            ServiceError::UserIsPending => {
                app_error(StatusCode::BAD_REQUEST, 10005, "User account is pending approval.")
            }
            ServiceError::UserIsLocked => {
                app_error(StatusCode::BAD_REQUEST, 10006, "User account is locked.")
            }
            ServiceError::InvalidUserStatus => {
                app_error(StatusCode::BAD_REQUEST, 10007, "User status is invalid.")
            }
            ServiceError::UserIsAdmin => {
                app_error(StatusCode::BAD_REQUEST, 10008, "Cannot update admin user.")
            }
            ServiceError::RoleIsSystem => {
                app_error(StatusCode::BAD_REQUEST, 10009, "Cannot modify system built-in role.")
            }
            ServiceError::MenuIsSystem => {
                app_error(StatusCode::BAD_REQUEST, 10010, "Cannot modify system built-in menu.")
            }
            ServiceError::InvalidCredentials => {
                app_error(StatusCode::UNAUTHORIZED, 10101, "Invalid username or password.")
            }
            ServiceError::TokenCreationFailed => app_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                10103,
                "Failed to generate login token. Please try again.",
            ),
            ServiceError::UsernameConflict => {
                app_error(StatusCode::CONFLICT, 10201, "Username already exists.")
            }
            ServiceError::EmailConflict => {
                app_error(StatusCode::CONFLICT, 10202, "Email already exists.")
            }
            ServiceError::DatabaseQueryFailed => app_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                20001,
                "Service is temporarily unavailable. Please try again later.",
            ),
            ServiceError::CreateAvatarFolderFailed => app_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                20002,
                "Failed to create avatar folder. Please try again later.",
            ),
            ServiceError::CreateAvatarFileFailed => app_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                20003,
                "Failed to create avatar file. Please try again later.",
            ),
            ServiceError::InvalidToken => app_error(
                StatusCode::UNAUTHORIZED,
                30000,
                "Invalid or expired token. Please log in again.",
            ),
        }
    }
}

/// Allows `sqlx::Error` to be converted into `AppError` for convenience in route handlers.
/// This should be used sparingly, prefer mapping to `ServiceError` in the service layer.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ServiceError::DatabaseQueryFailed.into()
    }
}
