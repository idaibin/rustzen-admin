use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{Json, Router, extract::State, routing::get};
use rustzen_auth::auth::CurrentUser;
use rustzen_ipc::{AccessMode, DelegationSigner, MenuDefinition, ModuleManifest, RouteManifest};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::{Barrier, Notify};

use super::{ModuleControlState, ModuleService};
use crate::{
    features::modules::{
        registry::ModuleRegistry,
        types::{GatewayLookup, ModuleCondition, ModuleSpec},
    },
    infra::permission::PermissionService,
};

#[tokio::test]
async fn navigation_keeps_persisted_enabled_menus_visible_without_a_runtime_manifest() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    PermissionService::reconcile_module_manifest(&pool, &test_manifest(false))
        .await
        .expect("persist module manifest");
    let state = ModuleControlState {
        pool,
        registry: ModuleRegistry::new(
            vec![ModuleSpec {
                id: "monitor",
                name: "Monitor",
                base_url: "http://127.0.0.1:9802".to_string(),
            }],
            &BTreeMap::new(),
        ),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };
    let authorized = CurrentUser::new(1, "user", ["monitor:view".to_string()], false);

    for condition in [ModuleCondition::Unavailable, ModuleCondition::Incompatible] {
        state.registry.update_module("monitor", |runtime| {
            runtime.condition = condition;
            assert!(runtime.manifest.is_none());
        });
        let navigation = ModuleService::navigation(&state, &authorized).await.expect("navigation");
        assert_eq!(navigation.len(), 1, "{condition:?} module keeps persisted menu visible");
        assert_eq!(navigation[0].module_name, "Monitor");
    }

    let unauthorized = CurrentUser::new(2, "user", Vec::new(), false);
    assert!(ModuleService::navigation(&state, &unauthorized).await.expect("navigation").is_empty());

    sqlx::query("UPDATE module_navigation SET status = 2 WHERE module_id = 'monitor'")
        .execute(&state.pool)
        .await
        .expect("hide menu");
    assert!(ModuleService::navigation(&state, &authorized).await.expect("navigation").is_empty());
    sqlx::query("UPDATE module_navigation SET status = 1 WHERE module_id = 'monitor'")
        .execute(&state.pool)
        .await
        .expect("restore menu");

    ModuleService::set_enabled(&state, "monitor", false).await.expect("disable");
    assert!(ModuleService::navigation(&state, &authorized).await.expect("navigation").is_empty());
}

#[tokio::test]
async fn status_and_dashboard_health_follow_fixed_module_order() {
    let state = ModuleControlState {
        pool: SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("pool"),
        registry: ModuleRegistry::new(ModuleSpec::fixed(), &BTreeMap::new()),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };
    let expected = ["monitor", "insights", "reports"];
    assert_eq!(ModuleSpec::fixed().into_iter().map(|spec| spec.id).collect::<Vec<_>>(), expected);
    assert_eq!(
        ModuleService::statuses(&state).into_iter().map(|status| status.id).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        ModuleService::dashboard_health(&state)
            .into_iter()
            .map(|health| health.module)
            .collect::<Vec<_>>(),
        expected
    );
}

