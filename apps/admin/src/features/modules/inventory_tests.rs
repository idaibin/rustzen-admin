//! Composition boundary tests that run under every Admin composition.
//!
//! The expectations derive from `ModuleSpec::installed()`: a `modules` build
//! installs the three example modules, a core build installs none. The
//! assertions therefore stay valid in both compositions instead of pinning
//! the full inventory.

use std::{collections::BTreeMap, sync::Arc};

use rustzen_ipc::DelegationSigner;
use sqlx::sqlite::SqlitePoolOptions;

use crate::features::modules::{
    registry::ModuleRegistry,
    service::{ModuleControlState, ModuleService},
    types::ModuleSpec,
};

async fn state_with_installed_inventory() -> ModuleControlState {
    ModuleControlState {
        pool: SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("pool"),
        registry: ModuleRegistry::new(ModuleSpec::installed(), &BTreeMap::new()),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("inventory-test-key").expect("signer"),
        enabled_update: Arc::default(),
    }
}

fn expected_installed_ids() -> Vec<&'static str> {
    #[cfg(feature = "modules")]
    { vec!["monitor", "insights", "reports"] }
    #[cfg(not(feature = "modules"))]
    { Vec::new() }
}

#[test]
fn installed_inventory_matches_the_build_composition() {
    let ids =
        ModuleSpec::installed().into_iter().map(|spec| spec.id).collect::<Vec<&'static str>>();
    assert_eq!(ids, expected_installed_ids());
}

#[tokio::test]
async fn statuses_and_dashboard_follow_the_installed_inventory() {
    let state = state_with_installed_inventory().await;
    let expected = expected_installed_ids();
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
async fn mutating_outside_the_installed_inventory_is_rejected_before_any_write() {
    // The migration seeds dormant module rows even in a core build; changing
    // one must be rejected before the database write, not after it.
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    // The modules build installs every seeded row (the schema's CHECK
    // constraint even limits ids to the three examples), so its rejection
    // case is a registry that installs a subset of the seeded rows.
    #[cfg(feature = "modules")]
    let registry_specs = ModuleSpec::installed()
        .into_iter()
        .filter(|spec| spec.id != "insights")
        .collect::<Vec<_>>();
    #[cfg(not(feature = "modules"))]
    let registry_specs = ModuleSpec::installed();
    let state = ModuleControlState {
        pool: pool.clone(),
        registry: ModuleRegistry::new(registry_specs, &BTreeMap::new()),
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("inventory-test-key").expect("signer"),
        enabled_update: Arc::default(),
    };
    #[cfg(feature = "modules")]
    let module = "insights";
    #[cfg(not(feature = "modules"))]
    let module = "monitor";
    let before = sqlx::query_scalar::<_, i64>("SELECT enabled FROM modules WHERE id = ?")
        .bind(module)
        .fetch_optional(&pool)
        .await
        .expect("seeded module row lookup");
    assert_eq!(before, Some(1), "the attempt must target an enabled row");
    // Disabling is a real state change from the seed value, so a
    // write-then-fail implementation would be exposed by the stored value.
    let outcome = ModuleService::set_enabled(&state, module, false).await;
    assert!(outcome.is_err(), "disabling {module} must be rejected");
    let stored = sqlx::query_scalar::<_, i64>("SELECT enabled FROM modules WHERE id = ?")
        .bind(module)
        .fetch_optional(&pool)
        .await
        .expect("seeded module row lookup");
    assert_eq!(stored, before, "the rejected mutation must not modify the dormant seed row");
}
