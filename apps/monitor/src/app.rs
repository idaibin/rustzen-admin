use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use rustzen_ipc::{HealthResponse, ModuleManifest};
use rustzen_storage::SqlitePool;

use crate::{config, features, infra, module_routes::build_module_routes};

#[derive(Clone)]
pub(crate) struct AppState {
    pub pool: SqlitePool,
    pub agent_token: Arc<str>,
    manifest: Arc<ModuleManifest>,
}

pub async fn run_controller() -> Result<(), Box<dyn std::error::Error>> {
    infra::db::verify_selected_database().await.map_err(std::io::Error::other)?;
    let pool = infra::db::connect().await?;
    features::monitoring::spawn_background(pool.clone());

    let (app, _) = build_app(pool, config::controller().monitor_agent_token.clone())?;
    let address = config::controller().bind_address();
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, "Monitor Controller started");
    axum::serve(listener, app).await?;
    Ok(())
}

pub(crate) fn build_app(
    pool: SqlitePool,
    agent_token: String,
) -> Result<(Router, ModuleManifest), Box<dyn std::error::Error>> {
    let (module_routes, manifest) = build_module_routes(&config::controller().ipc_token)?;
    let api_prefix = manifest.api_prefix.clone();
    let state = AppState {
        pool,
        agent_token: Arc::from(agent_token),
        manifest: Arc::new(manifest.clone()),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/internal/v1/manifest", get(runtime_manifest))
        .nest(&api_prefix, module_routes)
        .with_state(state);
    Ok((app, manifest))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::ok_selected(env!("CARGO_PKG_VERSION")))
}

