use crate::{
    common::error::{AppError, ServiceError},
    infra::extractor::CurrentUser,
    infra::jwt,
};

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

/// JWT authentication middleware.
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    let path = parts.uri.path().to_string();
    let token = extract_bearer_token(&parts.headers, &path)?;
    let claims = jwt::verify_token(token).map_err(|e| {
        tracing::warn!("JWT verification failed for {}: {:?}", path, e);
        ServiceError::InvalidToken
    })?;

    let current_user = CurrentUser::new(claims.user_id, claims.username.clone(), claims.is_system);
    parts.extensions.insert(current_user);
    parts.extensions.insert(pool);

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

fn extract_bearer_token<'a>(
    headers: &'a axum::http::HeaderMap,
    path: &str,
) -> Result<&'a str, AppError> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::debug!("Missing or invalid Authorization header for {}", path);
            AppError::from(ServiceError::InvalidToken)
        })
}
