#[cfg(feature = "notifications")]
use crate::features::notifications::{
    admission_types::AdmissionPolicy, ingress, maintenance, notification_routes,
};
#[cfg(any(feature = "full", test))]
use crate::infra::db::run_migrations;
use crate::{
    features::{
        account::account_routes,
        auth::{protected_auth_contract_routes, public_auth_routes},
        modules::{
            control_routes, gateway,
            service::{ModuleControlState, ModuleService},
        },
        system::system_contract_routes,
    },
    infra::{
        auth_runtime::{ServerAuthContextLoader, jwt_codec},
        config::CONFIG,
        db::{create_default_pool, test_connection},
        permission::PermissionService,
    },
};
#[cfg(feature = "full")]
use crate::{
    features::{
        dashboard::dashboard_routes,
        manage::{deploy::service::DeployService, manage_routes, task::service::TaskService},
    },
    middleware::log::log_middleware,
};

#[cfg(feature = "full")]
use axum::Extension;
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    middleware,
    routing::get,
};
use rustzen_auth::auth::auth_middleware;
use rustzen_ipc::{DelegationSigner, HealthResponse};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
#[cfg(feature = "full")]
use tower_http::services::ServeDir;

#[tracing::instrument(name = "run_server")]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database connection pool...");
    #[cfg(feature = "monitor-distribution")]
    crate::infra::db::verify_selected_database().await.map_err(std::io::Error::other)?;
    let pool = create_default_pool().await?;
    #[cfg(feature = "full")]
    run_migrations(&pool).await?;
    test_connection(&pool).await?;
    #[cfg(feature = "notifications")]
    let notification_maintenance = maintenance::start(pool.clone()).await?;
    #[cfg(feature = "notifications")]
    let notification_ingress = {
        let (key_id, key, previous) = CONFIG.notification_event_keys();
        ingress::start(
            pool.clone(),
            CONFIG.admin_database_path(),
            AdmissionPolicy::from_config(&CONFIG)?,
            &CONFIG.notification_ingress_address(),
            key_id.into(),
            key.as_bytes().to_vec(),
            previous.map(|(id, key, expires)| (id.into(), key.as_bytes().to_vec(), expires)),
        )
        .await?
    };
    #[cfg(feature = "full")]
    let task_service = std::sync::Arc::new(TaskService::new(pool.clone())?);
    #[cfg(feature = "full")]
    task_service.bootstrap().await?;
    #[cfg(feature = "full")]
    let deploy_service = std::sync::Arc::new(DeployService::new(pool.clone()));
    #[cfg(feature = "full")]
    deploy_service.bootstrap_installed_current().await?;
    let module_client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(1))
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let module_state = ModuleControlState::initialize(
        pool.clone(),
        module_client,
        DelegationSigner::new(CONFIG.ipc_token.as_bytes())?,
    )
    .await?;

    let (documented_routes, _documented_contracts) = documented_protected_routes();
    let (public_auth_router, _) = public_auth_routes().into_parts();
    #[cfg(feature = "full")]
    let protected_api: Router = documented_routes
        .layer(Extension(task_service))
        .layer(Extension(deploy_service))
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(pool.clone());
    #[cfg(feature = "monitor-distribution")]
    let protected_api: Router = documented_routes
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(pool.clone());

    let public_api: Router = public_auth_router.with_state(pool.clone());
    let (module_control_router, _) = control_routes().into_parts();
    #[cfg(feature = "full")]
    let module_control: Router = module_control_router
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(module_state.clone());
    #[cfg(feature = "monitor-distribution")]
    let module_control: Router = module_control_router
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(module_state.clone());
    let module_gateway: Router = gateway::routes().with_state(module_state.clone());

    PermissionService::sync_permissions(&pool).await?;

    #[cfg(feature = "full")]
    let avatars_prefix = CONFIG.avatars_prefix();
    #[cfg(feature = "full")]
    let uploads_prefix = CONFIG.files_prefix().to_string();
    #[cfg(feature = "full")]
    let uploads_service =
        ServeDir::new(CONFIG.uploads_dir()).append_index_html_on_directories(true);
    #[cfg(feature = "full")]
    let avatars_service =
        ServeDir::new(CONFIG.avatars_dir()).append_index_html_on_directories(true);
    #[cfg(feature = "full")]
    tracing::info!("Serving frontend assets embedded in rz");

    let admin_routes = Router::new()
        .route("/health", get(health))
        .merge(public_api)
        .merge(protected_api)
        .merge(module_control);
    #[cfg(feature = "full")]
    let admin_routes = admin_routes.nest_service(&avatars_prefix, avatars_service);
    #[cfg(feature = "full")]
    let admin_routes = admin_routes.nest_service(&uploads_prefix, uploads_service);
    #[cfg(any(feature = "full", feature = "monitor-distribution"))]
    let admin_routes = admin_routes.fallback(crate::infra::web::serve);
    let admin_routes = admin_routes.layer(admin_cors());
    let app = Router::new()
        // Keep module-owned CORS responses outside the Admin-wide CORS layer.
        .merge(module_gateway)
        .merge(admin_routes)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started successfully, listening on http://{}", addr);
    ModuleService::spawn_synchronizer(module_state);

    let server_result = axum::serve(listener, app).await;
    #[cfg(feature = "notifications")]
    notification_maintenance.shutdown().await;
    #[cfg(feature = "notifications")]
    notification_ingress.shutdown().await;
    server_result?;

    Ok(())
}

