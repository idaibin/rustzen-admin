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

#[tokio::test]
async fn protected_gateway_uses_authoritative_database_permissions() {
    let target = GatewayTarget {
        module: "reports".to_string(),
        base_url: "http://127.0.0.1:9804".to_string(),
        access: AccessMode::Protected,
        permission: Some("reports:view".to_string()),
    };
    let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, status) VALUES
             (8, 'gateway-user', 'gateway@example.com', 'unused', 1)",
    )
    .execute(&pool)
    .await
    .expect("gateway user");
    assert!(matches!(authorize(&pool, None, &target).await, Err(CoreError::InvalidToken)));

    let token = jwt_codec().encode(8, "gateway-user").expect("token");
    let claims = jwt_codec().decode(&token).expect("claims");
    crate::features::auth::session::SessionRepository::create(
        &pool,
        8,
        &claims.sid,
        claims.user_auth_epoch,
        claims.iat as i64,
        claims.exp as i64,
    )
    .await
    .expect("session");
    let bearer = format!("Bearer {token}");
    assert!(matches!(
        authorize(&pool, Some(&bearer), &target).await,
        Err(CoreError::PermissionDenied)
    ));

    let menu_id: i64 = sqlx::query_scalar(
        "INSERT INTO menus (name, code, menu_type, status, is_active)
             VALUES ('Reports view', 'reports:view', 3, 1, 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("permission");
    let role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status) VALUES ('Gateway', 'gateway', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("role");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (8, ?)")
        .bind(role_id)
        .execute(&pool)
        .await
        .expect("membership");
    sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
        .bind(role_id)
        .bind(menu_id)
        .execute(&pool)
        .await
        .expect("grant");
    assert_eq!(authorize(&pool, Some(&bearer), &target).await.expect("authorized"), Some(8));
}

#[tokio::test]
async fn gateway_fails_closed_when_the_authority_database_is_closed() {
    let verifier = DelegationVerifier::new("test-secret").expect("verifier");
    let upstream = axum::Router::new().route("/api/reports/echo", post(echo)).with_state(verifier);
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
    let spec = ModuleSpec { id: "reports", name: "Reports", base_url: format!("http://{address}") };
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
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers().get(header::CACHE_CONTROL).unwrap(), "no-store");
    assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "60");
    let body = to_bytes(response.into_body(), usize::MAX).await.expect("authority body");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).expect("authority JSON"),
        serde_json::json!({
            "code": 50302,
            "message": "Authorization authority unavailable",
            "data": null
        })
    );

    let snapshot = registry.snapshot();
    let mut modules = snapshot.as_ref().clone().into_modules();
    modules.get_mut("reports").expect("reports runtime").condition = ModuleCondition::Unavailable;
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
    modules.get_mut("reports").expect("reports runtime").condition = ModuleCondition::Incompatible;
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
