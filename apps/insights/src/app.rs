use std::{error::Error, sync::Arc};

use axum::{Json, Router, extract::State, routing::get};
use rustzen_ipc::{
    DelegationVerifier, HealthResponse, ModuleDefinition, ModuleManifest, ModuleRouter,
};
use rustzen_storage::SqlitePool;

use crate::{config, features, infra};

type StartupResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    manifest: Arc<ModuleManifest>,
}

pub async fn run() -> StartupResult<()> {
    let pool = infra::db::connect().await?;
    features::tracking::spawn_retention(pool.clone());
    let app = build_router(pool, &config::CONFIG.ipc_token)?;
    let address = config::CONFIG.bind_address();
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, "Insights service started");
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn build_router(pool: SqlitePool, ipc_token: &str) -> StartupResult<Router> {
    let definition = ModuleDefinition::from_toml(include_str!("../module.toml"))?;
    let module_id = definition.module.id.clone();
    let api_prefix = definition.module.api_prefix.clone();
    let verifier = DelegationVerifier::new(ipc_token)?;
    let module = ModuleRouter::<AppState>::new(module_id, verifier);
    let module = features::tracking::register(module)?;
    let module = features::overview::register(module)?;
    let module = features::query::register(module)?;
    let (module_routes, manifest) = module.build(&definition, env!("CARGO_PKG_VERSION"))?;
    let state = AppState { pool, manifest: Arc::new(manifest) };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/internal/v1/manifest", get(runtime_manifest))
        .nest(&api_prefix, module_routes)
        .with_state(state))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::ok(env!("CARGO_PKG_VERSION")))
}

async fn runtime_manifest(State(state): State<AppState>) -> Json<ModuleManifest> {
    Json((*state.manifest).clone())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
    };
    use rustzen_ipc::{DelegatedAccess, DelegatedContext, DelegationSigner};
    use serde_json::{Value, json};
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    use crate::infra::db;

    use super::build_router;

    const SECRET: &str = "insights-test-secret";

    #[tokio::test]
    async fn manifest_exposes_only_single_project_analytics_routes() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let response = app
            .oneshot(Request::builder().uri("/internal/v1/manifest").body(Body::empty()).unwrap())
            .await
            .expect("manifest");
        let manifest = response_json(response).await;
        assert_eq!(manifest["module"], "insights");
        assert_eq!(manifest["menus"].as_array().unwrap().len(), 2);
        assert_eq!(manifest["routes"].as_array().unwrap().len(), 4);
        assert!(
            manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| { route["method"] == "POST" && route["path"] == "/track" })
        );
        assert!(
            manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| { route["method"] == "GET" && route["path"] == "/events" })
        );
    }

    #[tokio::test]
    async fn tracking_overview_and_details_do_not_require_a_project() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let tracked = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                json!([
                    { "eventName": "page_view", "visitorId": "v1", "pagePath": "/home" },
                    { "eventName": "api_request", "visitorId": "v1", "apiPath": "/api/items", "durationMs": 40 }
                ]),
            ))
            .await
            .expect("track");
        assert_eq!(tracked.status(), StatusCode::OK);
        assert_eq!(response_json(tracked).await["data"]["accepted"], 2);

        let overview = app
            .clone()
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/overview",
                DelegatedAccess::protected("insights:overview:view"),
                Body::empty(),
            ))
            .await
            .expect("overview");
        assert_eq!(response_json(overview).await["data"]["eventCount"], 2);

        let details = app
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/events",
                DelegatedAccess::protected("insights:event:view"),
                Body::empty(),
            ))
            .await
            .expect("details");
        assert_eq!(response_json(details).await["data"]["total"], 2);
    }

    #[tokio::test]
    async fn event_query_filters_by_kind_path_and_combination_with_correct_totals() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let tracked = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                json!([
                    { "eventName": "page_view", "visitorId": "v1", "pagePath": "/analytics/overview" },
                    { "eventName": "api_request", "visitorId": "v1", "apiPath": "/api/insights/events" },
                    { "eventName": "custom_export", "visitorId": "v1", "pagePath": "/analytics/overview" },
                    { "eventName": "page_view", "visitorId": "v1", "pagePath": "/settings" }
                ]),
            ))
            .await
            .expect("track");
        assert_eq!(response_json(tracked).await["data"]["accepted"], 4);

        for (query, expected_names, expected_total) in [
            ("eventKind=page", vec!["page_view", "page_view"], 2),
            ("eventKind=api", vec!["api_request"], 1),
            ("eventKind=other", vec!["custom_export"], 1),
            ("path=analytics", vec!["custom_export", "page_view"], 2),
            ("eventKind=page&path=analytics", vec!["page_view"], 1),
        ] {
            let response = app
                .clone()
                .oneshot(signed_request(
                    Method::GET,
                    &format!("/api/insights/events?{query}"),
                    DelegatedAccess::protected("insights:event:view"),
                    Body::empty(),
                ))
                .await
                .expect("filtered events");
            let body = response_json(response).await;
            assert_eq!(body["data"]["total"], expected_total, "{query}");
            assert_eq!(
                body["data"]["data"]
                    .as_array()
                    .expect("event data")
                    .iter()
                    .map(|event| event["eventName"].as_str().expect("event name"))
                    .collect::<Vec<_>>(),
                expected_names,
                "{query}",
            );
        }
    }

    async fn test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        db::migrate(&pool).await.expect("migrate");
        pool
    }

    fn json_request(
        method: Method,
        uri: &str,
        access: DelegatedAccess,
        body: Value,
    ) -> Request<Body> {
        let mut request = signed_request(method, uri, access, Body::from(body.to_string()));
        request
            .headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse().expect("content type"));
        request
    }

    fn signed_request(
        method: Method,
        uri: &str,
        access: DelegatedAccess,
        body: Body,
    ) -> Request<Body> {
        let path = uri.split('?').next().expect("path");
        let user_id = matches!(access, DelegatedAccess::Protected(_)).then_some(7);
        let context = DelegatedContext::new(
            "insights-test-request",
            user_id,
            "insights",
            method.clone(),
            path,
            access,
        )
        .expect("context");
        let headers = DelegationSigner::new(SECRET).unwrap().sign(&context).unwrap();
        let mut request = Request::builder().method(method).uri(uri).body(body).unwrap();
        for (name, value) in headers {
            if let Some(name) = name {
                request.headers_mut().insert(name, value);
            }
        }
        request
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let body = to_bytes(response.into_body(), 1024 * 1024).await.expect("response body");
        serde_json::from_slice(&body).expect("JSON response")
    }
}
