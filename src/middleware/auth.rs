use crate::{
    common::error::{AppError, ServiceError},
    core::extractor::CurrentUser,
    core::jwt,
};

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

/// JWT authentication middleware
///
/// Steps:
/// 1. Extract JWT from Authorization header
/// 2. Validate token and extract claims
/// 3. Inject CurrentUser and PgPool into request extensions
///
/// Note: Only handles authentication, not authorization
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    tracing::debug!("Auth for: {}", parts.uri.path());

    // Extract Bearer token from Authorization header
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::debug!("Missing/invalid Authorization header for {}", parts.uri.path());
            AppError::from(ServiceError::InvalidToken)
        })?;

    // Verify JWT and extract claims
    let claims = jwt::verify_token(token).map_err(|e| {
        tracing::warn!("JWT verification failed for {}: {:?}", parts.uri.path(), e);
        ServiceError::InvalidToken
    })?;

    tracing::debug!(
        "JWT verified for user {} ({}) accessing {}",
        claims.user_id,
        claims.username,
        parts.uri.path()
    );

    // Inject user and database pool into request extensions
    let current_user = CurrentUser::new(claims.user_id, claims.username.clone());
    parts.extensions.insert(current_user);
    parts.extensions.insert(pool);

    let request = Request::from_parts(parts, body);

    tracing::debug!("Auth completed for user {} ({})", claims.user_id, claims.username);

    Ok(next.run(request).await)
}
