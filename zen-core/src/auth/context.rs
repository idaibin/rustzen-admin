use std::{collections::HashSet, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
    pub permissions: Arc<HashSet<String>>,
    pub is_super: bool,
}

impl CurrentUser {
    pub fn new(
        user_id: i64,
        username: impl Into<String>,
        permissions: impl IntoIterator<Item = String>,
        is_super: bool,
    ) -> Self {
        Self {
            user_id,
            username: username.into(),
            permissions: Arc::new(permissions.into_iter().collect()),
            is_super,
        }
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = CoreError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<CurrentUser>().cloned().ok_or(CoreError::MissingAuthContext)
    }
}
