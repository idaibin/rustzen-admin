use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::error::CoreError;

use super::{AuthClaims, CurrentUser, JwtCodec};

#[async_trait]
pub trait AuthContextLoader: Clone + Send + Sync + 'static {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError>;
}

pub async fn auth_middleware<L>(
    State((codec, loader)): State<(JwtCodec, L)>,
    mut request: Request,
    next: Next,
) -> Result<Response, CoreError>
where
    L: AuthContextLoader,
{
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(CoreError::InvalidToken)?;

    let claims = codec.decode(token).map_err(|_| CoreError::InvalidToken)?;
    let current_user = loader.load_current_user(&claims).await?;
    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
