use std::collections::BTreeMap;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    middleware,
};
use rustzen_auth::auth::{CurrentUser, auth_middleware};
use rustzen_ipc::DelegationSigner;
use tower::ServiceExt;

use crate::{
    features::modules::{registry::ModuleRegistry, service::ModuleControlState, types::ModuleSpec},
    infra::auth_runtime::{ServerAuthContextLoader, jwt_codec},
};

use super::{InstallationState, protected_routes, public_routes};

fn module_state(pool: sqlx::SqlitePool) -> ModuleControlState {
    ModuleControlState {
        pool,
        registry: ModuleRegistry::new(ModuleSpec::fixed(), &BTreeMap::new()),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new(b"installation-test-key").expect("signer"),
    }
}

#[tokio::test]
async fn anonymous_binding_is_exact_json_and_never_cached() {
    let (router, contracts) = public_routes().into_parts();
    let response =
        router.oneshot(Request::get("/__web-binding").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 512).await.unwrap()).unwrap();
    assert_eq!(body.as_object().unwrap().len(), 2);
    assert_eq!(body["bindingVersion"], 1);
    assert_eq!(body["webDigest"].as_str().unwrap().len(), 64);
    assert_eq!(contracts[0].path, "/__web-binding");
}

#[tokio::test]
async fn installation_uses_authenticated_identity_and_selected_state() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let state = InstallationState::for_test(module_state(pool.clone()));
    let (router, contracts) = protected_routes().into_parts();
    let app = router
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool)),
            auth_middleware,
        ))
        .with_state(state.clone());
    let unauthenticated = app
        .clone()
        .oneshot(Request::get("/api/installation").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let (router, _) = protected_routes().into_parts();
    let authenticated = router
        .with_state(state)
        .layer(axum::Extension(CurrentUser::new(
            1,
            "owner",
            ["monitor:node:view".to_owned(), "*".to_owned()],
            true,
        )))
        .oneshot(Request::get("/api/installation").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(authenticated.status(), StatusCode::OK);
    assert_eq!(authenticated.headers()[header::CACHE_CONTROL], "no-store");
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(authenticated.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["data"]["featureIds"], serde_json::json!(["access", "monitor"]));
    assert_eq!(body["data"]["capabilities"], serde_json::json!(["*", "monitor:node:view"]));
    assert_eq!(body["data"]["services"][0]["id"], "monitor");
    assert_eq!(body["data"]["services"][0]["state"], "unavailable");
    assert_eq!(contracts[0].path, "/api/installation");
}
