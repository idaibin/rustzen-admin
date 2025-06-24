use crate::{
    common::error::{AppError, ServiceError},
    core::jwt,
    features::system::user::{model::UserStatus, repo::UserRepository},
};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    // 1. 检查 Authorization header
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::from(ServiceError::InvalidCredentials))?;

    // 2. 验证 JWT token
    let claims = jwt::verify_token(token).map_err(|_| ServiceError::InvalidToken)?;

    // 3. 验证用户在数据库中的有效性
    let user = UserRepository::find_by_id(&pool, claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while validating user: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?
        .ok_or_else(|| {
            tracing::warn!("User {} from valid token not found in database", claims.user_id);
            ServiceError::InvalidToken
        })?;

    // 4. 检查用户状态
    if let Some(status) = UserStatus::from_i16(user.status) {
        if !status.is_active() {
            tracing::warn!(
                "Disabled user {} attempted to access protected resource",
                claims.user_id
            );
            return Err(AppError::from(ServiceError::UserIsDisabled));
        }
    } else {
        // 无效状态，也拒绝访问
        tracing::warn!("User {} has invalid status: {}", claims.user_id, user.status);
        return Err(AppError::from(ServiceError::UserIsDisabled));
    }

    // 5. 验证用户名一致性（防止token篡改）
    if user.username != claims.username {
        tracing::warn!(
            "Username mismatch: token has '{}' but database has '{}' for user ID {}",
            claims.username,
            user.username,
            claims.user_id
        );
        return Err(AppError::from(ServiceError::InvalidToken));
    }

    // 6. 验证通过，将claims插入到request中
    tracing::debug!("User {} successfully authenticated", claims.user_id);
    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}
