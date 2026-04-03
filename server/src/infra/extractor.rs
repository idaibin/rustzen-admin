use crate::common::error::{AppError, ServiceError};

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};

/// Current authenticated user info from auth middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    /// User ID from database
    pub user_id: i64,
    /// Username
    pub username: String,
    /// Whether the user is a built-in system administrator.
    pub is_system: bool,
}

impl CurrentUser {
    /// Create new CurrentUser instance
    pub fn new(user_id: i64, username: String, is_system: bool) -> Self {
        Self { user_id, username, is_system }
    }
}

/// Axum extractor for CurrentUser
///
/// Usage: async fn handler(current_user: CurrentUser) -> Response
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<CurrentUser>().cloned().ok_or_else(|| {
            tracing::error!(
                "CurrentUser not found - auth middleware missing or user not authenticated"
            );
            AppError::from(ServiceError::InvalidToken)
        })
    }
}
