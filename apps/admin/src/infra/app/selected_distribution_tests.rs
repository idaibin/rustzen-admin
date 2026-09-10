use super::*;
use crate::features::modules::types::ModuleSpec;
use crate::infra::{
    auth_runtime::{ServerAuthContextLoader, jwt_codec},
    db::run_migrations,
    permission::PermissionService,
};
use axum::{body::to_bytes, middleware};
use rustzen_auth::auth::auth_middleware;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn authenticated_token(pool: &SqlitePool, user_id: i64, username: &str) -> String {
    let epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("user auth epoch");
    let sid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let codec = jwt_codec();
    let claims = codec.claims_at(user_id, username, &sid, epoch, now);
    crate::features::auth::session::SessionRepository::create(
        pool,
        user_id,
        &sid,
        epoch,
        now,
        claims.exp as i64,
    )
    .await
    .expect("access session");
    codec.encode_claims(&claims).expect("session token")
}

#[test]
fn route_inventory_contains_access_and_selected_navigation_only() {
    let contracts = documented_all_contracts();
    let paths = contracts.iter().map(|contract| contract.path.as_str()).collect::<Vec<_>>();
    for required in [
        "/__web-binding",
        "/api/auth/login",
        "/api/auth/me",
        "/api/installation",
        "/api/account/profile",
        "/api/system/users",
        "/api/system/roles",
        "/api/system/menus/options",
        "/api/system/modules/navigation",
    ] {
        assert!(paths.contains(&required), "missing {required}");
    }
    assert!(
        paths
            .iter()
            .filter(|path| path.starts_with("/api/system/menus"))
            .all(|path| *path == "/api/system/menus/options"),
        "selected distribution must expose only the permission-options menu route"
    );
    for excluded_prefix in [
        "/api/manage/",
        "/api/dashboard/",
        "/api/system/status",
        "/api/system/modules/{module}/enabled",
        "/api/account/avatar",
    ] {
        assert!(
            paths.iter().all(|path| !path.starts_with(excluded_prefix)),
            "unexpected route under {excluded_prefix}"
        );
    }
    #[cfg(not(feature = "notifications"))]
    assert!(
        paths.iter().all(|path| !path.starts_with("/api/notifications")),
        "pure monitor distribution must omit notification routes"
    );
    #[cfg(feature = "notifications")]
    assert_eq!(
        paths.iter().filter(|path| path.starts_with("/api/notifications")).count(),
        6,
        "monitor-notify must expose the exact notification route owner"
    );
    assert_eq!(
        ModuleSpec::fixed().iter().map(|module| module.id).collect::<Vec<_>>(),
        if cfg!(feature = "analytics-distribution") { ["insights"] } else { ["monitor"] }
    );
}

#[tokio::test]
async fn omitted_api_paths_do_not_fall_through_to_the_selected_web_shell() {
    let response = crate::infra::web::serve("/api/manage/tasks".parse().expect("uri")).await;
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[axum::http::header::CONTENT_TYPE], "application/json");
    let body = to_bytes(response.into_body(), usize::MAX).await.expect("response body");
    assert!(!body.windows(5).any(|window| window == b"<html"));
}

#[tokio::test]
async fn role_management_keeps_the_permission_catalogue_and_immediately_grants_its_selection() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    let (routes, contracts) = documented_protected_routes();
    sqlx::query("UPDATE users SET status = 1 WHERE username = 'owner'")
        .execute(&pool)
        .await
        .expect("enable test owner");
    let permission_codes = super::routes::contract_permission_codes(contracts.iter());
    PermissionService::sync_permission_codes(&pool, &permission_codes)
        .await
        .expect("isolated permission catalogue");
    let app = routes
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(pool.clone());
    let owner = authenticated_token(&pool, 1, "owner").await;

    let options = app
        .clone()
        .oneshot(
            axum::http::Request::get("/api/system/menus/options")
                .header("authorization", format!("Bearer {owner}"))
                .body(axum::body::Body::empty())
                .expect("options request"),
        )
        .await
        .expect("options response");
    assert_eq!(options.status(), axum::http::StatusCode::OK);
    let options: serde_json::Value = serde_json::from_slice(
        &to_bytes(options.into_body(), usize::MAX).await.expect("options body"),
    )
    .expect("options JSON");
    let permission_id = options["data"]
        .as_array()
        .expect("option list")
        .iter()
        .find(|option| option["code"] == "system:menu:options")
        .and_then(|option| option["value"].as_i64())
        .expect("assignable permission option");

    let create_role = app
        .clone()
        .oneshot(
            axum::http::Request::post("/api/system/roles")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "application/json")
                .body(axum::body::Body::from(format!(
                    r#"{{"name":"Permission reader","code":"permission_reader","status":1,"menuIds":[{permission_id}],"description":null}}"#
                )))
                .expect("create role request"),
        )
        .await
        .expect("create role response");
    assert_eq!(create_role.status(), axum::http::StatusCode::OK);
    let role_id: i64 = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'permission_reader'")
        .fetch_one(&pool)
        .await
        .expect("created role");

    let create_user = app
        .clone()
        .oneshot(
            axum::http::Request::post("/api/system/users")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "application/json")
                .body(axum::body::Body::from(format!(
                    r#"{{"username":"permission_reader","email":"permission-reader@example.com","password":"ValidPassw0rd!","realName":"Permission Reader","roleIds":[{role_id}]}}"#
                )))
                .expect("create user request"),
        )
        .await
        .expect("create user response");
    assert_eq!(create_user.status(), axum::http::StatusCode::OK);
    let user_id: i64 = serde_json::from_slice::<serde_json::Value>(
        &to_bytes(create_user.into_body(), usize::MAX).await.expect("user body"),
    )
    .expect("user JSON")["data"]
        .as_i64()
        .expect("created user ID");
    let reader = authenticated_token(&pool, user_id, "permission_reader").await;
    let reader_options = app
        .clone()
        .oneshot(
            axum::http::Request::get("/api/system/menus/options")
                .header("authorization", format!("Bearer {reader}"))
                .body(axum::body::Body::empty())
                .expect("reader options request"),
        )
        .await
        .expect("reader options response");
    assert_eq!(reader_options.status(), axum::http::StatusCode::OK);

    for request in [
        axum::http::Request::get("/api/system/menus")
            .header("authorization", format!("Bearer {owner}"))
            .body(axum::body::Body::empty())
            .expect("menu list request"),
        axum::http::Request::builder()
            .method(axum::http::Method::PUT)
            .uri("/api/system/menus/inventory/1")
            .header("authorization", format!("Bearer {owner}"))
            .header("content-type", "application/json")
            .body(axum::body::Body::from("{}"))
            .expect("menu write request"),
    ] {
        let response = app.clone().oneshot(request).await.expect("menu response");
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
