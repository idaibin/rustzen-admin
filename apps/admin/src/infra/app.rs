use crate::{
    features::{
        account::account_routes,
        auth::{protected_auth_contract_routes, public_auth_routes},
        dashboard::dashboard_routes,
        manage::{deploy::service::DeployService, manage_routes, task::service::TaskService},
        modules::{
            control_routes, gateway,
            service::{ModuleControlState, ModuleService},
        },
        system::system_contract_routes,
    },
    infra::{
        auth_runtime::{ServerAuthContextLoader, jwt_codec},
        config::CONFIG,
        db::{create_default_pool, run_migrations, test_connection},
        permission::PermissionService,
    },
    middleware::log::log_middleware,
};

use axum::{
    Extension, Router,
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
use tower_http::{cors::CorsLayer, services::ServeDir};

#[tracing::instrument(name = "run_server")]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database connection pool...");
    let pool = create_default_pool().await?;
    run_migrations(&pool).await?;
    test_connection(&pool).await?;
    let task_service = std::sync::Arc::new(TaskService::new(pool.clone())?);
    task_service.bootstrap().await?;
    let deploy_service = std::sync::Arc::new(DeployService::new(pool.clone()));
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

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
            axum::http::HeaderName::from_static("x-rustzen-project-key"),
            axum::http::HeaderName::from_static("x-rustzen-monitor-agent-token"),
        ]);

    let (documented_routes, _documented_contracts) = documented_protected_routes();
    let (public_auth_router, _) = public_auth_routes().into_parts();
    let protected_api: Router = documented_routes
        .layer(Extension(task_service))
        .layer(Extension(deploy_service))
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(pool.clone());

    let public_api: Router = public_auth_router.with_state(pool.clone());
    let (module_control_router, _) = control_routes().into_parts();
    let module_control: Router = module_control_router
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(module_state.clone());
    let module_gateway: Router = gateway::routes().with_state(module_state.clone());

    PermissionService::sync_permissions(&pool).await?;

    let uploads_prefix = CONFIG.files_prefix().to_string();
    let avatars_prefix = CONFIG.avatars_prefix();
    let uploads_service =
        ServeDir::new(CONFIG.uploads_dir()).append_index_html_on_directories(true);
    let avatars_service =
        ServeDir::new(CONFIG.avatars_dir()).append_index_html_on_directories(true);
    tracing::info!("Serving frontend assets embedded in rz");

    let app = Router::new()
        .route("/health", get(health))
        .merge(public_api)
        .merge(protected_api)
        .merge(module_control)
        .merge(module_gateway)
        .nest_service(&avatars_prefix, avatars_service)
        .nest_service(&uploads_prefix, uploads_service)
        .layer(cors)
        .fallback(crate::infra::web::serve)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started successfully, listening on http://{}", addr);
    ModuleService::spawn_synchronizer(module_state);

    axum::serve(listener, app).await?;

    Ok(())
}

pub(crate) fn documented_protected_routes()
-> (Router<SqlitePool>, Vec<crate::infra::contract::RouteContract>) {
    crate::infra::contract::ContractRouter::new()
        .nest("/api/auth", protected_auth_contract_routes())
        .expect("static API contract")
        .nest("/api/system", system_contract_routes())
        .expect("static API contract")
        .nest("/api/account", account_routes())
        .expect("static API contract")
        .nest("/api/dashboard", dashboard_routes())
        .expect("static API contract")
        .nest("/api/manage", manage_routes())
        .expect("static API contract")
        .into_parts()
}

pub(crate) fn documented_all_contracts() -> Vec<crate::infra::contract::RouteContract> {
    let (_, mut contracts) = documented_protected_routes();
    let (_, public_contracts) = public_auth_routes().into_parts();
    contracts.extend(public_contracts);
    let (_, control_contracts) = control_routes().into_parts();
    contracts.extend(control_contracts);
    contracts
}

async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse::ok(env!("CARGO_PKG_VERSION")))
}

fn server_addr() -> String {
    format!("{}:{}", CONFIG.admin_host(), CONFIG.admin_port())
}

