#[cfg(feature = "notifications")]
use crate::features::notifications::notification_routes;
#[cfg(feature = "full")]
use crate::features::{dashboard::dashboard_routes, manage::manage_routes};
use crate::{
    features::{
        account::account_routes,
        auth::{protected_auth_contract_routes, public_auth_routes},
        modules::control_routes,
        system::system_contract_routes,
    },
    infra::contract::{ContractRouter, RegisteredAccess, RouteContract},
};
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use rustzen_ipc::HealthResponse;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

pub(super) fn contract_permission_codes<'a>(
    contracts: impl Iterator<Item = &'a RouteContract>,
) -> Vec<String> {
    contracts
        .flat_map(|contract| match &contract.access {
            RegisteredAccess::Require(codes)
            | RegisteredAccess::Any(codes)
            | RegisteredAccess::All(codes) => codes.clone(),
            RegisteredAccess::Public | RegisteredAccess::Authenticated => Vec::new(),
        })
        .collect()
}

pub(super) fn admin_cors() -> CorsLayer {
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

pub(crate) fn documented_protected_routes() -> (Router<SqlitePool>, Vec<RouteContract>) {
    let routes = ContractRouter::new()
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
pub(crate) fn documented_all_contracts() -> Vec<RouteContract> {
    let (_, mut contracts) = documented_protected_routes();
    let (_, public_contracts) = public_auth_routes().into_parts();
    contracts.extend(public_contracts);
    let (_, control_contracts) = control_routes().into_parts();
    contracts.extend(control_contracts);
    contracts
}

pub(super) async fn health() -> axum::Json<HealthResponse> {
    #[cfg(feature = "monitor-distribution")]
    let response = HealthResponse::ok_selected(env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "full")]
    let response = HealthResponse::ok(env!("CARGO_PKG_VERSION"));
    axum::Json(response)
}
