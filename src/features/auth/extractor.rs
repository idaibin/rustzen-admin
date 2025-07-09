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
}

impl CurrentUser {
    /// Create new CurrentUser instance
    pub fn new(user_id: i64, username: String) -> Self {
        Self { user_id, username }
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

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            parts.extensions.get::<CurrentUser>().cloned().ok_or_else(|| {
                tracing::error!(
                    "CurrentUser not found - auth middleware missing or user not authenticated"
                );
                AppError::from(ServiceError::InvalidToken)
            })
        }
    }
}
