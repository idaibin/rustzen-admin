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

use crate::infra::{auth_runtime::jwt_codec, permission::PermissionService};

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

    let user_id = match authorize(&request, &target) {
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

fn authorize(request: &Request, target: &GatewayTarget) -> Result<Option<i64>, CoreError> {
    let Some(permission) = target.permission.as_deref() else {
        return Ok(None);
    };
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(CoreError::InvalidToken)?;
    let claims = jwt_codec().decode(token).map_err(|_| CoreError::InvalidToken)?;
    let user = PermissionService::load_current_user(claims.user_id, &claims.username)
        .map_err(|_| CoreError::InvalidToken)?;
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
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use axum::{
        body::{Body, Bytes, to_bytes},
        extract::{Request, State},
        http::{Method, StatusCode, header},
        routing::post,
    };
    use rustzen_auth::error::CoreError;
    use rustzen_ipc::{
        AccessMode, DelegatedAccess, DelegationSigner, DelegationVerifier, ModuleManifest,
        RouteManifest,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    use super::{authorize, routes};
    use crate::{
        features::modules::{
            registry::{ModuleRegistry, RegistrySnapshot},
            service::ModuleControlState,
            types::{GatewayTarget, ModuleCondition, ModuleRuntime, ModuleSpec},
        },
        infra::{auth_runtime::jwt_codec, permission::PermissionService},
    };

    #[test]
    fn protected_gateway_rejects_missing_and_insufficient_memory_credentials() {
        let target = GatewayTarget {
            module: "reports".to_string(),
            base_url: "http://127.0.0.1:9804".to_string(),
            access: AccessMode::Protected,
            permission: Some("reports:view".to_string()),
        };
        let missing = Request::builder().body(Body::empty()).expect("missing request");
        assert!(matches!(authorize(&missing, &target), Err(CoreError::InvalidToken)));

        PermissionService::cache_user_permissions(8, &["reports:export".to_string()]);
        let token = jwt_codec().encode(8, "gateway-user").expect("token");
        let insufficient = Request::builder()
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .expect("insufficient request");
        assert!(matches!(authorize(&insufficient, &target), Err(CoreError::PermissionDenied)));

        PermissionService::cache_user_permissions(8, &["reports:*".to_string()]);
        assert_eq!(authorize(&insufficient, &target).expect("wildcard authorized"), Some(8));

        PermissionService::cache_user_permissions(8, &["reports:view".to_string()]);
        assert_eq!(authorize(&insufficient, &target).expect("authorized"), Some(8));
        PermissionService::clear_user_cache(8);
    }

    #[tokio::test]
    async fn warm_gateway_streams_with_memory_auth_and_a_closed_database() {
        let verifier = DelegationVerifier::new("test-secret").expect("verifier");
        let upstream =
            axum::Router::new().route("/api/reports/echo", post(echo)).with_state(verifier);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind upstream");
        let address = listener.local_addr().expect("upstream address");
        let server = tokio::spawn(async move {
            axum::serve(listener, upstream).await.expect("serve upstream");
        });

        let manifest = ModuleManifest {
            module: "reports".to_string(),
            name: "Reports".to_string(),
            api_prefix: "/api/reports".to_string(),
            contract_version: 1,
            release_version: env!("CARGO_PKG_VERSION").to_string(),
            menus: Vec::new(),
            routes: vec![RouteManifest {
                method: "POST".to_string(),
                path: "/echo".to_string(),
                access: AccessMode::Protected,
                permission: Some("reports:view".to_string()),
            }],
        };
        let spec =
            ModuleSpec { id: "reports", name: "Reports", base_url: format!("http://{address}") };
        let registry = ModuleRegistry::new(vec![spec.clone()], &BTreeMap::new());
        registry.replace(RegistrySnapshot::from_modules(BTreeMap::from([(
            "reports".to_string(),
            ModuleRuntime {
                spec,
                enabled: true,
                condition: ModuleCondition::Healthy,
                manifest: Some(Arc::new(manifest)),
                manifest_hash: Some([1; 32]),
                last_seen_at: Some(chrono::Utc::now()),
                error: None,
            },
        )])));
        PermissionService::cache_user_permissions(7, &["reports:view".to_string()]);
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.expect("pool");
        pool.close().await;
        let state = ModuleControlState {
            pool,
            registry: registry.clone(),
            client: reqwest::Client::builder().build().expect("client"),
            signer: DelegationSigner::new("test-secret").expect("signer"),
            enabled_update: Arc::default(),
        };
        let app = routes().with_state(state);
        for uri in ["/api", "/api/", "/api/reports", "/api/reports/", "/api/unknown/path"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).expect("404 request"))
                .await
                .expect("404 response");
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        }
        let malformed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/reports//echo")
                    .body(Body::empty())
                    .expect("malformed request"),
            )
            .await
            .expect("malformed response");
        assert_eq!(malformed.status(), StatusCode::NOT_FOUND);

        let payload = Bytes::from(vec![b'x'; 128 * 1024]);
        let token = jwt_codec().encode(7, "gateway-user").expect("token");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/reports/echo")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .body(Body::from(payload.clone()))
                    .expect("request"),
            )
            .await
            .expect("gateway response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.expect("response body"),
            payload
        );

        let snapshot = registry.snapshot();
        let mut modules = snapshot.as_ref().clone().into_modules();
        modules.get_mut("reports").expect("reports runtime").condition =
            ModuleCondition::Unavailable;
        registry.replace(RegistrySnapshot::from_modules(modules));
        let unavailable = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/reports/echo")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("unavailable request"),
            )
            .await
            .expect("unavailable response");
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(unavailable.into_body(), usize::MAX).await.expect("error body");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).expect("error JSON"),
            serde_json::json!({
                "code": 40001,
                "message": "reports worker is temporarily unavailable.",
                "data": null
            })
        );

        let snapshot = registry.snapshot();
        let mut modules = snapshot.as_ref().clone().into_modules();
        modules.get_mut("reports").expect("reports runtime").condition =
            ModuleCondition::Incompatible;
        registry.replace(RegistrySnapshot::from_modules(modules));
        let incompatible = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/reports/echo")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("incompatible request"),
            )
            .await
            .expect("incompatible response");
        assert_eq!(incompatible.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(incompatible.into_body(), usize::MAX).await.expect("error body");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).expect("error JSON"),
            serde_json::json!({
                "code": 40001,
                "message": "reports worker is temporarily unavailable.",
                "data": null
            })
        );
        PermissionService::clear_user_cache(7);
        server.abort();
    }

    async fn echo(
        State(verifier): State<DelegationVerifier>,
        request: Request,
    ) -> Result<Body, StatusCode> {
        assert!(request.headers().get(header::AUTHORIZATION).is_none());
        let context = verifier
            .verify_for_route(
                request.headers(),
                request.method(),
                request.uri().path(),
                "reports",
                &DelegatedAccess::protected("reports:view"),
            )
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        assert_eq!(context.user_id, Some(7));
        Ok(request.into_body())
    }
}
