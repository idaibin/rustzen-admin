use axum::{
    Json,
    extract::{
        FromRequest, FromRequestParts, Query, Request,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

/// Typed JSON for module handlers that returns the Admin module-input envelope
/// for valid JSON whose data does not match the target type.
pub struct ModuleJson<T>(pub T);

impl<T, S> FromRequest<S> for ModuleJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ModuleJsonRejection;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(ModuleJsonRejection::from)
    }
}

pub enum ModuleJsonRejection {
    Data(ModuleInputRejection),
    Passthrough(JsonRejection),
}

impl From<JsonRejection> for ModuleJsonRejection {
    fn from(rejection: JsonRejection) -> Self {
        if matches!(rejection, JsonRejection::JsonDataError(_)) {
            Self::Data(ModuleInputRejection::new(rejection.status(), rejection.body_text()))
        } else {
            Self::Passthrough(rejection)
        }
    }
}

impl IntoResponse for ModuleJsonRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Data(rejection) => rejection.into_response(),
            Self::Passthrough(rejection) => rejection.into_response(),
        }
    }
}

/// Typed query parameters with the Admin module-input error envelope.
pub struct ModuleQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ModuleQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ModuleInputRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state).await.map(|Query(value)| Self(value)).map_err(
            |rejection: QueryRejection| {
                ModuleInputRejection::new(rejection.status(), rejection.body_text())
            },
        )
    }
}

pub struct ModuleInputRejection {
    status: StatusCode,
    message: String,
}

impl ModuleInputRejection {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

impl IntoResponse for ModuleInputRejection {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "code": 40002,
                "message": self.message,
                "data": null,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        routing::{get, post},
    };
    use serde::Deserialize;
    use tower::ServiceExt;

    use super::{ModuleJson, ModuleQuery};

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        required_value: String,
    }

    async fn json_handler(ModuleJson(payload): ModuleJson<Payload>) -> String {
        payload.required_value
    }

    async fn query_handler(ModuleQuery(payload): ModuleQuery<Payload>) -> String {
        payload.required_value
    }

    #[tokio::test]
    async fn data_and_query_rejections_use_the_module_error_envelope() {
        let app =
            Router::new().route("/json", post(json_handler)).route("/query", get(query_handler));
        for request in [
            Request::builder()
                .method("POST")
                .uri("/json")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .expect("JSON request"),
            Request::builder().uri("/query").body(Body::empty()).expect("query request"),
        ] {
            let response = app.clone().oneshot(request).await.expect("response");
            assert!(matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ));
            let body = to_bytes(response.into_body(), usize::MAX).await.expect("response body");
            let body: serde_json::Value = serde_json::from_slice(&body).expect("JSON envelope");
            assert_eq!(body["code"], 40002);
            assert!(body["message"].as_str().is_some_and(|message| !message.is_empty()));
            assert!(body["data"].is_null());
        }
    }

    #[tokio::test]
    async fn syntax_and_content_type_rejections_remain_axum_passthroughs() {
        let app = Router::new().route("/json", post(json_handler));
        for request in [
            Request::builder()
                .method("POST")
                .uri("/json")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{"))
                .expect("syntax request"),
            Request::builder()
                .method("POST")
                .uri("/json")
                .body(Body::from("{}"))
                .expect("content type request"),
        ] {
            let response = app.clone().oneshot(request).await.expect("response");
            assert_ne!(
                response.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()),
                Some("application/json")
            );
        }
    }
}
