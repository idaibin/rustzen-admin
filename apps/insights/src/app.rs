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
    build_router_with_ingestion(pool, ipc_token, features::tracking::IngestionState::new())
}

fn build_router_with_ingestion(
    pool: SqlitePool,
    ipc_token: &str,
    ingestion: Arc<features::tracking::IngestionState>,
) -> StartupResult<Router> {
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
    let state = AppState { pool, ingestion, manifest: Arc::new(manifest) };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/internal/v1/manifest", get(runtime_manifest))
        .nest(&api_prefix, module_routes)
        .with_state(state))
}

#[cfg(test)]
fn build_router_with_storage_capacity_checker(
    pool: SqlitePool,
    ipc_token: &str,
    checker: features::tracking::StorageCapacityChecker,
) -> StartupResult<Router> {
    build_router_with_ingestion(
        pool,
        ipc_token,
        features::tracking::IngestionState::with_storage_capacity_checker(Some(checker)),
    )
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::ok(env!("CARGO_PKG_VERSION")))
}

async fn runtime_manifest(State(state): State<AppState>) -> Json<ModuleManifest> {
    Json((*state.manifest).clone())
}

#[cfg(test)]
mod tests;
