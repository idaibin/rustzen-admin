use std::{sync::Arc, time::Duration};

use chrono::Utc;
#[cfg(feature = "full")]
use rustzen_auth::auth::AuthClaims;
use rustzen_auth::auth::CurrentUser;
use rustzen_ipc::{DelegationSigner, ModuleManifest, ModuleStorageReport};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::{common::error::ServiceError, infra::permission::PermissionService};

#[cfg(feature = "full")]
use super::types::{ModuleHealthResponse, ModuleRuntime, ModuleStatusResponse};
use super::{
    registry::ModuleRegistry,
    repo::ModuleRepository,
    types::{ModuleCondition, ModuleSpec, RuntimeMenuResponse},
};

const SYNC_INTERVAL: Duration = Duration::from_secs(10);
const SYNC_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct ModuleControlState {
    pub pool: SqlitePool,
    pub registry: ModuleRegistry,
    pub client: reqwest::Client,
    pub signer: DelegationSigner,
    #[cfg(feature = "full")]
    pub enabled_update: Arc<tokio::sync::Mutex<()>>,
}

impl ModuleControlState {
    pub async fn initialize(
        pool: SqlitePool,
        client: reqwest::Client,
        signer: DelegationSigner,
    ) -> Result<Self, ServiceError> {
        let enabled = ModuleRepository::load_enabled(&pool).await?;
        let registry = ModuleRegistry::new(ModuleSpec::fixed(), &enabled);
        Ok(Self {
            pool,
            registry,
            client,
            signer,
            #[cfg(feature = "full")]
            enabled_update: Arc::default(),
        })
    }
}

pub struct ModuleService;

impl ModuleService {
    #[cfg(feature = "full")]
    pub fn statuses(state: &ModuleControlState) -> Vec<ModuleStatusResponse> {
        state.registry.snapshot().statuses()
    }

    #[cfg(feature = "full")]
    pub fn dashboard_health(state: &ModuleControlState) -> Vec<ModuleHealthResponse> {
        let snapshot = state.registry.snapshot();
        snapshot
            .display_modules()
            .into_iter()
            .map(|runtime| ModuleHealthResponse {
                module: runtime.spec.id,
                available: runtime.available(),
                release_version: Some(runtime)
                    .filter(|runtime| runtime.available())
                    .and_then(|runtime| runtime.manifest.as_deref())
                    .map(|manifest| manifest.release_version.clone()),
            })
            .collect()
    }

    pub async fn navigation(
        state: &ModuleControlState,
        user: &CurrentUser,
    ) -> Result<Vec<RuntimeMenuResponse>, ServiceError> {
        let snapshot = state.registry.snapshot();
        Ok(ModuleRepository::list_navigation(&state.pool)
            .await?
            .into_iter()
            .filter(|menu| {
                snapshot
                    .modules()
                    .get(&menu.module)
                    .is_some_and(|runtime| runtime.enabled && user.has_capability(&menu.permission))
            })
            .map(|mut menu| {
                if let Some(runtime) = snapshot.modules().get(&menu.module) {
                    menu.module_name = runtime.spec.name.to_string();
                }
                menu
            })
            .collect())
    }

    #[cfg(all(feature = "full", test))]
    pub async fn set_enabled(
        state: &ModuleControlState,
        module: &str,
        enabled: bool,
    ) -> Result<Vec<ModuleStatusResponse>, ServiceError> {
        let _enabled_guard = state.enabled_update.lock().await;
        ModuleRepository::set_enabled(&state.pool, module, enabled).await?;
        if !state.registry.update_module(module, |runtime| {
            if enabled && !runtime.enabled {
                runtime.condition = ModuleCondition::Unavailable;
                runtime.error = Some("awaiting Manifest refresh".to_string());
            }
            runtime.enabled = enabled;
        }) {
            return Err(ServiceError::NotFound(format!("Module {module}")));
        }
        Ok(Self::statuses(state))
    }

    #[cfg(feature = "full")]
    pub async fn set_enabled_authorized(
        state: &ModuleControlState,
        module: &str,
        enabled: bool,
        actor: &AuthClaims,
    ) -> Result<Vec<ModuleStatusResponse>, ServiceError> {
        let _enabled_guard = state.enabled_update.lock().await;
        ModuleRepository::set_enabled_authorized(&state.pool, module, enabled, actor).await?;
        if !state.registry.update_module(module, |runtime| {
            if enabled && !runtime.enabled {
                runtime.condition = ModuleCondition::Unavailable;
                runtime.error = Some("awaiting Manifest refresh".to_string());
            }
            runtime.enabled = enabled;
        }) {
            return Err(ServiceError::NotFound(format!("Module {module}")));
        }
        Ok(Self::statuses(state))
    }

    pub fn spawn_synchronizer(state: ModuleControlState) {
        tokio::spawn(async move {
            loop {
                Self::sync_once(&state).await;
                tokio::time::sleep(SYNC_INTERVAL).await;
            }
        });
    }

