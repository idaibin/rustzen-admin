use crate::common::response::ApiResponse;
use axum::{Json, Router, routing::get};
use serde_json::Value;

pub fn router() -> Router {
    Router::new().route("/", get(get_log_list))
}

async fn get_log_list() -> Json<ApiResponse<Vec<Value>>> {
    ApiResponse::success(vec![])
}