#[tokio::test]
async fn changed_manifest_swaps_after_commit_and_invalid_change_rolls_back() {
    let manifest = Arc::new(RwLock::new(test_manifest(false)));
    let upstream = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }))
        .route("/internal/v1/manifest", get(manifest_handler))
        .with_state(Arc::clone(&manifest));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        axum::serve(listener, upstream).await.expect("serve");
    });

    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let spec = ModuleSpec { id: "monitor", name: "Monitor", base_url: format!("http://{address}") };
    let state = ModuleControlState {
        pool: pool.clone(),
        registry: ModuleRegistry::new(vec![spec], &BTreeMap::new()),
        client: reqwest::Client::builder().timeout(Duration::from_secs(1)).build().expect("client"),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };

    assert_eq!(
        state.registry.snapshot().lookup(&axum::http::Method::GET, "/api/monitor/nodes").0,
        GatewayLookup::ServiceUnavailable
    );
    ModuleService::sync_once(&state).await;
    assert_eq!(
        state.registry.snapshot().lookup(&axum::http::Method::GET, "/api/monitor/nodes").0,
        GatewayLookup::Found
    );

    *manifest.write().expect("manifest write") = test_manifest(true);
    ModuleService::sync_once(&state).await;
    assert_eq!(
        state.registry.snapshot().lookup(&axum::http::Method::POST, "/api/monitor/restart").0,
        GatewayLookup::Found
    );
    let active_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM menus WHERE module_id = 'monitor' AND is_active = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("active before invalid");

    let mut invalid = test_manifest(true);
    invalid.contract_version = rustzen_ipc::CONTRACT_VERSION + 1;
    invalid.routes.push(RouteManifest {
        method: "DELETE".to_string(),
        path: "/invalid".to_string(),
        access: AccessMode::Protected,
        permission: Some("monitor:delete".to_string()),
    });
    *manifest.write().expect("invalid manifest write") = invalid;
    ModuleService::sync_once(&state).await;
    let snapshot = state.registry.snapshot();
    let runtime = snapshot.modules().get("monitor").expect("monitor runtime");
    assert_eq!(runtime.condition, ModuleCondition::Incompatible);
    assert!(runtime.manifest.as_ref().is_some_and(|manifest| {
        manifest.routes.iter().all(|route| route.permission.as_deref() != Some("monitor:delete"))
    }));
    assert_eq!(
        snapshot.lookup(&axum::http::Method::GET, "/api/monitor/nodes").0,
        GatewayLookup::ServiceUnavailable
    );
    let active_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM menus WHERE module_id = 'monitor' AND is_active = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("active after invalid");
    assert_eq!(active_after, active_before);
    let invalid_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM menus WHERE code = 'monitor:delete'")
            .fetch_one(&pool)
            .await
            .expect("invalid capability count");
    assert_eq!(invalid_count, 0);

    *manifest.write().expect("restore manifest write") = test_manifest(true);
    ModuleService::sync_once(&state).await;
    let last_seen = state
        .registry
        .snapshot()
        .modules()
        .get("monitor")
        .and_then(|runtime| runtime.last_seen_at)
        .expect("healthy last seen");
    server.abort();
    let _ = server.await;
    ModuleService::sync_once(&state).await;
    let snapshot = state.registry.snapshot();
    let runtime = snapshot.modules().get("monitor").expect("unavailable monitor");
    assert_eq!(runtime.condition, ModuleCondition::Unavailable);
    assert!(runtime.compatible(), "temporary outage must keep the last-known-good menu state");
    assert_eq!(runtime.last_seen_at, Some(last_seen));
    let dashboard = ModuleService::dashboard_health(&state);
    assert_eq!(dashboard[0].module, "monitor");
    assert!(!dashboard[0].available);
    assert_eq!(dashboard[0].release_version, None);

    let mut recovered_manifest = test_manifest(true);
    recovered_manifest.routes.push(RouteManifest {
        method: "POST".to_string(),
        path: "/recover".to_string(),
        access: AccessMode::Protected,
        permission: Some("monitor:recover".to_string()),
    });
    *manifest.write().expect("recovered manifest write") = recovered_manifest;
    let restarted_listener =
        tokio::net::TcpListener::bind(address).await.expect("rebind restarted service");
    let restarted = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }))
        .route("/internal/v1/manifest", get(manifest_handler))
        .with_state(Arc::clone(&manifest));
    let restarted_server = tokio::spawn(async move {
        axum::serve(restarted_listener, restarted).await.expect("serve restarted service");
    });
    ModuleService::sync_once(&state).await;
    assert_eq!(
        state.registry.snapshot().lookup(&axum::http::Method::POST, "/api/monitor/recover").0,
        GatewayLookup::Found
    );
    let recovered_capability: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM menus WHERE code = 'monitor:recover' AND is_active = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("recovered capability");
    assert_eq!(recovered_capability, 1);
    restarted_server.abort();
}

#[tokio::test]
async fn disabled_modules_are_not_polled_and_reenable_waits_for_a_fresh_manifest() {
    let manifest = Arc::new(RwLock::new(test_manifest(false)));
    let upstream = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }))
        .route("/internal/v1/manifest", get(manifest_handler))
        .with_state(manifest);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        axum::serve(listener, upstream).await.expect("serve");
    });

    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let spec = ModuleSpec { id: "monitor", name: "Monitor", base_url: format!("http://{address}") };
    let state = ModuleControlState {
        pool,
        registry: ModuleRegistry::new(vec![spec], &BTreeMap::new()),
        client: reqwest::Client::builder().build().expect("client"),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };

    ModuleService::set_enabled(&state, "monitor", false).await.expect("disable");
    ModuleService::sync_once(&state).await;
    let snapshot = state.registry.snapshot();
    let runtime = snapshot.modules().get("monitor").expect("disabled monitor");
    assert!(!runtime.enabled);
    assert!(runtime.manifest.is_none());

    ModuleService::set_enabled(&state, "monitor", true).await.expect("enable");
    let snapshot = state.registry.snapshot();
    let runtime = snapshot.modules().get("monitor").expect("enabled monitor");
    assert_eq!(runtime.condition, ModuleCondition::Unavailable);
    assert_eq!(runtime.error.as_deref(), Some("awaiting Manifest refresh"));
    assert_eq!(
        snapshot.lookup(&axum::http::Method::GET, "/api/monitor/nodes").0,
        GatewayLookup::ServiceUnavailable
    );

    ModuleService::sync_once(&state).await;
    assert_eq!(
        state.registry.snapshot().lookup(&axum::http::Method::GET, "/api/monitor/nodes").0,
        GatewayLookup::Found
    );
    server.abort();
}

