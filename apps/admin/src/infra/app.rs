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
    let protected_legacy = Router::new()
        .nest("/account", account_routes())
        .nest("/dashboard", dashboard_routes())
        .nest("/manage", manage_routes());
    let protected_api: Router = documented_routes
        .merge(Router::new().nest("/api", protected_legacy))
        .layer(Extension(task_service))
        .layer(Extension(deploy_service))
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new()),
            auth_middleware,
        ))
        .with_state(pool.clone());

    let public_api: Router =
        Router::new().nest("/auth", public_auth_routes()).with_state(pool.clone());
    let module_control: Router = control_routes()
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
        .nest("/api", public_api)
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
        .into_parts()
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
        body::Body,
        http::{Request, StatusCode},
        middleware,
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
        assert_eq!(auth_response.status(), StatusCode::UNAUTHORIZED);

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
        assert_eq!(user_response.status(), StatusCode::FORBIDDEN);

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
    }
}
