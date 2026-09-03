use std::{path::PathBuf, sync::Arc};

use axum::{Json, Router, routing::get};
use rustzen_ipc::{DelegationVerifier, HealthResponse, ModuleDefinition, ModuleRouter};
use rustzen_storage::SqlitePool;

use crate::{config, features::automation, infra::db};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub output_dir: PathBuf,
    pub browser_path: Option<String>,
    pub headless: bool,
    pub max_concurrency: usize,
}

impl AppState {
    fn new(pool: SqlitePool, output_dir: PathBuf) -> Self {
        Self {
            pool,
            output_dir,
            browser_path: config::CONFIG.browser_path().map(str::to_string),
            headless: config::CONFIG.reports_headless,
            max_concurrency: config::CONFIG.reports_max_concurrency,
        }
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::create_pool().await?;
    db::run_migrations(&pool).await?;
    db::test_connection(&pool).await?;
    let output_dir = config::CONFIG.data_dir().join("reports");
    tokio::fs::create_dir_all(&output_dir).await?;
    let state = AppState::new(pool, output_dir);
    automation::initialize(&state).await?;
    let app = build_router(state.clone(), &config::CONFIG.ipc_token)?;
    let address = config::CONFIG.bind_address();
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let workers = automation::spawn(state);
    let shutdown = workers.shutdown.clone();
    tracing::info!(%address, "Reports service started");
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            shutdown.send_replace(true);
        })
        .await;
    workers.shutdown().await;
    result?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! {
            _ = terminate.recv() => {},
            _ = tokio::signal::ctrl_c() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("Reports service shutting down");
}

