use crate::{
    common::api::{AppError, ServiceError},
    core::jwt,
};
use axum::{extract::Request, http::header, middleware::Next, response::Response};

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::from(ServiceError::InvalidCredentials))?;

    let claims = jwt::verify_token(token).map_err(|_| ServiceError::InvalidToken)?;

    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);

    Ok(next.run(request).await)
}
