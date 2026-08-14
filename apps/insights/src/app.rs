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
    pub ingestion: Arc<features::tracking::IngestionState>,
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
    let module = features::settings::register(module)?;
    let module = features::overview::register(module)?;
    let module = features::query::register(module)?;
    let (module_routes, manifest) = module.build(&definition, env!("CARGO_PKG_VERSION"))?;
    let state = AppState {
        pool,
        ingestion: features::tracking::IngestionState::new(),
        manifest: Arc::new(manifest),
    };

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
        assert_eq!(manifest["routes"].as_array().unwrap().len(), 7);
        assert!(manifest["routes"].as_array().unwrap().iter().any(|route| {
            route["method"] == "OPTIONS" && route["path"] == "/track" && route["access"] == "public"
        }));
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
        assert!(manifest["routes"].as_array().unwrap().iter().any(|route| {
            route["method"] == "PUT"
                && route["path"] == "/collection-policy"
                && route["permission"] == "insights:manage"
        }));
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
        assert_eq!(tracked.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://app.example");
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
    async fn public_tracking_requires_project_key_and_origin_policy() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let response = app
            .oneshot(json_request_without_policy(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                json!({ "eventName": "page_view", "visitorId": "v1", "pagePath": "/home" }),
            ))
            .await
            .expect("track");
        assert_ne!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tracking_cors_preflight_requires_enabled_allowed_origin() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let allowed = app
            .clone()
            .oneshot(preflight_request(
                "HTTPS://APP.EXAMPLE:443/",
                "POST",
                "content-type, x-rustzen-project-key",
            ))
            .await
            .expect("allowed preflight");
        assert_eq!(allowed.status(), StatusCode::NO_CONTENT);
        assert_eq!(allowed.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://app.example");
        assert_eq!(allowed.headers()[header::ACCESS_CONTROL_ALLOW_METHODS], "POST");
        assert_eq!(
            allowed.headers()[header::ACCESS_CONTROL_ALLOW_HEADERS],
            "content-type, x-rustzen-project-key"
        );
        assert_eq!(allowed.headers()[header::VARY], "Origin");

        let denied = app
            .oneshot(preflight_request(
                "https://not-allowed.example",
                "POST",
                "content-type, x-rustzen-project-key",
            ))
            .await
            .expect("denied preflight");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        assert!(denied.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
        assert_eq!(denied.headers()[header::VARY], "Origin");
    }

    #[tokio::test]
    async fn tracking_cors_post_echoes_only_verified_origin() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let allowed = app
            .clone()
            .oneshot(tracking_json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                "test-project-key",
                "https://app.example",
                json!({
                    "eventName": "unknown",
                    "visitorId": "visitor-invalid",
                    "pagePath": "/cors"
                }),
            ))
            .await
            .expect("allowed business error");
        assert_eq!(allowed.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(allowed.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://app.example");
        assert_eq!(allowed.headers()[header::VARY], "Origin");

        let denied = app
            .oneshot(tracking_json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                "test-project-key",
                "https://not-allowed.example",
                json!({
                    "eventName": "page_view",
                    "visitorId": "visitor-denied",
                    "pagePath": "/cors"
                }),
            ))
            .await
            .expect("denied origin");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        assert!(denied.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
        assert!(denied.headers().get(header::ACCESS_CONTROL_ALLOW_HEADERS).is_none());
        assert_eq!(denied.headers()[header::VARY], "Origin");
    }

    #[tokio::test]
    async fn oversized_batch_is_rejected_without_persistence() {
        let pool = test_pool().await;
        let app = build_router(pool, SECRET).expect("router");
        let oversized = json!({
            "eventName": "page_view",
            "visitorId": "v1",
            "pagePath": "/home",
            "properties": { "feature": "x".repeat(66_000) }
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                oversized,
            ))
            .await
            .expect("track");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let query = app
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/events",
                DelegatedAccess::protected("insights:event:view"),
                Body::empty(),
            ))
            .await
            .expect("query");
        assert_eq!(response_json(query).await["data"]["total"], 0);
    }

    #[tokio::test]
    async fn batch_limit_rejects_without_partial_persistence() {
        let pool = test_pool().await;
        let app = build_router(pool, SECRET).expect("router");
        let events = (0..51)
            .map(|index| {
                json!({
                    "eventName": "page_view",
                    "visitorId": format!("visitor-{index}"),
                    "pagePath": "/home"
                })
            })
            .collect::<Vec<_>>();
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/insights/track",
                DelegatedAccess::Public,
                Value::Array(events),
            ))
            .await
            .expect("track");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let query = app
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/events",
                DelegatedAccess::protected("insights:event:view"),
                Body::empty(),
            ))
            .await
            .expect("query");
        assert_eq!(response_json(query).await["data"]["total"], 0);
    }

    #[tokio::test]
    async fn per_project_origin_rate_limit_rejects_the_whole_request() {
        let pool = test_pool().await;
        let app = build_router(pool, SECRET).expect("router");
        let mut statuses = Vec::new();
        for index in 0..31 {
            let response = app
                .clone()
                .oneshot(json_request(
                    Method::POST,
                    "/api/insights/track",
                    DelegatedAccess::Public,
                    json!({
                        "eventName": "page_view",
                        "visitorId": format!("visitor-{index}"),
                        "pagePath": "/home"
                    }),
                ))
                .await
                .expect("track");
            statuses.push(response.status());
        }
        assert_eq!(statuses.iter().filter(|status| **status == StatusCode::OK).count(), 30);
        assert_eq!(statuses.last(), Some(&StatusCode::TOO_MANY_REQUESTS));

        let query = app
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/events",
                DelegatedAccess::protected("insights:event:view"),
                Body::empty(),
            ))
            .await
            .expect("query");
        assert_eq!(response_json(query).await["data"]["total"], 30);
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
                    { "eventName": "page_view", "visitorId": "v1", "pagePath": "/settings" },
                    { "eventName": "custom_export", "visitorId": "v1", "pagePath": "/exports/100%_done" }
                ]),
            ))
            .await
            .expect("track");
        assert_eq!(response_json(tracked).await["data"]["accepted"], 5);

        for (query, expected_names, expected_total) in [
            ("eventKind=page", vec!["page_view", "page_view"], 2),
            ("eventKind=api", vec!["api_request"], 1),
            ("eventKind=other", vec!["custom_export", "custom_export"], 2),
            ("path=analytics", vec!["custom_export", "page_view"], 2),
            ("eventKind=page&path=analytics", vec!["page_view"], 1),
            ("path=%25_", vec!["custom_export"], 1),
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

    #[tokio::test]
    async fn collection_policy_requires_the_manage_capability() {
        let app = build_router(test_pool().await, SECRET).expect("router");
        let unsigned = app
            .clone()
            .oneshot(Request::get("/api/insights/collection-policy").body(Body::empty()).unwrap())
            .await
            .expect("unsigned policy request");
        assert_eq!(unsigned.status(), StatusCode::UNAUTHORIZED);

        let viewer = app
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/collection-policy",
                DelegatedAccess::protected("insights:event:view"),
                Body::empty(),
            ))
            .await
            .expect("viewer policy request");
        assert_eq!(viewer.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn collection_policy_put_rotates_hash_without_returning_key() {
        let pool = test_pool().await;
        let app = build_router(pool.clone(), SECRET).expect("router");
        let current = app
            .clone()
            .oneshot(signed_request(
                Method::GET,
                "/api/insights/collection-policy",
                DelegatedAccess::protected("insights:manage"),
                Body::empty(),
            ))
            .await
            .expect("current policy");
        let current = response_json(current).await;
        assert_eq!(current["data"]["collectionEnabled"], true);
        assert_eq!(current["data"]["projectConfigured"], true);
        assert!(current["data"].get("projectKey").is_none());

        let updated = app
            .clone()
            .oneshot(managed_json_request(
                Method::PUT,
                "/api/insights/collection-policy",
                json!({
                    "collectionEnabled": false,
                    "projectKey": "rotated-project-key",
                    "allowedOrigins": ["HTTPS://APP.EXAMPLE:443/"]
                }),
            ))
            .await
            .expect("update policy");
        assert_eq!(updated.status(), StatusCode::OK);
        let updated = response_json(updated).await;
        assert_eq!(updated["data"]["collectionEnabled"], false);
        assert_eq!(updated["data"]["allowedOrigins"], json!(["https://app.example"]));
        assert!(updated["data"].get("projectKey").is_none());

        let stored_hash: String = sqlx::query_scalar(
            "SELECT project_key_hash FROM insights_projects WHERE id = 'default'",
        )
        .fetch_one(&pool)
        .await
        .expect("stored project key hash");
        assert_eq!(
            stored_hash,
            crate::features::settings::service::hash_project_key("rotated-project-key")
                .expect("hash")
        );
        assert!(!stored_hash.contains("rotated-project-key"));
    }

    #[tokio::test]
    async fn collection_policy_put_configures_a_clean_install_without_sql() {
        let pool = test_pool_without_collection_policy().await;
        let app = build_router(pool.clone(), SECRET).expect("router");
        let response = app
            .oneshot(managed_json_request(
                Method::PUT,
                "/api/insights/collection-policy",
                json!({
                    "collectionEnabled": true,
                    "projectKey": "fresh-project-key",
                    "allowedOrigins": ["https://fresh.example"]
                }),
            ))
            .await
            .expect("configure collection policy");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["data"]["collectionEnabled"], true);
        assert_eq!(body["data"]["projectConfigured"], true);
        assert_eq!(body["data"]["allowedOrigins"], json!(["https://fresh.example"]));
        assert!(body["data"].get("projectKey").is_none());

        let stored: (i64, String) = sqlx::query_as(
            "SELECT collection_enabled, project_key_hash
             FROM insights_projects WHERE id = 'default'",
        )
        .fetch_one(&pool)
        .await
        .expect("persisted clean-install policy");
        assert_eq!(stored.0, 1);
        assert_eq!(
            stored.1,
            crate::features::settings::service::hash_project_key("fresh-project-key")
                .expect("hash")
        );
    }

    #[tokio::test]
    async fn collection_policy_requires_a_new_key_before_first_enable() {
        let pool = test_pool_without_collection_policy().await;
        let app = build_router(pool, SECRET).expect("router");
        let response = app
            .oneshot(managed_json_request(
                Method::PUT,
                "/api/insights/collection-policy",
                json!({
                    "collectionEnabled": true,
                    "allowedOrigins": ["https://fresh.example"]
                }),
            ))
            .await
            .expect("enable collection policy");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn policy_changes_reject_old_tracking_without_new_rows() {
        let pool = test_pool().await;
        let app = build_router(pool.clone(), SECRET).expect("router");

        for (label, update, key) in [
            (
                "disabled",
                json!({
                    "collectionEnabled": false,
                    "allowedOrigins": ["https://app.example"]
                }),
                "test-project-key",
            ),
            (
                "rotated",
                json!({
                    "collectionEnabled": true,
                    "projectKey": "rotated-project-key",
                    "allowedOrigins": ["https://app.example"]
                }),
                "test-project-key",
            ),
            (
                "origin-removed",
                json!({
                    "collectionEnabled": true,
                    "projectKey": "rotated-project-key",
                    "allowedOrigins": []
                }),
                "rotated-project-key",
            ),
        ] {
            let updated = app
                .clone()
                .oneshot(managed_json_request(
                    Method::PUT,
                    "/api/insights/collection-policy",
                    update,
                ))
                .await
                .expect("policy update");
            assert_eq!(updated.status(), StatusCode::OK, "{label}");

            let tracked = app
                .clone()
                .oneshot(tracking_json_request(
                    Method::POST,
                    "/api/insights/track",
                    DelegatedAccess::Public,
                    key,
                    "https://app.example",
                    json!({
                        "eventName": "page_view",
                        "visitorId": format!("visitor-{label}"),
                        "pagePath": "/policy-barrier"
                    }),
                ))
                .await
                .expect("track after policy update");
            assert_eq!(tracked.status(), StatusCode::FORBIDDEN, "{label}");
        }

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM insights_events")
            .fetch_one(&pool)
            .await
            .expect("event count");
        assert_eq!(total, 0);
    }

    async fn test_pool() -> sqlx::SqlitePool {
        let pool = test_pool_without_collection_policy().await;
        sqlx::query(
            "UPDATE insights_projects
             SET collection_enabled = 1,
                 project_key_hash = ?,
                 allowed_origins = '[\"https://app.example\"]'",
        )
        .bind("d749661188e7505ba96c87f623991d468dbbb04222ad81bcc7f5f5c110fb598e")
        .execute(&pool)
        .await
        .expect("enable test collection policy");
        pool
    }

    async fn test_pool_without_collection_policy() -> sqlx::SqlitePool {
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
        tracking_json_request(method, uri, access, "test-project-key", "https://app.example", body)
    }

    fn tracking_json_request(
        method: Method,
        uri: &str,
        access: DelegatedAccess,
        project_key: &str,
        origin: &str,
        body: Value,
    ) -> Request<Body> {
        let mut request = signed_request(method, uri, access, Body::from(body.to_string()));
        request
            .headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse().expect("content type"));
        request
            .headers_mut()
            .insert("x-rustzen-project-key", project_key.parse().expect("project key"));
        request.headers_mut().insert(header::ORIGIN, origin.parse().expect("origin"));
        request
    }

    fn preflight_request(origin: &str, method: &str, requested_headers: &str) -> Request<Body> {
        let mut request = signed_request(
            Method::OPTIONS,
            "/api/insights/track",
            DelegatedAccess::Public,
            Body::empty(),
        );
        request.headers_mut().insert(header::ORIGIN, origin.parse().expect("origin"));
        request
            .headers_mut()
            .insert(header::ACCESS_CONTROL_REQUEST_METHOD, method.parse().expect("request method"));
        request.headers_mut().insert(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            requested_headers.parse().expect("request headers"),
        );
        request
    }

    fn json_request_without_policy(
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

    fn managed_json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
        let mut request = signed_request(
            method,
            uri,
            DelegatedAccess::protected("insights:manage"),
            Body::from(body.to_string()),
        );
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