fn build_router(state: AppState, ipc_token: &str) -> Result<Router, rustzen_ipc::ManifestError> {
    let verifier = DelegationVerifier::new(ipc_token).map_err(rustzen_ipc::ManifestError::from)?;
    let definition = ModuleDefinition::from_toml(include_str!("../module.toml"))?;
    let api_prefix = definition.module.api_prefix.clone();
    let module = ModuleRouter::<AppState>::new(definition.module.id.clone(), verifier);
    let module = automation::routes(module)?;
    let (module_routes, manifest) = module.build(&definition, env!("CARGO_PKG_VERSION"))?;
    let manifest = Arc::new(manifest);
    Ok(Router::new()
        .route("/health", get(health))
        .route(
            "/internal/v1/manifest",
            get(move || {
                let manifest = Arc::clone(&manifest);
                async move { Json(manifest.as_ref().clone()) }
            }),
        )
        .nest(&api_prefix, module_routes)
        .with_state(state))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::ok(env!("CARGO_PKG_VERSION")))
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

    use super::{AppState, build_router};
    use crate::infra::db::MIGRATOR;

    async fn test_app() -> (axum::Router, AppState) {
        let pool =
            SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&pool).await.unwrap();
        let output_dir = std::env::temp_dir().join(format!("rz-reports-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&output_dir).await.unwrap();
        let state =
            AppState { pool, output_dir, browser_path: None, headless: true, max_concurrency: 1 };
        (build_router(state.clone(), "test-secret").unwrap(), state)
    }

    #[tokio::test]
    async fn manifest_exposes_templates_runs_and_live_view_only() {
        let (app, state) = test_app().await;
        let response = app
            .oneshot(Request::builder().uri("/internal/v1/manifest").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let manifest = body(response).await;
        assert_eq!(manifest["module"], "reports");
        assert_eq!(manifest["menus"].as_array().unwrap().len(), 3);
        assert!(manifest["menus"].as_array().unwrap().iter().any(|menu| {
            menu["code"] == "schedules"
                && menu["path"] == "/reports/templates"
                && menu["permission"] == "reports:schedule:view"
        }));
        assert_eq!(manifest["routes"].as_array().unwrap().len(), 23);
        assert!(
            manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| { route["path"] == "/runs/{id}/live-frame" })
        );
        assert!(
            manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| { route["path"] == "/schedules" && route["method"] == "GET" })
        );
        assert!(manifest["routes"].as_array().unwrap().iter().any(|route| {
            route["path"] == "/settings"
                && route["method"] == "GET"
                && route["permission"] == "reports:schedule:view"
        }));
        assert!(manifest["routes"].as_array().unwrap().iter().any(|route| {
            route["path"] == "/flow-options"
                && route["method"] == "GET"
                && route["permission"] == "reports:schedule:view"
        }));
        tokio::fs::remove_dir_all(state.output_dir).await.unwrap();
    }

    #[tokio::test]
    async fn target_template_and_filling_run_form_the_minimal_report_loop() {
        let (app, state) = test_app().await;
        let system = body(
            app.clone()
                .oneshot(request(
                    Method::POST,
                    "/api/reports/systems",
                    "reports:system:manage",
                    json!({"name":"Fixture","baseUrl":"https://fixture.local"}),
                ))
                .await
                .unwrap(),
        )
        .await;
        let system_id = system["data"]["id"].as_str().unwrap();
        let flow = body(
            app.clone()
                .oneshot(request(
                    Method::POST,
                    "/api/reports/flows",
                    "reports:flow:manage",
                    json!({
                        "systemId": system_id,
                        "name": "Monthly filling",
                        "steps": [{"action":"fill","selector":"#value","value":"{{input.value}}"}]
                    }),
                ))
                .await
                .unwrap(),
        )
        .await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/api/reports/runs",
                "reports:run:manage",
                json!({"flowId":flow["data"]["id"],"input":{"value":"42"}}),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let created_run = body(response).await;
        assert_eq!(created_run["data"]["status"], "queued");
        assert!(created_run["data"].get("inputJson").is_none());

        let sensitive = app
            .oneshot(request(
                Method::POST,
                "/api/reports/runs",
                "reports:run:manage",
                json!({"flowId":flow["data"]["id"],"input":{"password":"secret"}}),
            ))
            .await
            .unwrap();
        assert_eq!(sensitive.status(), StatusCode::BAD_REQUEST);
        tokio::fs::remove_dir_all(state.output_dir).await.unwrap();
    }

    #[tokio::test]
    async fn schedules_validate_capability_safe_input_and_lifecycle() {
        let (app, state) = test_app().await;
        let system = body(
            app.clone()
                .oneshot(request(
                    Method::POST,
                    "/api/reports/systems",
                    "reports:system:manage",
                    json!({"name":"Fixture","baseUrl":"https://fixture.local"}),
                ))
                .await
                .unwrap(),
        )
        .await;
        let flow = body(
            app.clone()
                .oneshot(request(
                    Method::POST,
                    "/api/reports/flows",
                    "reports:flow:manage",
                    json!({
                        "systemId": system["data"]["id"],
                        "name": "Scheduled flow",
                        "steps": [{"action":"fill","selector":"#value","value":"{{input.value}}"}]
                    }),
                ))
                .await
                .unwrap(),
        )
        .await;
        let flow_id = flow["data"]["id"].as_str().unwrap();
        let created = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/api/reports/schedules",
                "reports:schedule:manage",
                json!({
                    "flowId": flow_id,
                    "cadence": "daily",
                    "dueTime": "23:59",
                    "input": {"value":"42"},
                    "description": "Nightly report"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::OK);
        let schedule = body(created).await;
        let schedule_id = schedule["data"]["id"].as_str().unwrap();
        assert_eq!(schedule["data"]["cadence"], "daily");
        assert_eq!(schedule["data"]["timezone"], "UTC");
        assert!(schedule["data"]["lastOccurrence"].is_null());

        let flow_options = body(
            app.clone()
                .oneshot(request(
                    Method::GET,
                    "/api/reports/flow-options",
                    "reports:schedule:view",
                    json!({}),
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(flow_options["data"].as_array().unwrap().len(), 1);
        assert_eq!(flow_options["data"][0]["id"], flow_id);
        assert_eq!(flow_options["data"][0]["name"], "Scheduled flow");
        assert_eq!(flow_options["data"][0]["enabled"], true);
        assert!(flow_options["data"][0].get("steps").is_none());

        let flow_view_forbidden = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/api/reports/flow-options",
                "reports:flow:view",
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(flow_view_forbidden.status(), StatusCode::FORBIDDEN);

        let settings = body(
            app.clone()
                .oneshot(request(
                    Method::GET,
                    "/api/reports/settings",
                    "reports:schedule:view",
                    json!({}),
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(settings["data"]["timezone"], "UTC");

        let forbidden = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/api/reports/schedules",
                "reports:schedule:view",
                json!({"flowId":flow_id,"cadence":"daily","dueTime":"23:59"}),
            ))
            .await
            .unwrap();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let sensitive = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/api/reports/schedules",
                "reports:schedule:manage",
                json!({"flowId":flow_id,"cadence":"daily","dueTime":"23:59","input":{"accessToken":"secret"}}),
            ))
            .await
            .unwrap();
        assert_eq!(sensitive.status(), StatusCode::BAD_REQUEST);

        let updated = body(
            app.clone()
                .oneshot(request(
                    Method::PUT,
                    &format!("/api/reports/schedules/{schedule_id}"),
                    "reports:schedule:manage",
                    json!({"flowId":flow_id,"cadence":"weekly","weekday":0,"dueTime":"08:30","enabled":false}),
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(updated["data"]["cadence"], "weekly");
        assert_eq!(updated["data"]["enabled"], false);
        assert!(updated["data"]["nextDue"].is_null());

        let deleted = app
            .clone()
            .oneshot(request(
                Method::DELETE,
                &format!("/api/reports/schedules/{schedule_id}"),
                "reports:schedule:manage",
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::OK);
        tokio::fs::remove_dir_all(state.output_dir).await.unwrap();
    }

    fn request(method: Method, uri: &str, capability: &str, body: Value) -> Request<Body> {
        let context = DelegatedContext::new(
            "request-1",
            Some(7),
            "reports",
            method.clone(),
            uri,
            DelegatedAccess::protected(capability),
        )
        .unwrap();
        let headers = DelegationSigner::new("test-secret").unwrap().sign(&context).unwrap();
        let mut request = Request::builder().method(method).uri(uri);
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn body(response: axum::response::Response) -> Value {
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
    }
}
