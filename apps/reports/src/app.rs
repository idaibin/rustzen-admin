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
    db::verify_existing_database_before_write(&config::CONFIG.database_path())
        .await
        .map_err(std::io::Error::other)?;
    let pool = db::create_pool().await?;
    db::run_migrations(&pool).await?;
    db::verify_selected_schema(&pool).await.map_err(std::io::Error::other)?;
    db::test_connection(&pool).await?;
    let output_dir = config::CONFIG.data_dir().join("reports");
    tokio::fs::create_dir_all(&output_dir).await?;
    let state = AppState::new(pool.clone(), output_dir);
    automation::initialize(&state).await?;
    #[cfg(feature = "notifications")]
    let notification_relay = {
        let (url, key_id, key) = config::CONFIG.notification_transport();
        crate::notifications::start(pool.clone(), url.into(), key_id.into(), key.as_bytes()).await?
    };
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
    #[cfg(feature = "notifications")]
    notification_relay.shutdown().await;
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
mod tests;