async fn runtime_manifest(State(state): State<AppState>) -> Json<ModuleManifest> {
    Json((*state.manifest).clone())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::State,
        http::{HeaderMap, Request, StatusCode},
        response::IntoResponse,
    };
    use rustzen_ipc::{AccessMode, DelegatedAccess, DelegatedContext, DelegationSigner};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::features::monitoring::{record_at, submit};
    use crate::infra::db::migrated_test_pool;
    use crate::protocol::{
        AGENT_REPORT_AUTH_HEADER, AGENT_REPORT_METHOD, AGENT_REPORT_PATH, AGENT_REPORT_ROUTE,
        AgentReport, AgentReportStatus, ByteUsage, MAX_AGENT_REPORT_BODY_BYTES,
        parse_agent_response,
    };

    use super::{AppState, build_app};

    #[tokio::test]
    async fn runtime_manifest_is_derived_from_the_registered_routes() {
        let pool = migrated_test_pool().await;
        let (_, manifest) = build_app(pool, "agent-secret".to_string()).expect("build app");

        assert_eq!(manifest.module, "monitor");
        assert_eq!(manifest.api_prefix, "/api/monitor");
        assert_eq!(manifest.release_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.menus.len(), 4);
        assert!(!manifest.menus.iter().any(|menu| menu.code == "settings"));
        assert!(manifest.menus.iter().any(|menu| {
            menu.code == "incidents"
                && menu.path == "/monitoring/incidents"
                && menu.permission == "monitor:incident:view"
        }));
        assert_eq!(manifest.routes.len(), 13);
        assert!(manifest.routes.iter().any(|route| {
            route.method == AGENT_REPORT_METHOD
                && route.path == AGENT_REPORT_ROUTE
                && route.access == AccessMode::Public
                && route.permission.is_none()
        }));
        assert!(manifest.routes.iter().any(|route| {
            route.method == "GET"
                && route.path == "/overview"
                && route.permission.as_deref() == Some("monitor:overview:view")
        }));
        assert!(
            manifest
                .routes
                .iter()
                .filter(|route| {
                    route.method == "GET"
                        && route.path.starts_with("/nodes")
                        && route.permission.as_deref() == Some("monitor:node:view")
                })
                .count()
                == 4
        );
        assert!(
            manifest
                .routes
                .iter()
                .filter(|route| {
                    route.method == "GET"
                        && route.path.starts_with("/incidents")
                        && route.permission.as_deref() == Some("monitor:incident:view")
                })
                .count()
                == 2
        );
        assert!(manifest.routes.iter().all(|route| !route.path.contains("heartbeat")));
        assert!(manifest.routes.iter().all(|route| !route.path.contains("checks")));
        assert!(manifest.routes.iter().any(|route| {
            route.method == "GET"
                && route.path == "/nodes/{node_id}/alert-settings"
                && route.permission.as_deref() == Some("monitor:node:view")
        }));
        assert!(manifest.routes.iter().any(|route| {
            route.method == "GET"
                && route.path == "/alert-settings"
                && route.permission.as_deref() == Some("monitor:node:view")
        }));
        assert_eq!(
            manifest
                .routes
                .iter()
                .filter(|route| {
                    matches!(route.method.as_str(), "PUT" | "DELETE")
                        && route.path == "/nodes/{node_id}/alert-settings"
                        && route.permission.as_deref() == Some("monitor:manage")
                })
                .count(),
            2
        );
    }

    #[tokio::test]
    async fn agent_report_route_returns_401_without_token_and_422_for_invalid_input() {
        let pool = migrated_test_pool().await;
        let (_, manifest) = build_app(pool.clone(), "agent-secret".to_string()).expect("build app");
        let state =
            AppState { pool, agent_token: Arc::from("agent-secret"), manifest: Arc::new(manifest) };
        let report = AgentReport {
            node_id: "route-node".to_string(),
            boot_id: Uuid::new_v4(),
            sequence: 1,
            hostname: "route-node".to_string(),
            agent_version: "test".to_string(),
            collected_at: chrono::Utc::now(),
            cpu_percent: 10.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: Vec::new(),
        };
        let unauthorized = submit(State(state.clone()), Request::new(Body::from("{")))
            .await
            .unwrap_err()
            .into_response();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        let mut invalid_report = report;
        invalid_report.cpu_percent = 101.0;
        let mut headers = HeaderMap::new();
        headers.insert(AGENT_REPORT_AUTH_HEADER, "agent-secret".parse().unwrap());
        let mut request = Request::new(Body::from(serde_json::to_vec(&invalid_report).unwrap()));
        *request.headers_mut() = headers;
        let invalid_pool = migrated_test_pool().await;
        let invalid = submit(
            State(AppState {
                pool: invalid_pool.clone(),
                agent_token: Arc::from("agent-secret"),
                manifest: state.manifest.clone(),
            }),
            request,
        )
        .await
        .unwrap_err()
        .into_response();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM monitor_nodes")
                .fetch_one(&invalid_pool)
                .await
                .unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn agent_report_route_returns_accepted_duplicate_and_stale_envelopes() {
        let pool = migrated_test_pool().await;
        let (_, manifest) = build_app(pool.clone(), "agent-secret".to_string()).expect("build app");
        let state =
            AppState { pool, agent_token: Arc::from("agent-secret"), manifest: Arc::new(manifest) };
        let boot = Uuid::new_v4();
        let at = chrono::Utc::now();
        let report = AgentReport {
            node_id: "route-status-node".to_string(),
            boot_id: boot,
            sequence: 1,
            hostname: "route-status-node".to_string(),
            agent_version: "test".to_string(),
            collected_at: at,
            cpu_percent: 10.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: Vec::new(),
        };

        async fn submit_report(state: AppState, report: &AgentReport) -> serde_json::Value {
            let mut request = Request::new(Body::from(serde_json::to_vec(report).unwrap()));
            request.headers_mut().insert(AGENT_REPORT_AUTH_HEADER, "agent-secret".parse().unwrap());
            let response = submit(State(state), request).await.unwrap().into_response();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            serde_json::from_slice(&body).unwrap()
        }

        let accepted = submit_report(state.clone(), &report).await;
        assert_eq!(accepted["code"], 0);
        assert_eq!(accepted["data"]["status"], "accepted");
        let duplicate = submit_report(state.clone(), &report).await;
        assert_eq!(duplicate["data"]["status"], "duplicate");
        let mut newer = report.clone();
        newer.sequence = 2;
        newer.collected_at = at + chrono::Duration::seconds(30);
        assert_eq!(submit_report(state.clone(), &newer).await["data"]["status"], "accepted");
        assert_eq!(submit_report(state, &report).await["data"]["status"], "stale");
    }

    #[tokio::test]
    async fn historical_fenced_reports_return_200_statuses_but_new_sequence_is_422() {
        let pool = migrated_test_pool().await;
        let (_, manifest) = build_app(pool.clone(), "agent-secret".to_string()).expect("build app");
        let state =
            AppState { pool, agent_token: Arc::from("agent-secret"), manifest: Arc::new(manifest) };
        let received = chrono::Utc::now();
        let historical = received - chrono::Duration::minutes(10);
        let first_boot = Uuid::new_v4();
        let make_report = |boot, sequence, collected_at| AgentReport {
            node_id: "historical-route-node".to_string(),
            boot_id: boot,
            sequence,
            hostname: "historical-route-node".to_string(),
            agent_version: "test".to_string(),
            collected_at,
            cpu_percent: 10.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: Vec::new(),
        };
        record_at(&state.pool, make_report(first_boot, 1, historical), historical).await.unwrap();
        let second = make_report(first_boot, 2, historical + chrono::Duration::seconds(30));
        record_at(&state.pool, second.clone(), second.collected_at).await.unwrap();

        async fn post(state: &AppState, report: &AgentReport) -> axum::response::Response {
            let mut request = Request::new(Body::from(serde_json::to_vec(report).unwrap()));
            request.headers_mut().insert(AGENT_REPORT_AUTH_HEADER, "agent-secret".parse().unwrap());
            match submit(State(state.clone()), request).await {
                Ok(value) => value.into_response(),
                Err(error) => error.into_response(),
            }
        }
        let duplicate = post(&state, &second).await;
        assert_eq!(duplicate.status(), StatusCode::OK);
        let body = axum::body::to_bytes(duplicate.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["data"]["status"],
            "duplicate"
        );
        let lower = make_report(first_boot, 1, historical);
        let lower_response = post(&state, &lower).await;
        assert_eq!(lower_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(lower_response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["data"]["status"],
            "stale"
        );

        let second_boot = Uuid::new_v4();
        let takeover = make_report(second_boot, 1, historical + chrono::Duration::seconds(60));
        record_at(&state.pool, takeover, historical + chrono::Duration::seconds(60)).await.unwrap();
        let retired = make_report(first_boot, 3, historical + chrono::Duration::seconds(90));
        let retired_response = post(&state, &retired).await;
        assert_eq!(retired_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(retired_response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["data"]["status"],
            "stale"
        );
        let new_sequence = make_report(second_boot, 2, historical + chrono::Duration::seconds(90));
        let invalid = post(&state, &new_sequence).await;
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM resource_samples WHERE node_id='historical-route-node'"
            )
            .fetch_one(&state.pool)
            .await
            .unwrap(),
            3
        );
    }
    fn delegated_headers() -> HeaderMap {
        let context = DelegatedContext::new(
            "wire-test",
            None,
            "monitor",
            axum::http::Method::POST,
            AGENT_REPORT_PATH,
            DelegatedAccess::Public,
        )
        .unwrap();
        DelegationSigner::new("rustzen-dev-ipc-token-change-in-production")
            .unwrap()
            .sign(&context)
            .unwrap()
    }

    #[tokio::test]
    async fn agent_report_router_conforms_to_shared_wire_contract() {
        let pool = migrated_test_pool().await;
        let (router, _) = build_app(pool, "agent-secret".into()).unwrap();
        let report = AgentReport {
            node_id: "wire-node".into(),
            boot_id: Uuid::new_v4(),
            sequence: 1,
            hostname: "wire-node".into(),
            agent_version: "test".into(),
            collected_at: chrono::Utc::now(),
            cpu_percent: 1.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: vec![],
        };
        let mut request = Request::builder()
            .method("POST")
            .uri(AGENT_REPORT_PATH)
            .body(Body::from(serde_json::to_vec(&report).unwrap()))
            .unwrap();
        *request.headers_mut() = delegated_headers();
        request.headers_mut().insert(AGENT_REPORT_AUTH_HEADER, "agent-secret".parse().unwrap());
        let response = router.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_success(), "{}", response.status());
        let bytes =
            axum::body::to_bytes(response.into_body(), MAX_AGENT_REPORT_BODY_BYTES).await.unwrap();
        assert_eq!(
            parse_agent_response(true, std::str::from_utf8(&bytes).unwrap()).unwrap(),
            AgentReportStatus::Accepted
        );
        let mut missing_request = Request::builder()
            .method("POST")
            .uri(AGENT_REPORT_PATH)
            .body(Body::from(serde_json::to_vec(&report).unwrap()))
            .unwrap();
        *missing_request.headers_mut() = delegated_headers();
        let missing = router.clone().oneshot(missing_request).await.unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        let mut oversized_request = Request::builder()
            .method("POST")
            .uri(AGENT_REPORT_PATH)
            .body(Body::from(vec![b'x'; MAX_AGENT_REPORT_BODY_BYTES + 1]))
            .unwrap();
        *oversized_request.headers_mut() = delegated_headers();
        oversized_request
            .headers_mut()
            .insert(AGENT_REPORT_AUTH_HEADER, "agent-secret".parse().unwrap());
        let oversized = router.oneshot(oversized_request).await.unwrap();
        assert_eq!(oversized.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