#[cfg(all(test, feature = "monitor-distribution"))]
mod monitor_distribution_tests {
    use super::*;
    use crate::features::modules::types::ModuleSpec;
    use axum::body::to_bytes;
    use tower::ServiceExt;

    #[test]
    fn route_inventory_contains_access_and_monitor_navigation_only() {
        let contracts = documented_all_contracts();
        let paths = contracts.iter().map(|contract| contract.path.as_str()).collect::<Vec<_>>();
        for required in [
            "/api/auth/login",
            "/api/auth/me",
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
            "monitor distribution must expose only the permission-options menu route"
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
            5,
            "monitor-notify must expose the exact notification route owner"
        );
        assert_eq!(
            ModuleSpec::fixed().iter().map(|module| module.id).collect::<Vec<_>>(),
            ["monitor"]
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
        let (routes, _) = documented_protected_routes();
        sqlx::query("UPDATE users SET status = 1 WHERE username = 'owner'")
            .execute(&pool)
            .await
            .expect("enable test owner");
        PermissionService::sync_permissions(&pool).await.expect("permission cache");
        let app = routes
            .route_layer(middleware::from_fn_with_state(
                (jwt_codec(), ServerAuthContextLoader::new()),
                auth_middleware,
            ))
            .with_state(pool.clone());
        let owner = jwt_codec().encode(1, "owner").expect("owner token");

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
        let role_id: i64 =
            sqlx::query_scalar("SELECT id FROM roles WHERE code = 'permission_reader'")
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
        let reader = jwt_codec().encode(user_id, "permission_reader").expect("reader token");
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
}

fn admin_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
            axum::http::HeaderName::from_static("x-rustzen-project-key"),
            axum::http::HeaderName::from_static("x-rustzen-monitor-agent-token"),
        ])
}

pub(crate) fn documented_protected_routes()
-> (Router<SqlitePool>, Vec<crate::infra::contract::RouteContract>) {
    let routes = crate::infra::contract::ContractRouter::new()
        .nest("/api/auth", protected_auth_contract_routes())
        .expect("static API contract")
        .nest("/api/system", system_contract_routes())
        .expect("static API contract")
        .nest("/api/account", account_routes())
        .expect("static API contract");
    #[cfg(feature = "notifications")]
    let routes = routes
        .nest("/api/notifications", notification_routes())
        .expect("static notification API contract");
    #[cfg(feature = "full")]
    let routes = routes
        .nest("/api/dashboard", dashboard_routes())
        .expect("static API contract")
        .nest("/api/manage", manage_routes())
        .expect("static API contract");
    routes.into_parts()
}

#[cfg(any(feature = "full", feature = "monitor-distribution", test))]
pub(crate) fn documented_all_contracts() -> Vec<crate::infra::contract::RouteContract> {
    let (_, mut contracts) = documented_protected_routes();
    let (_, public_contracts) = public_auth_routes().into_parts();
    contracts.extend(public_contracts);
    let (_, control_contracts) = control_routes().into_parts();
    contracts.extend(control_contracts);
    contracts
}

async fn health() -> axum::Json<HealthResponse> {
    #[cfg(feature = "monitor-distribution")]
    let response = HealthResponse::ok_selected(env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "full")]
    let response = HealthResponse::ok(env!("CARGO_PKG_VERSION"));
    axum::Json(response)
}

fn server_addr() -> String {
    format!("{}:{}", CONFIG.admin_host(), CONFIG.admin_port())
}

#[cfg(all(test, feature = "full"))]
mod tests;

#[cfg(feature = "monitor-distribution")]
mod selected_contract;
#[cfg(feature = "monitor-distribution")]
pub(crate) use selected_contract::selected_contract_json;