#[cfg(test)]
mod contract_route_tests {
    use super::*;
    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        extract::ConnectInfo,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
    };
    use rustzen_auth::{
        auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec},
        error::CoreError,
    };
    use tower::ServiceExt;

    #[derive(Clone)]
    struct TestLoader;

    #[async_trait]
    impl AuthContextLoader for TestLoader {
        async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
            let permissions =
                if claims.username == "owner" { vec!["*".to_owned()] } else { Vec::new() };
            Ok(CurrentUser::new(claims.user_id, claims.username.clone(), permissions, false))
        }
    }

    async fn assert_json_error(response: axum::response::Response, status: StatusCode, code: i32) {
        assert_eq!(response.status(), status);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .expect("JSON error response");
        assert_eq!(body["code"], code);
        assert!(body["message"].is_string());
        assert!(body["data"].is_null());
    }

    #[tokio::test]
    async fn documented_nested_routes_match_their_final_public_paths() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        let (routes, contracts) = documented_protected_routes();
        assert!(contracts.iter().any(|contract| contract.path == "/api/auth/me"));
        assert!(contracts.iter().any(|contract| contract.path == "/api/system/users"));
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool);

        let auth_response = app
            .clone()
            .oneshot(Request::get("/api/auth/me").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_json_error(auth_response, StatusCode::UNAUTHORIZED, 401).await;

        let viewer = codec.encode(2, "viewer").expect("token");
        let user_response = app
            .clone()
            .oneshot(
                Request::post("/api/system/users")
                    .header("authorization", format!("Bearer {viewer}"))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_json_error(user_response, StatusCode::FORBIDDEN, 403).await;

        let owner = codec.encode(1, "owner").expect("token");
        let missing_content_type = app
            .clone()
            .oneshot(
                Request::post("/api/system/users")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_content_type.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            missing_content_type.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert!(!to_bytes(missing_content_type.into_body(), usize::MAX).await.unwrap().is_empty());

        let malformed_json = app
            .clone()
            .oneshot(
                Request::post("/api/system/users")
                    .header("authorization", format!("Bearer {owner}"))
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(malformed_json.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            malformed_json.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert!(!to_bytes(malformed_json.into_body(), usize::MAX).await.unwrap().is_empty());

        let invalid_dto = app
            .oneshot(
                Request::post("/api/system/users")
                    .header("authorization", format!("Bearer {owner}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username": 1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_dto.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(invalid_dto.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");
        assert!(!to_bytes(invalid_dto.into_body(), usize::MAX).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn login_statuses_return_the_documented_json_errors() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let (routes, _) = public_auth_routes().into_parts();
        let app = routes
            .layer(Extension(ConnectInfo(
                "127.0.0.1:3000".parse::<std::net::SocketAddr>().expect("address"),
            )))
            .with_state(pool.clone());

        for (status, expected_status, code) in [
            (2_i16, StatusCode::FORBIDDEN, 10004),
            (3_i16, StatusCode::BAD_REQUEST, 10005),
            (4_i16, StatusCode::BAD_REQUEST, 10006),
        ] {
            sqlx::query("UPDATE users SET status = ? WHERE username = 'owner'")
                .bind(status)
                .execute(&pool)
                .await
                .expect("update seeded owner status");
            let response = app
                .clone()
                .oneshot(
                    Request::post("/api/auth/login")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"username":"owner","password":"anything"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_json_error(response, expected_status, code).await;
        }
    }

    #[tokio::test]
    async fn business_and_internal_errors_match_the_json_error_envelope() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let (routes, _) = documented_protected_routes();
        PermissionService::sync_permissions(&pool).await.expect("permission cache");
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool);
        let owner = codec.encode(1, "owner").expect("token");
        let not_found = app
            .clone()
            .oneshot(
                Request::delete("/api/system/users/999")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_json_error(not_found, StatusCode::NOT_FOUND, 10001).await;

        let owner_status = codec.encode(1, "owner").expect("status token");
        let invalid_status = app
            .clone()
            .oneshot(
                Request::put("/api/system/users/1/status")
                    .header("authorization", format!("Bearer {owner_status}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"status":99}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_json_error(invalid_status, StatusCode::BAD_REQUEST, 10007).await;

        assert_json_error(
            crate::common::error::AppError::from(
                crate::common::error::ServiceError::UsernameConflict,
            )
            .into_response(),
            StatusCode::CONFLICT,
            10201,
        )
        .await;
        assert_json_error(
            crate::common::error::AppError::from(
                crate::common::error::ServiceError::DatabaseQueryFailed,
            )
            .into_response(),
            StatusCode::INTERNAL_SERVER_ERROR,
            20001,
        )
        .await;
    }

    #[tokio::test]
    async fn role_management_rejects_deletion_of_assigned_custom_role() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let menu_id: i64 = sqlx::query_scalar(
            "INSERT INTO menus (
                 parent_id, name, code, menu_type, sort_order, status, is_system, is_manual,
                 is_active
             ) VALUES (0, 'Role list', 'test:role:list', 2, 1, 1, FALSE, TRUE, TRUE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("assignable permission menu");

        let (routes, _) = documented_protected_routes();
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool.clone());
        let owner = codec.encode(1, "owner").expect("owner token");
        let create_response = app
            .clone()
            .oneshot(
                Request::post("/api/system/roles")
                    .header("authorization", format!("Bearer {owner}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "name": "Auditor",
                            "code": "auditor",
                            "status": 1,
                            "menuIds": [menu_id],
                            "description": "Can review role lists"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        let role_id: i64 = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'auditor'")
            .fetch_one(&pool)
            .await
            .expect("created custom role");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(1_i64)
            .bind(role_id)
            .execute(&pool)
            .await
            .expect("assign custom role");

        let delete_response = app
            .oneshot(
                Request::delete(format!("/api/system/roles/{role_id}"))
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::BAD_REQUEST);
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(delete_response.into_body(), usize::MAX).await.unwrap(),
        )
        .expect("role deletion error");
        assert_eq!(payload["code"], 10002);
        assert!(payload["message"].as_str().unwrap().contains("assigned"));

        let role_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE id = ? AND deleted_at IS NULL")
                .bind(role_id)
                .fetch_one(&pool)
                .await
                .expect("role remains after refused deletion");
        let assignment_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_roles WHERE role_id = ?")
                .bind(role_id)
                .fetch_one(&pool)
                .await
                .expect("role assignment remains after refused deletion");
        assert_eq!(role_count, 1);
        assert_eq!(assignment_count, 1);
    }

    #[tokio::test]
    async fn role_list_exposes_assignment_count_and_deletable_state() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");

        let unassigned_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Unassigned role', 'unassigned_role', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("unassigned role");
        let assigned_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Assigned role', 'assigned_role', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("assigned role");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(1_i64)
            .bind(assigned_role_id)
            .execute(&pool)
            .await
            .expect("assigned role relation");

        let (routes, _) = documented_protected_routes();
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool);
        let owner = codec.encode(1, "owner").expect("owner token");
        let response = app
            .oneshot(
                Request::get("/api/system/roles?current=1&pageSize=20")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .expect("role list response");
        let rows = payload["data"].as_array().expect("role list data");
        let row_for = |code: &str| {
            rows.iter()
                .find(|row| row["code"] == code)
                .unwrap_or_else(|| panic!("missing role {code}"))
        };

        let unassigned = row_for("unassigned_role");
        assert_eq!(unassigned["id"], unassigned_role_id);
        assert_eq!(unassigned["assignedUserCount"], 0);
        assert_eq!(unassigned["deletable"], true);

        let assigned = row_for("assigned_role");
        assert_eq!(assigned["id"], assigned_role_id);
        assert_eq!(assigned["assignedUserCount"], 1);
        assert_eq!(assigned["deletable"], false);

        let owner = row_for("owner");
        assert_eq!(owner["deletable"], false);
    }

    fn multipart_body(boundary: &str, fields: &[(&str, Option<&str>, &str, &str)]) -> Body {
        let mut body = String::new();
        for (name, filename, content_type, value) in fields {
            body.push_str(&format!("--{boundary}\r\n"));
            body.push_str(&format!("Content-Disposition: form-data; name=\"{name}\""));
            if let Some(filename) = filename {
                body.push_str(&format!("; filename=\"{filename}\""));
            }
            body.push_str("\r\n");
            if !content_type.is_empty() {
                body.push_str(&format!("Content-Type: {content_type}\r\n"));
            }
            body.push_str(&format!("\r\n{value}\r\n"));
        }
        body.push_str(&format!("--{boundary}--\r\n"));
        Body::from(body)
    }

    #[tokio::test]
    async fn multipart_contract_routes_accept_generated_file_inputs() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let (routes, _) = documented_protected_routes();
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .layer(Extension(std::sync::Arc::new(DeployService::new(pool.clone()))))
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool);
        let owner = codec.encode(1, "owner").expect("token");

        let boundary = "avatar-boundary";
        let avatar_response = app
            .clone()
            .oneshot(
                Request::post("/api/account/avatar")
                    .header("authorization", format!("Bearer {owner}"))
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(multipart_body(
                        boundary,
                        &[("file", Some("avatar.png"), "image/png", "png")],
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(avatar_response.status(), StatusCode::OK);
        let avatar_payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(avatar_response.into_body(), usize::MAX).await.unwrap(),
        )
        .expect("avatar response");
        let avatar_url = avatar_payload["data"].as_str().expect("avatar URL");
        assert!(avatar_url.ends_with(".png"));
        crate::common::files::remove_avatar_by_url(avatar_url).await.expect("avatar cleanup");

        let oversized_avatar = "x".repeat(3 * 1024 * 1024 + 1024);
        let oversized_response = app
            .clone()
            .oneshot(
                Request::post("/api/account/avatar")
                    .header("authorization", format!("Bearer {owner}"))
                    .header("content-type", "multipart/form-data; boundary=oversized-avatar")
                    .body(multipart_body(
                        "oversized-avatar",
                        &[("file", Some("avatar.png"), "image/png", &oversized_avatar)],
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let oversized_status = oversized_response.status();
        let oversized_content_type = oversized_response.headers().get("content-type").cloned();
        let oversized_body = to_bytes(oversized_response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(oversized_status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(oversized_content_type.unwrap(), "text/plain; charset=utf-8");
        assert!(!oversized_body.is_empty());

        let invalid_deployment_id = app
            .clone()
            .oneshot(
                Request::get("/api/manage/deploy/not-a-number")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_deployment_id.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            invalid_deployment_id.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert!(!to_bytes(invalid_deployment_id.into_body(), usize::MAX).await.unwrap().is_empty());

        let deployment_boundary = "deployment-boundary";
        let deployment_response = app
            .oneshot(
                Request::post("/api/manage/deploy/upload")
                    .header("authorization", format!("Bearer {owner}"))
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={deployment_boundary}"),
                    )
                    .body(multipart_body(
                        deployment_boundary,
                        &[
                            ("component", None, "text/plain", "release"),
                            ("version", None, "text/plain", "0.5.0"),
                            (
                                "file",
                                Some("release.tar"),
                                "application/octet-stream",
                                "not-a-bundle",
                            ),
                        ],
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deployment_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn export_logs_route_returns_csv_content_type_and_body() {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let (routes, _) = documented_protected_routes();
        let codec = JwtCodec::new("contract-test", 60);
        let app = Router::new()
            .merge(routes)
            .route_layer(middleware::from_fn_with_state(
                (codec.clone(), TestLoader),
                auth_middleware,
            ))
            .with_state(pool);
        let owner = codec.encode(1, "owner").expect("token");
        let invalid_query = app
            .clone()
            .oneshot(
                Request::get("/api/manage/logs/export?current=invalid")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_query.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            invalid_query.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert!(!to_bytes(invalid_query.into_body(), usize::MAX).await.unwrap().is_empty());
        let response = app
            .oneshot(
                Request::get("/api/manage/logs/export")
                    .header("authorization", format!("Bearer {owner}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/csv; charset=utf-8");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            body,
            "ID,user_id,username,action,description,status,duration_ms,ip_address,user_agent,created_at\n"
        );
    }

    #[test]
    fn admin_native_contract_inventory_has_no_duplicate_operations_or_routes() {
        let contracts = documented_all_contracts();
        assert_eq!(contracts.len(), 42);

        let mut operations = std::collections::BTreeSet::new();
        let mut routes = std::collections::BTreeSet::new();
        for contract in contracts {
            assert!(operations.insert(contract.operation.operation_id()));
            assert!(routes.insert((contract.method, contract.path)));
        }
        assert_eq!(operations.len(), 42);
        assert_eq!(routes.len(), 42);
    }
}
