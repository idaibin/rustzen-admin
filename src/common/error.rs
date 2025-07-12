use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

// --- Business Service Errors ---

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

    /// Internal server error.
    // #[error("Internal server error")]
    // InternalServerError,

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

    /// The user does not have permission to perform this action.
    #[error("Permission denied")]
    PermissionDenied,

    /// A username that was provided already exists.
    #[error("Username already exists")]
    UsernameConflict,

    /// An email that was provided already exists.
    #[error("Email already exists")]
    EmailConflict,

    /// One or more role IDs do not exist or are inactive.
    #[error("One or more role IDs are invalid")]
    InvalidRoleId,

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
}

// --- Axum Error Handling ---

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
            // 1xxxx: User/Business Errors
            ServiceError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                10001, // Business-Common-01
                format!("{} not found.", resource),
            ),
            ServiceError::InvalidOperation(reason) => (
                StatusCode::BAD_REQUEST,
                10002, // Business-Common-02
                reason,
            ),
            ServiceError::PasswordHashingFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10003,
                "Password processing failed. Please try again.".into(),
            ),
            ServiceError::UserIsDisabled => {
                (StatusCode::FORBIDDEN, 10004, "User account is disabled.".into())
            }
            ServiceError::UserIsPending => {
                (StatusCode::BAD_REQUEST, 10005, "User account is pending approval.".into())
            }
            ServiceError::UserIsLocked => {
                (StatusCode::BAD_REQUEST, 10006, "User account is locked.".into())
            }
            ServiceError::InvalidUserStatus => {
                (StatusCode::BAD_REQUEST, 10007, "User status is invalid.".into())
            }
            ServiceError::UserIsAdmin => {
                (StatusCode::BAD_REQUEST, 10008, "Cannot update admin user.".into())
            }
            ServiceError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                10101, // Business-Auth-01
                "Invalid username or password.".to_string(),
            ),
            ServiceError::TokenCreationFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10103, // Business-Auth-03
                "Failed to generate login token. Please try again.".to_string(),
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
            ServiceError::InvalidRoleId => (
                StatusCode::BAD_REQUEST,
                10203, // Business-User-03
                "One or more role IDs are invalid.".to_string(),
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
            // 2xxxx: System Errors
            ServiceError::DatabaseQueryFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001, // System-Common-01
                "Service is temporarily unavailable. Please try again later.".to_string(),
            ),
            // ServiceError::InternalServerError => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     20002, // System-Common-02
            //     "Internal server error. Please contact the administrator.".to_string(),
            // ),
            // 3xxxx: Permission Errors
            ServiceError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                30000, // System-Auth-01
                "Invalid or expired token. Please log in again.".to_string(),
            ),
            ServiceError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                30001, // System-Auth-02
                "You do not have permission to perform this action.".to_string(),
            ),
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