#[tokio::test]
async fn concurrent_enabled_updates_keep_database_and_registry_in_sync() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let state = ModuleControlState {
        pool: pool.clone(),
        registry: ModuleRegistry::new(ModuleSpec::fixed(), &BTreeMap::new()),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };

    for _ in 0..32 {
        let barrier = Arc::new(Barrier::new(3));
        let disable_state = state.clone();
        let disable_barrier = Arc::clone(&barrier);
        let disable = tokio::spawn(async move {
            disable_barrier.wait().await;
            ModuleService::set_enabled(&disable_state, "monitor", false).await.expect("disable");
        });
        let enable_state = state.clone();
        let enable_barrier = Arc::clone(&barrier);
        let enable = tokio::spawn(async move {
            enable_barrier.wait().await;
            ModuleService::set_enabled(&enable_state, "monitor", true).await.expect("enable");
        });

        barrier.wait().await;
        disable.await.expect("disable task");
        enable.await.expect("enable task");

        let stored: bool = sqlx::query_scalar("SELECT enabled FROM modules WHERE id = 'monitor'")
            .fetch_one(&pool)
            .await
            .expect("stored state");
        let cached =
            state.registry.snapshot().modules().get("monitor").expect("monitor runtime").enabled;
        assert_eq!(cached, stored);
    }
}

#[derive(Clone)]
struct DelayedManifest {
    started: Arc<Notify>,
    release: Arc<Notify>,
    manifest: ModuleManifest,
}

#[tokio::test]
async fn disabling_during_sync_cannot_be_overwritten_by_the_manifest_swap() {
    let delayed = DelayedManifest {
        started: Arc::new(Notify::new()),
        release: Arc::new(Notify::new()),
        manifest: test_manifest(false),
    };
    let upstream = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }))
        .route("/internal/v1/manifest", get(delayed_manifest_handler))
        .with_state(delayed.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        axum::serve(listener, upstream).await.expect("serve");
    });

    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let state = ModuleControlState {
        pool,
        registry: ModuleRegistry::new(
            vec![ModuleSpec {
                id: "monitor",
                name: "Monitor",
                base_url: format!("http://{address}"),
            }],
            &BTreeMap::new(),
        ),
        client: reqwest::Client::builder().build().expect("client"),
        signer: DelegationSigner::new("secret").expect("signer"),
        enabled_update: Arc::default(),
    };

    let sync_state = state.clone();
    let sync = tokio::spawn(async move { ModuleService::sync_once(&sync_state).await });
    delayed.started.notified().await;
    ModuleService::set_enabled(&state, "monitor", false).await.expect("disable during sync");
    delayed.release.notify_one();
    sync.await.expect("sync task");
    let snapshot = state.registry.snapshot();
    let runtime = snapshot.modules().get("monitor").expect("monitor runtime");
    assert!(!runtime.enabled);
    assert!(runtime.manifest.is_some());
    assert!(!runtime.available());
    server.abort();
}

async fn delayed_manifest_handler(State(state): State<DelayedManifest>) -> Json<ModuleManifest> {
    state.started.notify_one();
    state.release.notified().await;
    Json(state.manifest)
}

async fn manifest_handler(
    State(manifest): State<Arc<RwLock<ModuleManifest>>>,
) -> Json<ModuleManifest> {
    Json(manifest.read().expect("manifest read").clone())
}

fn test_manifest(changed: bool) -> ModuleManifest {
    let mut routes = vec![RouteManifest {
        method: "GET".to_string(),
        path: "/nodes".to_string(),
        access: AccessMode::Protected,
        permission: Some("monitor:view".to_string()),
    }];
    if changed {
        routes.push(RouteManifest {
            method: "POST".to_string(),
            path: "/restart".to_string(),
            access: AccessMode::Protected,
            permission: Some("monitor:manage".to_string()),
        });
    }
    ModuleManifest {
        module: "monitor".to_string(),
        name: "Monitor".to_string(),
        api_prefix: "/api/monitor".to_string(),
        contract_version: rustzen_ipc::CONTRACT_VERSION,
        release_version: env!("CARGO_PKG_VERSION").to_string(),
        menus: vec![MenuDefinition {
            code: "monitor".to_string(),
            title: "Monitor".to_string(),
            path: "/monitor".to_string(),
            icon: "monitor".to_string(),
            sort_order: 10,
            permission: "monitor:view".to_string(),
        }],
        routes,
    }
}
