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

    let admin_routes = Router::new()
        .route("/health", get(health))
        .merge(public_api)
        .merge(protected_api)
        .merge(module_control)
        .nest_service(&avatars_prefix, avatars_service)
        .nest_service(&uploads_prefix, uploads_service)
        .fallback(crate::infra::web::serve)
        .layer(admin_cors());
    let app = Router::new()
        // Keep module-owned CORS responses outside the Admin-wide CORS layer.
        .merge(module_gateway)
        .merge(admin_routes)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started successfully, listening on http://{}", addr);
    ModuleService::spawn_synchronizer(module_state);

    axum::serve(listener, app).await?;

    Ok(())
}

fn admin_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
            axum::http::HeaderName::from_static("x-rustzen-project-key"),
            axum::http::HeaderName::from_static("x-rustzen-monitor-agent-token"),
        ])
}

pub(crate) fn documented_protected_routes() -> (
    Router<SqlitePool>,
    Vec<crate::infra::contract::RouteContract>,
) {
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
mod tests;
