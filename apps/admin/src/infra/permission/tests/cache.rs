use super::super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]

async fn concurrent_user_cache_fill_cannot_restore_a_revoked_capability() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let menu_id: i64 = sqlx::query_scalar(
        "INSERT INTO menus (name, code, menu_type, status, is_system, is_manual)
         VALUES ('Concurrent capability', 'custom:concurrent:view', 2, 1, FALSE, TRUE)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("menu");
    let role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status) VALUES ('Concurrent role', 'concurrent_role', 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("role");
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash, status)
         VALUES ('concurrent-user', 'concurrent@example.com', 'hash', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("user");
    sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
        .bind(role_id)
        .bind(menu_id)
        .execute(&pool)
        .await
        .expect("role menu");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(role_id)
        .execute(&pool)
        .await
        .expect("user role");
    PermissionService::refresh_all_user_permissions(&pool).await.expect("initial cache");

    let start = Arc::new(tokio::sync::Barrier::new(2));
    let fill_pool = pool.clone();
    let fill_start = Arc::clone(&start);
    let fill = tokio::spawn(async move {
        fill_start.wait().await;
        for _ in 0..32 {
            PermissionService::refresh_user_permissions(&fill_pool, user_id)
                .await
                .expect("concurrent user fill");
            tokio::task::yield_now().await;
        }
    });
    start.wait().await;
    sqlx::query("DELETE FROM role_menus WHERE role_id = ? AND menu_id = ?")
        .bind(role_id)
        .bind(menu_id)
        .execute(&pool)
        .await
        .expect("revoke capability");
    PermissionService::refresh_all_user_permissions(&pool).await.expect("revoked snapshot");
    fill.await.expect("fill task");

    let user = PermissionService::load_current_user(user_id, "concurrent-user")
        .expect("cached active user");
    assert!(!user.permissions.contains("custom:concurrent:view"));
    PermissionService::clear_user_cache(user_id);
}
