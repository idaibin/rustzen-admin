use axum::{
    Json,
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use rustzen_auth::auth::CurrentUser;

use crate::common::api::ApiResponse;

use super::{
    service::{InstallationState, web_digest},
    types::WebBindingResponse,
};

pub async fn web_binding() -> Response {
    no_store(
        Json(WebBindingResponse { binding_version: 1, web_digest: web_digest() }).into_response(),
    )
}

pub async fn installation(user: CurrentUser, State(state): State<InstallationState>) -> Response {
    no_store(Json(ApiResponse::new(state.response(&user), None)).into_response())
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
