use super::{super::*, seeds::role_permission_codes, support::monitor_manifest};

#[tokio::test]
async fn module_reconciliation_preserves_manual_overrides_and_custom_grants() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");

    let initial = monitor_manifest(
        &["monitor:view", "monitor:manage"],
        "monitor:view",
        "Monitor",
        "/monitor",
        "monitor",
        10,
    );
    PermissionService::reconcile_module_manifest(&pool, &initial).await.expect("initial reconcile");
    let wildcard_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM menus WHERE code = 'monitor:*'")
            .fetch_one(&pool)
            .await
            .expect("wildcard count");
    assert_eq!(wildcard_count, 0, "Manifest reconciliation must persist exact capabilities");

    let view_id: i64 = sqlx::query_scalar("SELECT id FROM menus WHERE code = 'monitor:view'")
        .fetch_one(&pool)
        .await
        .expect("view menu");
    let custom_role: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status) VALUES ('Monitor custom', 'monitor_custom', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("custom role");
    sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
        .bind(custom_role)
        .bind(view_id)
        .execute(&pool)
        .await
        .expect("custom grant");
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash, status) VALUES ('module-user', 'module@example.com', 'hash', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("custom user");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(custom_role)
        .execute(&pool)
        .await
        .expect("user role");
    let navigation_id: i64 =
        sqlx::query_scalar("SELECT id FROM module_navigation WHERE module_id='monitor'")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query(
        "UPDATE module_navigation SET name = 'Custom Monitor', icon = 'custom-icon', sort_order = 77,
                status = 2, is_manual = TRUE WHERE id = ?",
    )
    .bind(navigation_id)
    .execute(&pool)
    .await
    .expect("manual override");
    crate::features::system::menu::repo::MenuRepository::update_navigation(
        &pool,
        navigation_id,
        "Custom Monitor",
        None,
        77,
        2,
    )
    .await
    .expect("null icon preserves the current presentation");

    let changed = monitor_manifest(
        &["monitor:view", "monitor:manage", "monitor:restart"],
        "monitor:view",
        "Changed default",
        "/monitor-v2",
        "changed-icon",
        1,
    );
    PermissionService::reconcile_module_manifest(&pool, &changed).await.expect("changed reconcile");
    let row: (String, Option<String>, i32, i16, Option<String>, bool, bool) = sqlx::query_as(
        "SELECT name, icon, sort_order, status, path, is_manual, is_active
             FROM module_navigation WHERE id = ?",
    )
    .bind(navigation_id)
    .fetch_one(&pool)
    .await
    .expect("manual row");
    assert_eq!(
        row,
        (
            "Custom Monitor".to_string(),
            Some("custom-icon".to_string()),
            77,
            2,
            Some("/monitor-v2".to_string()),
            true,
            true,
        )
    );
    let permission_while_hidden: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_permissions WHERE user_id = ? AND menu_code = 'monitor:view'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("hidden permission");
    assert_eq!(permission_while_hidden, 1, "menu visibility is not authorization");
    let custom_restart: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM role_menus rm
         INNER JOIN menus m ON m.id = rm.menu_id
         WHERE rm.role_id = ? AND m.code = 'monitor:restart'",
    )
    .bind(custom_role)
    .fetch_one(&pool)
    .await
    .expect("new custom grant");
    assert_eq!(custom_restart, 0, "new capabilities must not reach custom roles");
    assert!(
        role_permission_codes(&pool, BUILTIN_ADMIN_ROLE_CODE)
            .await
            .contains(&"monitor:restart".to_string())
    );
    assert!(
        !role_permission_codes(&pool, BUILTIN_VIEWER_ROLE_CODE)
            .await
            .contains(&"monitor:restart".to_string())
    );

    let removed = monitor_manifest(
        &["monitor:manage", "monitor:restart"],
        "monitor:manage",
        "Monitor",
        "/monitor",
        "monitor",
        10,
    );
    PermissionService::reconcile_module_manifest(&pool, &removed).await.expect("removed reconcile");
    let active: bool = sqlx::query_scalar("SELECT is_active FROM menus WHERE id = ?")
        .bind(view_id)
        .fetch_one(&pool)
        .await
        .expect("inactive old capability");
    assert!(!active);
    let retained_grant: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM role_menus WHERE role_id = ? AND menu_id = ?")
            .bind(custom_role)
            .bind(view_id)
            .fetch_one(&pool)
            .await
            .expect("retained grant");
    assert_eq!(retained_grant, 1);
    let effective_grant: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_permissions WHERE user_id = ? AND menu_code = 'monitor:view'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("effective removed grant");
    assert_eq!(effective_grant, 0);
    let transferred: (String, Option<String>, i32, i16, Option<String>, bool) = sqlx::query_as(
        "SELECT name, icon, sort_order, status, path, is_manual
             FROM module_navigation WHERE code = 'monitor:manage' AND is_active = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("transferred module menu override");
    assert_eq!(
        transferred,
        (
            "Custom Monitor".to_string(),
            Some("custom-icon".to_string()),
            77,
            2,
            Some("/monitor".to_string()),
            true,
        )
    );
}
