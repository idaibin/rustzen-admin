//! Access-control routes retained by the monitor distribution.
//!
//! Role management needs the permission catalogue to create and edit roles. The full menu
//! management surface owns all mutations; this module deliberately exposes only that catalogue.

use axum::extract::{Query, State};
use rustzen_auth::capability::system_menu;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::{
    common::{
        api::{ApiResponse, AppResult, OptionsQuery},
        query::{fetch_with_filters, push_ilike},
    },
    infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor},
};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct PermissionOptionResponse {
    label: String,
    value: i64,
    code: String,
    is_system: bool,
    module_id: Option<String>,
    module_menu_code: Option<String>,
}

pub fn permission_option_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/options",
            OperationDescriptor::GetMenuOptions,
            AccessPolicy::Require(system_menu::OPTIONS),
            axum::routing::get(list_permission_options),
        )
        .expect("static permission-option contract")
}

async fn list_permission_options(
    State(pool): State<SqlitePool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<PermissionOptionResponse>> {
    let options = fetch_with_filters(
        &pool,
        "SELECT id AS value, name AS label, code, is_system, module_id, module_menu_code
         FROM menus WHERE is_active = TRUE AND deleted_at IS NULL",
        |builder| push_ilike(builder, "name", query.q.as_deref()),
        Some("sort_order ASC, name ASC"),
        query.limit,
        None,
    )
    .await?;
    Ok(ApiResponse::success(options))
}
