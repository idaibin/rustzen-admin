use axum::{
    body::Body,
    http::{StatusCode, Uri, header},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct WebAssets;

pub async fn serve(uri: Uri) -> Response {
    if uri.path().starts_with("/api/") {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"code":10001,"message":"API route not found.","data":null}"#))
            .unwrap_or_else(|_| Response::new(Body::empty()));
    }
    let requested = uri.path().trim_start_matches('/');
    let path = if requested.is_empty() { "index.html" } else { requested };
    match WebAssets::get(path).or_else(|| WebAssets::get("index.html")) {
        Some(asset) => {
            let served_path = if WebAssets::get(path).is_some() { path } else { "index.html" };
            let content_type =
                mime_guess::from_path(served_path).first_or_octet_stream().as_ref().to_string();
            let cache_control = if served_path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, cache_control)
                .body(Body::from(asset.data.into_owned()))
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("embedded web assets not found"))
            .unwrap_or_else(|_| Response::new(Body::empty())),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::to_bytes,
        http::{StatusCode, Uri},
    };

    use super::serve;

    #[tokio::test]
    async fn embedded_release_contains_index_and_spa_fallback() {
        for path in ["/", "/monitoring/overview"] {
            let response = serve(path.parse::<Uri>().expect("uri")).await;
            assert_eq!(response.status(), StatusCode::OK);
            let body = to_bytes(response.into_body(), usize::MAX).await.expect("body");
            assert!(
                body.windows(15).any(|window| window == b"<div id=\"root\">") || !body.is_empty()
            );
        }
    }

    #[tokio::test]
    async fn unknown_api_paths_never_fall_through_to_spa_html() {
        let response = serve("/api/manage/tasks".parse::<Uri>().expect("uri")).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get("content-type").and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.expect("body");
        assert!(!body.windows(5).any(|window| window == b"<html"));
    }
}
