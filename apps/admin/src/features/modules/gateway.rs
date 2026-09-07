use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderName, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use rustzen_auth::error::CoreError;
use rustzen_ipc::{
    DelegatedAccess, DelegatedContext, IPC_ACCESS_HEADER, IPC_CONTRACT_VERSION_HEADER,
    IPC_MODULE_HEADER, IPC_REQUEST_ID_HEADER, IPC_SIGNATURE_HEADER, IPC_TIMESTAMP_HEADER,
    IPC_USER_ID_HEADER,
};
use uuid::Uuid;

use crate::{features::auth::session::SessionRepository, infra::auth_runtime::jwt_codec};

use super::{
    service::ModuleControlState,
    types::{GatewayLookup, GatewayTarget},
};

pub fn routes() -> Router<ModuleControlState> {
    Router::new()
        .route("/api", any(api_not_found))
        .route("/api/", any(api_not_found))
        .route("/api/{module}", any(api_not_found))
        .route("/api/{module}/", any(api_not_found))
        .route("/api/{module}/{*path}", any(forward))
}

async fn api_not_found() -> Response {
    status_error(StatusCode::NOT_FOUND, 404, "Not found")
}

async fn forward(State(state): State<ModuleControlState>, request: Request) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let path_and_query =
        request.uri().path_and_query().map_or_else(|| path.clone(), ToString::to_string);
    let (lookup, target) = state.registry.snapshot().lookup(&method, &path);
    let target = match (lookup, target) {
        (GatewayLookup::Found, Some(target)) => target,
        (GatewayLookup::NotFound, _) => {
            return status_error(StatusCode::NOT_FOUND, 404, "Not found");
        }
        (GatewayLookup::MethodNotAllowed, _) => {
            return status_error(StatusCode::METHOD_NOT_ALLOWED, 405, "Method not allowed");
        }
        (GatewayLookup::ServiceUnavailable, _) => {
            return status_error(
                StatusCode::SERVICE_UNAVAILABLE,
                40001,
                module_unavailable_message(&path),
            );
        }
        _ => return status_error(StatusCode::NOT_FOUND, 404, "Not found"),
    };

    let authorization = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let user_id = match authorize(&state.pool, authorization.as_deref(), &target).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };
    let context = match DelegatedContext::new(
        Uuid::new_v4().to_string(),
        user_id,
        target.module.clone(),
        method.clone(),
        path,
        match target.access {
            rustzen_ipc::AccessMode::Public => DelegatedAccess::Public,
            rustzen_ipc::AccessMode::Protected => DelegatedAccess::Protected(
                target.permission.clone().expect("validated protected route permission"),
            ),
        },
    ) {
        Ok(context) => context,
        Err(error) => {
            tracing::error!(%error, "Failed to construct delegated request context");
            return status_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                500,
                "Gateway delegation failed",
            );
        }
    };
    let delegated_headers = match state.signer.sign(&context) {
        Ok(headers) => headers,
        Err(error) => {
            tracing::error!(%error, "Failed to sign delegated request");
            return status_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                500,
                "Gateway delegation failed",
            );
        }
    };

    let (parts, body) = request.into_parts();
    let mut headers = forwarded_request_headers(&parts.headers);
    headers.extend(delegated_headers);
    let upstream = state
        .client
        .request(method, format!("{}{}", target.base_url, path_and_query))
        .headers(headers)
        .body(reqwest::Body::wrap_stream(body.into_data_stream()))
        .send()
        .await;
    let upstream = match upstream {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, module = %target.module, "Module gateway request failed");
            return status_error(
                StatusCode::SERVICE_UNAVAILABLE,
                40001,
                format!("{} worker is temporarily unavailable.", target.module),
            );
        }
    };

    let status = upstream.status();
    let headers = forwarded_response_headers(upstream.headers());
    let mut response = Response::builder().status(status);
    if let Some(response_headers) = response.headers_mut() {
        response_headers.extend(headers);
    }
    response.body(Body::from_stream(upstream.bytes_stream())).unwrap_or_else(|error| {
        tracing::error!(%error, "Failed to construct gateway response");
        status_error(StatusCode::INTERNAL_SERVER_ERROR, 500, "Gateway response failed")
    })
}

async fn authorize(
    pool: &sqlx::SqlitePool,
    authorization: Option<&str>,
    target: &GatewayTarget,
) -> Result<Option<i64>, CoreError> {
    let Some(permission) = target.permission.as_deref() else {
        return Ok(None);
    };
    let token = authorization
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(CoreError::InvalidToken)?;
    let claims = jwt_codec().decode(token).map_err(|_| CoreError::InvalidToken)?;
    let user =
        SessionRepository::load_authoritative_user(pool, &claims, chrono::Utc::now().timestamp())
            .await
            .map_err(|error| match error {
                crate::common::error::ServiceError::DatabaseQueryFailed => {
                    CoreError::AuthorityUnavailable
                }
                _ => CoreError::InvalidToken,
            })?;
    if !user.has_capability(permission) {
        return Err(CoreError::PermissionDenied);
    }
    Ok(Some(user.user_id))
}

fn forwarded_request_headers(source: &HeaderMap) -> HeaderMap {
    source
        .iter()
        .filter(|(name, _)| !is_private_request_header(name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn forwarded_response_headers(source: &HeaderMap) -> HeaderMap {
    source
        .iter()
        .filter(|(name, _)| !is_hop_by_hop(name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn is_private_request_header(name: &HeaderName) -> bool {
    is_hop_by_hop(name)
        || name == header::HOST
        || name == header::AUTHORIZATION
        || name == header::CONTENT_LENGTH
        || name == IPC_CONTRACT_VERSION_HEADER
        || name == IPC_TIMESTAMP_HEADER
        || name == IPC_REQUEST_ID_HEADER
        || name == IPC_USER_ID_HEADER
        || name == IPC_MODULE_HEADER
        || name == IPC_ACCESS_HEADER
        || name == IPC_SIGNATURE_HEADER
}

fn is_hop_by_hop(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn module_unavailable_message(path: &str) -> String {
    let module = path.trim_start_matches('/').split('/').nth(1).unwrap_or("module");
    format!("{module} worker is temporarily unavailable.")
}

fn status_error(status: StatusCode, code: i32, message: impl Into<String>) -> Response {
    let message = message.into();
    (status, axum::Json(serde_json::json!({ "code": code, "message": message, "data": null })))
        .into_response()
}

#[cfg(all(test, feature = "full"))]
#[path = "gateway_tests.rs"]
mod tests;