    pub async fn sync_once(state: &ModuleControlState) {
        let specs = state
            .registry
            .snapshot()
            .modules()
            .values()
            .filter(|runtime| runtime.enabled)
            .map(|runtime| runtime.spec.clone())
            .collect::<Vec<_>>();
        for spec in specs {
            Self::sync_module(state, spec).await;
        }
    }

    async fn sync_module(state: &ModuleControlState, spec: ModuleSpec) {
        let manifest_url = format!("{}/internal/v1/manifest", spec.base_url);
        let health_url = format!("{}/health", spec.base_url);
        let storage_url = format!("{}/internal/v1/storage", spec.base_url);
        let (manifest_response, health_response, storage_response) = tokio::join!(
            state.client.get(manifest_url).timeout(SYNC_REQUEST_TIMEOUT).send(),
            state.client.get(health_url).timeout(SYNC_REQUEST_TIMEOUT).send(),
            state.client.get(storage_url).timeout(SYNC_REQUEST_TIMEOUT).send(),
        );
        let storage_report = match storage_response {
            Ok(response) if response.status().is_success() => response
                .bytes()
                .await
                .ok()
                .and_then(|bytes| serde_json::from_slice::<ModuleStorageReport>(&bytes).ok()),
            _ => None,
        };
        let store_storage = |runtime: &mut ModuleRuntime| {
            if let Some(report) = &storage_report {
                runtime.storage = Some(report.clone());
            }
        };
        let health_ok = health_response.is_ok_and(|response| response.status().is_success());
        let manifest_bytes = match manifest_response {
            Ok(response) if response.status().is_success() => match response.bytes().await {
                Ok(bytes) => bytes,
                Err(error) => {
                    Self::mark_unavailable(
                        state,
                        spec.id,
                        format!("failed to read Manifest response: {error}"),
                    );
                    return;
                }
            },
            Ok(response) => {
                let error = format!("Manifest endpoint returned {}", response.status());
                if response.status().is_server_error() {
                    Self::mark_unavailable(state, spec.id, error);
                } else {
                    Self::mark_incompatible(state, spec.id, error);
                }
                return;
            }
            Err(error) => {
                Self::mark_unavailable(state, spec.id, error.to_string());
                return;
            }
        };
        let manifest_hash: [u8; 32] = Sha256::digest(&manifest_bytes).into();
        let previous = state.registry.snapshot();
        if previous.modules().get(spec.id).and_then(|runtime| runtime.manifest_hash)
            == Some(manifest_hash)
        {
            Self::update_health(state, spec.id, health_ok);
            state.registry.update_module(spec.id, |runtime| {
                store_storage(runtime);
            });
            return;
        }
        let manifest = match serde_json::from_slice::<ModuleManifest>(&manifest_bytes) {
            Ok(manifest) => manifest,
            Err(error) => {
                Self::mark_incompatible(state, spec.id, format!("invalid Manifest JSON: {error}"));
                return;
            }
        };

        if let Err(error) = validate_fixed_manifest(&spec, &manifest) {
            Self::mark_incompatible(state, spec.id, error);
            return;
        }

        if let Err(error) =
            PermissionService::reconcile_module_manifest(&state.pool, &manifest).await
        {
            Self::mark_unavailable(state, spec.id, error.to_string());
            return;
        }

        state.registry.update_module(spec.id, |runtime| {
            runtime.condition =
                if health_ok { ModuleCondition::Healthy } else { ModuleCondition::Unavailable };
            runtime.manifest = Some(Arc::new(manifest));
            runtime.manifest_hash = Some(manifest_hash);
            store_storage(runtime);
            if health_ok {
                runtime.last_seen_at = Some(Utc::now());
            }
            runtime.error = (!health_ok).then(|| "health check failed".to_string());
        });
    }

    fn update_health(state: &ModuleControlState, module: &str, healthy: bool) {
        state.registry.update_module(module, |runtime| {
            runtime.condition =
                if healthy { ModuleCondition::Healthy } else { ModuleCondition::Unavailable };
            if healthy {
                runtime.last_seen_at = Some(Utc::now());
            }
            runtime.error = (!healthy).then(|| "health check failed".to_string());
        });
    }

    fn mark_unavailable(state: &ModuleControlState, module: &str, error: String) {
        Self::update_condition(state, module, ModuleCondition::Unavailable, error);
    }

    fn mark_incompatible(state: &ModuleControlState, module: &str, error: String) {
        Self::update_condition(state, module, ModuleCondition::Incompatible, error);
    }

    fn update_condition(
        state: &ModuleControlState,
        module: &str,
        condition: ModuleCondition,
        error: String,
    ) {
        state.registry.update_module(module, |runtime| {
            runtime.condition = condition;
            runtime.error = Some(error);
        });
    }
}

fn validate_fixed_manifest(spec: &ModuleSpec, manifest: &ModuleManifest) -> Result<(), String> {
    manifest.validate().map_err(|error| error.to_string())?;
    if manifest.module != spec.id {
        return Err(format!("Manifest module {} does not match {}", manifest.module, spec.id));
    }
    Ok(())
}

#[cfg(all(test, feature = "full"))]
mod tests;
