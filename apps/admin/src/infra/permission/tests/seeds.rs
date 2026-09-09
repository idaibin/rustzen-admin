use super::super::*;

#[test]
fn expands_capability_chain_and_dedupes_codes() {
    let codes = vec![
        "system:user:list".to_string(),
        "system:user:create".to_string(),
        "dashboard:view".to_string(),
        "system:user:*".to_string(),
    ];

    let expanded = expand_permission_codes(&codes);

    assert_eq!(
        expanded,
        vec![
            "dashboard:*".to_string(),
            "dashboard:view".to_string(),
            "system:*".to_string(),
            "system:user:*".to_string(),
            "system:user:create".to_string(),
            "system:user:list".to_string(),
        ]
    );
}

#[test]
fn wildcard_capability_uses_clear_display_title() {
    assert_eq!(permission_title("*"), "全部权限");
}

#[test]
fn builds_menu_seed_records_with_capability_titles_and_types() {
    let codes = vec![
        "dashboard:view".to_string(),
        "system:user:*".to_string(),
        "system:user:list".to_string(),
        "report:view".to_string(),
        "manage:log:export".to_string(),
    ];

    let records = build_menu_seed_records(&codes);

    let dashboard_parent = records
        .iter()
        .find(|record| record.permission_code == "dashboard:*")
        .expect("dashboard parent");
    assert_eq!(dashboard_parent.title, "仪表盘管理");
    assert_eq!(dashboard_parent.menu_type, MENU_TYPE_DIRECTORY);
    assert_eq!(dashboard_parent.parent_code, None);

    let dashboard_view = records
        .iter()
        .find(|record| record.permission_code == "dashboard:view")
        .expect("dashboard view");
    assert_eq!(dashboard_view.title, "仪表盘查看");
    assert_eq!(dashboard_view.menu_type, MENU_TYPE_MENU);
    assert_eq!(dashboard_view.parent_code.as_deref(), Some("dashboard:*"));

    let report_view =
        records.iter().find(|record| record.permission_code == "report:view").expect("report view");
    assert_eq!(report_view.title, "报表查看");
    assert_eq!(report_view.menu_type, MENU_TYPE_MENU);
    assert_eq!(report_view.parent_code.as_deref(), Some("report:*"));

    let user_group = records
        .iter()
        .find(|record| record.permission_code == "system:user:*")
        .expect("user group");
    assert_eq!(user_group.title, "系统用户管理");
    assert_eq!(user_group.menu_type, MENU_TYPE_DIRECTORY);
    assert_eq!(user_group.parent_code.as_deref(), Some("system:*"));

    let user_list = records
        .iter()
        .find(|record| record.permission_code == "system:user:list")
        .expect("user list");
    assert_eq!(user_list.title, "系统用户列表");
    assert_eq!(user_list.menu_type, MENU_TYPE_MENU);
    assert_eq!(user_list.parent_code.as_deref(), Some("system:user:*"));

    let log_export = records
        .iter()
        .find(|record| record.permission_code == "manage:log:export")
        .expect("log export");
    assert_eq!(log_export.title, "管理日志导出");
    assert_eq!(log_export.menu_type, MENU_TYPE_BUTTON);
    assert_eq!(log_export.parent_code.as_deref(), Some("manage:log:*"));
}

#[test]
fn seed_records_default_to_system_owned_capabilities() {
    let record = menu_seed_record("system:user:list");

    assert!(!record.is_manual);
    assert!(record.is_system);
}

