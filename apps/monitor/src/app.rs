use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use rustzen_ipc::{
    DelegationVerifier, HealthResponse, ModuleDefinition, ModuleManifest, ModuleRouter, Require,
};
use rustzen_storage::SqlitePool;

use crate::{config, features, infra};

const MODULE_TOML: &str = include_str!("../module.toml");

#[derive(Clone)]
pub(crate) struct AppState {
    pub pool: SqlitePool,
    pub agent_token: Arc<str>,
    manifest: Arc<ModuleManifest>,
}

pub async fn run_controller() -> Result<(), Box<dyn std::error::Error>> {
    let pool = infra::db::connect().await?;
    infra::db::migrate(&pool).await?;
    infra::db::verify(&pool).await?;
    features::metrics::spawn_retention(pool.clone());
    features::checks::spawn_scheduler(pool.clone());
    features::checks::spawn_retention(pool.clone());
    features::incidents::spawn_evaluator(pool.clone());

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
    let definition = ModuleDefinition::from_toml(MODULE_TOML)?;
    let module_id = definition.module.id.clone();
    let api_prefix = definition.module.api_prefix.clone();
    let verifier = DelegationVerifier::new(&config::controller().ipc_token)?;
    let module_router = ModuleRouter::<AppState>::new(module_id, verifier)
        .post_public("/heartbeat", features::heartbeat::handler::submit)?
        .get_with_permission(
            "/overview",
            features::nodes::handler::overview,
            Require(rustzen_auth::capability::monitor::OVERVIEW_VIEW),
        )?
        .get_with_permission(
            "/nodes",
            features::nodes::handler::list,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?
        .get_with_permission(
            "/nodes/{node_id}",
            features::nodes::handler::get,
            Require(rustzen_auth::capability::monitor::NODE_VIEW),
        )?;
    let module_router = features::metrics::routes(module_router)?;
    let module_router = features::checks::routes(module_router)?;
    let module_router = features::incidents::routes(module_router)?;
    let (module_routes, manifest) = module_router.build(&definition, env!("CARGO_PKG_VERSION"))?;
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
    Json(HealthResponse::ok(env!("CARGO_PKG_VERSION")))
}

async fn runtime_manifest(State(state): State<AppState>) -> Json<ModuleManifest> {
    Json((*state.manifest).clone())
}

#[cfg(test)]
mod tests {
    use rustzen_ipc::AccessMode;

    use crate::infra::db::migrated_test_pool;

    use super::build_app;

    #[tokio::test]
    async fn runtime_manifest_is_derived_from_the_registered_routes() {
        let pool = migrated_test_pool().await;
        let (_, manifest) = build_app(pool, "agent-secret".to_string()).expect("build app");

        assert_eq!(manifest.module, "monitor");
        assert_eq!(manifest.api_prefix, "/api/monitor");
        assert_eq!(manifest.release_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.menus.len(), 4);
        assert!(manifest.menus.iter().any(|menu| {
            menu.code == "incidents"
                && menu.path == "/monitoring/incidents"
                && menu.permission == "monitor:incident:view"
        }));
        assert_eq!(manifest.routes.len(), 15);
        assert!(manifest.routes.iter().any(|route| {
            route.method == "POST"
                && route.path == "/heartbeat"
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
                == 3
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
    }
}
