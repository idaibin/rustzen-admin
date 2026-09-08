#[cfg(feature = "monitor-distribution")]
use crate::features::installation::{
    InstallationState, protected_routes as installation_routes, public_routes as web_binding_routes,
};
#[cfg(feature = "notifications")]
use crate::features::notifications::{
    admission_types::AdmissionPolicy, ingress, maintenance, realtime::RealtimeHub,
};
#[cfg(feature = "full")]
use crate::infra::db::run_migrations;
#[cfg(feature = "full")]
use crate::{
    features::manage::{deploy::service::DeployService, task::service::TaskService},
    middleware::log::log_middleware,
};
use crate::{
    features::{
        auth::public_auth_routes,
        modules::{
            control_routes, gateway,
            service::{ModuleControlState, ModuleService},
        },
    },
    infra::{
        auth_runtime::{ServerAuthContextLoader, jwt_codec},
        config::CONFIG,
        db::{create_default_pool, test_connection},
        permission::PermissionService,
    },
};

#[cfg(feature = "notifications")]
use axum::Extension;
use axum::{Router, middleware, routing::get};
use rustzen_auth::auth::auth_middleware;
use rustzen_ipc::DelegationSigner;
use std::net::SocketAddr;
#[cfg(feature = "full")]
use tower_http::services::ServeDir;

use super::routes::{admin_cors, contract_permission_codes, documented_protected_routes, health};

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
    let notification_realtime = RealtimeHub::new(pool.clone());
    #[cfg(feature = "notifications")]
    let notification_maintenance =
        maintenance::start(pool.clone(), notification_realtime.clone()).await?;
    #[cfg(feature = "notifications")]
    let notification_ingress = {
        let (key_id, key, previous) = CONFIG.notification_event_keys();
        let keys = vec![ingress::ProducerKeys {
            producer: "monitor",
            current_id: key_id.into(),
            current_secret: key.as_bytes().to_vec(),
            previous: previous
                .map(|(id, key, expires)| (id.into(), key.as_bytes().to_vec(), expires)),
        }];
        #[cfg(feature = "reports-notifications")]
        let keys = {
            let mut keys = keys;
            let (key_id, key, previous) = CONFIG.reports_notification_event_keys();
            keys.push(ingress::ProducerKeys {
                producer: "reports",
                current_id: key_id.into(),
                current_secret: key.as_bytes().to_vec(),
                previous: previous
                    .map(|(id, key, expires)| (id.into(), key.as_bytes().to_vec(), expires)),
            });
            keys
        };
        ingress::start_with_keys(
            pool.clone(),
            CONFIG.admin_database_path(),
            AdmissionPolicy::from_config(&CONFIG)?,
            &CONFIG.notification_ingress_address(),
            keys,
            notification_realtime.clone(),
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
    #[cfg(feature = "monitor-distribution")]
    let installation_state =
        InstallationState::load(module_state.clone()).map_err(std::io::Error::other)?;

    let (documented_routes, documented_contracts) = documented_protected_routes();
    #[cfg(feature = "notifications")]
    let documented_routes = documented_routes.layer(Extension(notification_realtime.clone()));
    let (public_auth_router, _) = public_auth_routes().into_parts();
    #[cfg(feature = "full")]
    let protected_api: Router = documented_routes
        .layer(Extension(task_service))
        .layer(Extension(deploy_service))
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(pool.clone());
    #[cfg(feature = "monitor-distribution")]
    let protected_api: Router = documented_routes
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(pool.clone());

    let public_api: Router = public_auth_router.with_state(pool.clone());
    #[cfg(feature = "monitor-distribution")]
    let (web_binding, _) = web_binding_routes().into_parts();
    #[cfg(feature = "monitor-distribution")]
    let (installation_router, installation_contracts) = installation_routes().into_parts();
    #[cfg(feature = "monitor-distribution")]
    let installation_api: Router = installation_router
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(installation_state);
    let (module_control_router, module_control_contracts) = control_routes().into_parts();
    #[cfg(feature = "full")]
    let module_control: Router = module_control_router
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(module_state.clone());
    #[cfg(feature = "monitor-distribution")]
    let module_control: Router = module_control_router
        .route_layer(middleware::from_fn_with_state(
            (jwt_codec(), ServerAuthContextLoader::new(pool.clone())),
            auth_middleware,
        ))
        .with_state(module_state.clone());
    let module_gateway: Router = gateway::routes().with_state(module_state.clone());

    let permission_codes = contract_permission_codes(
        documented_contracts.iter().chain(module_control_contracts.iter()),
    );
    #[cfg(feature = "monitor-distribution")]
    let permission_codes = {
        let mut permission_codes = permission_codes;
        permission_codes.extend(contract_permission_codes(installation_contracts.iter()));
        permission_codes
    };
    PermissionService::sync_permission_codes(&pool, &permission_codes).await?;

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
    #[cfg(feature = "monitor-distribution")]
    let admin_routes = admin_routes.merge(web_binding).merge(installation_api);
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
    notification_realtime.shutdown();
    #[cfg(feature = "notifications")]
    notification_maintenance.shutdown().await;
    #[cfg(feature = "notifications")]
    notification_ingress.shutdown().await;
    server_result?;

    Ok(())
}
fn server_addr() -> String {
    format!("{}:{}", CONFIG.admin_host(), CONFIG.admin_port())
}
