use super::{super::*, support::monitor_manifest};
use tokio::sync::Barrier;

use crate::features::system::menu::{service::MenuService, types::UpdateMenuPayload};

#[tokio::test]
async fn module_navigation_preserves_multiple_pages_with_one_capability() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::infra::db::run_migrations(&pool).await.unwrap();
    let mut manifest = monitor_manifest(
        &["monitor:view"],
        "monitor:view",
        "Overview",
        "/monitoring/overview",
        "monitor",
        10,
    );
    let mut settings = manifest.menus[0].clone();
    settings.code = "settings".into();
    settings.title = "Settings".into();
    settings.path = "/monitoring/settings".into();
    manifest.menus.push(settings);
    PermissionService::reconcile_module_manifest(&pool, &manifest).await.unwrap();
    let inventory = MenuService::list_module_menu_inventory(&pool).await.unwrap();
    assert_eq!(inventory.len(), 2, "navigation identity must not collapse by capability");
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM menus WHERE code='monitor:view' AND is_active=TRUE",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "one assignable capability for both pages");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn module_menu_edits_never_succeed_against_a_removed_navigation_row() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::infra::db::run_migrations(&pool).await.unwrap();
    let initial =
        monitor_manifest(&["monitor:view"], "monitor:view", "Monitor", "/monitor", "monitor", 10);
    PermissionService::reconcile_module_manifest(&pool, &initial).await.unwrap();
    let id = MenuService::list_module_menu_inventory(&pool).await.unwrap()[0].id;
    let mut removed = initial;
    removed.menus.clear();
    let request =
        UpdateMenuPayload { name: "Custom".into(), sort_order: 42, status: 2, icon: None };
    let start = Arc::new(Barrier::new(3));
    let edit_pool = pool.clone();
    let edit_start = start.clone();
    let edit_request = request.clone();
    let edit = tokio::spawn(async move {
        edit_start.wait().await;
        MenuService::update_menu(&edit_pool, id, edit_request).await
    });
    let sync_pool = pool.clone();
    let sync_start = start.clone();
    let sync = tokio::spawn(async move {
        sync_start.wait().await;
        PermissionService::reconcile_module_manifest(&sync_pool, &removed).await
    });
    start.wait().await;
    let edited = edit.await.unwrap();
    sync.await.unwrap().unwrap();
    assert!(edited.is_ok() || matches!(edited, Err(ServiceError::NotFound(_))));
    assert!(MenuService::list_module_menu_inventory(&pool).await.unwrap().is_empty());
    assert!(matches!(
        MenuService::update_menu(&pool, id, request).await,
        Err(ServiceError::NotFound(_))
    ));
    let permissions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM menus WHERE code='monitor:view' AND is_active=TRUE",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(permissions, 1, "removing navigation must not revoke its surviving API capability");
}

#[tokio::test]
async fn module_menu_names_are_scoped_independently_from_manual_menus() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    sqlx::query(
        "INSERT INTO menus (name, code, menu_type, status, is_system, is_manual)
         VALUES ('Monitor', 'custom:monitor:list', 2, 1, FALSE, TRUE)",
    )
    .execute(&pool)
    .await
    .expect("conflicting manual menu");

    let manifest =
        monitor_manifest(&["monitor:view"], "monitor:view", "Monitor", "/monitor", "monitor", 10);
    PermissionService::reconcile_module_manifest(&pool, &manifest)
        .await
        .expect("module menu may reuse a title in its own navigation group");
    let module_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM menus WHERE module_id = 'monitor'")
            .fetch_one(&pool)
            .await
            .expect("module rows after rollback");
    assert_eq!(module_rows, 1);
    let custom_active: bool =
        sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'custom:monitor:list'")
            .fetch_one(&pool)
            .await
            .expect("custom menu remains");
    assert!(custom_active);
}