#[test]
fn builtin_role_policy_keeps_owner_as_only_wildcard_grant() {
    let menu_codes = vec![
        "*".to_string(),
        "system:*".to_string(),
        "system:user:*".to_string(),
        "system:user:list".to_string(),
        "system:user:create".to_string(),
        "dashboard:*".to_string(),
        "dashboard:view".to_string(),
        "manage:*".to_string(),
        "manage:task:*".to_string(),
        "manage:task:list".to_string(),
        "manage:task:run".to_string(),
        "manage:deploy:*".to_string(),
        "manage:deploy:list".to_string(),
        "manage:log:export".to_string(),
    ];

    let owner_codes = builtin_role_permission_codes(BUILTIN_OWNER_ROLE_CODE, &menu_codes);
    let admin_codes = builtin_role_permission_codes(BUILTIN_ADMIN_ROLE_CODE, &menu_codes);
    let viewer_codes = builtin_role_permission_codes(BUILTIN_VIEWER_ROLE_CODE, &menu_codes);

    assert_eq!(owner_codes, vec!["*".to_string()]);

    assert!(admin_codes.contains(&"system:user:create".to_string()));
    assert!(admin_codes.contains(&"manage:log:export".to_string()));
    assert!(admin_codes.contains(&"manage:task:list".to_string()));
    assert!(!admin_codes.contains(&"manage:task:run".to_string()));
    assert!(!admin_codes.contains(&"manage:deploy:list".to_string()));
    assert!(!admin_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
    assert!(!admin_codes.contains(&"manage:deploy:run".to_string()));

    assert!(viewer_codes.contains(&"system:user:list".to_string()));
    assert!(viewer_codes.contains(&"dashboard:view".to_string()));
    assert!(viewer_codes.contains(&"manage:task:list".to_string()));
    assert!(!viewer_codes.contains(&"manage:deploy:list".to_string()));
    assert!(!viewer_codes.contains(&"system:user:create".to_string()));
    assert!(!viewer_codes.contains(&"manage:task:run".to_string()));
    assert!(!viewer_codes.contains(&"manage:log:export".to_string()));
    assert!(!viewer_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
    assert!(!viewer_codes.contains(&"manage:deploy:run".to_string()));
}

#[tokio::test]
async fn sync_permissions_persists_builtin_roles_and_default_owner() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");

    sqlx::query(
        "INSERT INTO menus (
             parent_id, name, code, menu_type, sort_order, status, is_system, is_manual,
             is_active, created_at, updated_at
         ) VALUES
             (0, '过期权限', 'manage:dict:options', 3, 1, 1, TRUE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
             (0, '当前手工覆盖', 'dashboard:view', 3, 1, 1, TRUE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
             (0, '手工权限', 'custom:manual:view', 3, 1, 1, FALSE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .execute(&pool)
    .await
    .expect("legacy and manual permissions");

    let stale_menu_id: i64 =
        sqlx::query_scalar("SELECT id FROM menus WHERE code = 'manage:dict:options'")
            .fetch_one(&pool)
            .await
            .expect("stale menu id");
    let custom_role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status, is_system)
         VALUES ('Legacy dictionary role', 'legacy_dictionary', 1, FALSE)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy custom role");
    let custom_user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash, status)
         VALUES ('legacy-dictionary-user', 'legacy-dictionary@example.com', 'hash', 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy custom user");
    sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
        .bind(custom_role_id)
        .bind(stale_menu_id)
        .execute(&pool)
        .await
        .expect("legacy dictionary grant");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(custom_user_id)
        .bind(custom_role_id)
        .execute(&pool)
        .await
        .expect("legacy dictionary user role");

    let seeded_accounts = sqlx::query_as::<_, (String, String)>(
        "SELECT u.username, r.code
         FROM users u
         INNER JOIN user_roles ur ON ur.user_id = u.id
         INNER JOIN roles r ON r.id = ur.role_id
         WHERE u.is_system = TRUE
         ORDER BY u.username",
    )
    .fetch_all(&pool)
    .await
    .expect("seeded accounts");
    assert_eq!(
        seeded_accounts,
        vec![
            ("admin".to_string(), "admin".to_string()),
            ("owner".to_string(), "owner".to_string()),
            ("viewer".to_string(), "viewer".to_string()),
        ]
    );

    rustzen_auth::permission::register_permission_codes([
        "dashboard:view",
        "system:user:list",
        "system:user:create",
        "manage:task:list",
        "manage:task:run",
        "manage:deploy:list",
        "manage:deploy:run",
    ]);

    PermissionService::sync_permissions(&pool).await.expect("permission sync");

    let owner_permissions = role_permission_codes(&pool, BUILTIN_OWNER_ROLE_CODE).await;
    let admin_permissions = role_permission_codes(&pool, BUILTIN_ADMIN_ROLE_CODE).await;
    let viewer_permissions = role_permission_codes(&pool, BUILTIN_VIEWER_ROLE_CODE).await;

    assert_eq!(owner_permissions, vec!["*".to_string()]);
    assert!(admin_permissions.contains(&"system:user:create".to_string()));
    assert!(admin_permissions.contains(&"manage:task:list".to_string()));
    assert!(!admin_permissions.contains(&"manage:task:run".to_string()));
    assert!(!admin_permissions.contains(&"manage:deploy:list".to_string()));
    assert!(!admin_permissions.contains(&"manage:deploy:run".to_string()));
    assert!(!admin_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

    assert!(viewer_permissions.contains(&"dashboard:view".to_string()));
    assert!(viewer_permissions.contains(&"system:user:list".to_string()));
    assert!(viewer_permissions.contains(&"manage:task:list".to_string()));
    assert!(!viewer_permissions.contains(&"manage:deploy:list".to_string()));
    assert!(!viewer_permissions.contains(&"system:user:create".to_string()));
    assert!(!viewer_permissions.contains(&"manage:task:run".to_string()));
    assert!(!viewer_permissions.contains(&"manage:deploy:run".to_string()));
    assert!(!viewer_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

    let stale_core_active: bool =
        sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'manage:dict:options'")
            .fetch_one(&pool)
            .await
            .expect("stale core permission");
    let manual_active: bool =
        sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'custom:manual:view'")
            .fetch_one(&pool)
            .await
            .expect("manual permission");
    let current_core_active: bool =
        sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'dashboard:view'")
            .fetch_one(&pool)
            .await
            .expect("current core permission");
    assert!(!stale_core_active);
    assert!(manual_active);
    assert!(current_core_active);
    let current_core_name: String =
        sqlx::query_scalar("SELECT name FROM menus WHERE code = 'dashboard:view'")
            .fetch_one(&pool)
            .await
            .expect("current manual override");
    assert_eq!(current_core_name, "当前手工覆盖");

    let stale_effective_permissions = sqlx::query_scalar::<_, String>(
        "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
    )
    .bind(custom_user_id)
    .fetch_all(&pool)
    .await
    .expect("legacy dictionary effective permissions");
    assert!(stale_effective_permissions.is_empty());
    let cached_user =
        PermissionService::load_current_user(custom_user_id, "legacy-dictionary-user")
            .expect("legacy user permission cache");
    assert!(!cached_user.has_capability("manage:dict:options"));
    PermissionService::clear_user_cache(custom_user_id);

    let owner_wildcard_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM user_permissions
         WHERE username = ? AND menu_code = ?",
    )
    .bind(DEFAULT_OWNER_USERNAME)
    .bind(SYSTEM_WILDCARD)
    .fetch_one(&pool)
    .await
    .expect("owner permissions");
    assert_eq!(owner_wildcard_count, 1);
}

pub(super) async fn role_permission_codes(pool: &SqlitePool, role_code: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT m.code
         FROM roles r
         INNER JOIN role_menus rm ON rm.role_id = r.id
         INNER JOIN menus m ON m.id = rm.menu_id
         WHERE r.code = ?
           AND r.deleted_at IS NULL
           AND m.deleted_at IS NULL
         ORDER BY m.code",
    )
    .bind(role_code)
    .fetch_all(pool)
    .await
    .expect("role permission codes")
}

#[test]
fn load_current_user_marks_super_from_cached_capability_wildcard() {
    PermissionService::cache_user_permissions(42, &["*".to_string()]);

    let user = PermissionService::load_current_user(42, "root").expect("cached user");

    assert_eq!(user.user_id, 42);
    assert_eq!(user.username, "root");
    assert!(user.is_super);
    assert!(user.permissions.contains("*"));

    PermissionService::clear_user_cache(42);
}
