use crate::common::api::ApiResponse;
use axum::{Json, Router, routing::get};
use serde_json::Value;
use sqlx::PgPool;

pub fn log_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_log_list))
}

async fn get_log_list() -> Json<ApiResponse<Vec<Value>>> {
    ApiResponse::success(vec![])
}
