use axum::{
    body::Body,
    http::{StatusCode, Uri, header},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[cfg(feature = "full")]
#[folder = "../web/dist"]
struct FullWebAssets;

#[cfg(all(feature = "monitor-distribution", not(feature = "notifications")))]
#[derive(RustEmbed)]
#[folder = "selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/dist"]
struct MonitorWebAssets;

#[cfg(all(feature = "monitor-distribution", feature = "notifications"))]
#[derive(RustEmbed)]
#[folder = "selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d/dist"]
struct MonitorNotifyWebAssets;

#[cfg(feature = "full")]
type WebAssets = FullWebAssets;
#[cfg(all(feature = "monitor-distribution", not(feature = "notifications")))]
type WebAssets = MonitorWebAssets;
#[cfg(all(feature = "monitor-distribution", feature = "notifications"))]
type WebAssets = MonitorNotifyWebAssets;

#[cfg(all(feature = "monitor-distribution", not(feature = "notifications")))]
const SELECTED_WEB_INVENTORY: &str = include_str!(
    "../../selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/inventory.json"
);
#[cfg(all(feature = "monitor-distribution", feature = "notifications"))]
const SELECTED_WEB_INVENTORY: &str = include_str!(
    "../../selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d/inventory.json"
);
#[cfg(all(feature = "monitor-distribution", not(feature = "notifications")))]
const SELECTED_WEB_PRESET: &str = "monitor";
#[cfg(all(feature = "monitor-distribution", feature = "notifications"))]
const SELECTED_WEB_PRESET: &str = "monitor-notify";

pub async fn serve(uri: Uri) -> Response {
    #[cfg(feature = "monitor-distribution")]
    debug_assert!(
        SELECTED_WEB_INVENTORY.contains(&format!("\"preset\": \"{SELECTED_WEB_PRESET}\""))
    );
    if uri.path() == "/api"
        || uri.path().starts_with("/api/")
        || uri.path() == "/internal"
        || uri.path().starts_with("/internal/")
    {
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
            } else if served_path == "index.html" {
                "no-store"
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

    #[cfg(feature = "monitor-distribution")]
    use super::{SELECTED_WEB_INVENTORY, SELECTED_WEB_PRESET};
    use super::{WebAssets, serve};

    #[tokio::test]
    async fn embedded_release_contains_index_and_spa_fallback() {
        for path in ["/", "/monitoring/overview"] {
            let response = serve(path.parse::<Uri>().expect("uri")).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[axum::http::header::CACHE_CONTROL], "no-store");
            let body = to_bytes(response.into_body(), usize::MAX).await.expect("body");
            assert!(
                body.windows(15).any(|window| window == b"<div id=\"root\">") || !body.is_empty()
            );
        }
    }

    #[tokio::test]
    async fn content_addressed_assets_remain_immutable() {
        let asset = WebAssets::iter()
            .find(|path| path.starts_with("assets/") && path.ends_with(".js"))
            .expect("embedded JavaScript asset");
        let response = serve(format!("/{asset}").parse::<Uri>().expect("uri")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[axum::http::header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
    }

    #[cfg(feature = "monitor-distribution")]
    #[test]
    fn monitor_embed_is_composition_qualified_and_excludes_full_capabilities() {
        assert!(SELECTED_WEB_INVENTORY.contains(&format!("\"preset\": \"{SELECTED_WEB_PRESET}\"")));
        let composition = if SELECTED_WEB_PRESET == "monitor" {
            "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b"
        } else {
            "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d"
        };
        assert!(SELECTED_WEB_INVENTORY.contains(composition));
        for asset in WebAssets::iter() {
            let bytes = WebAssets::get(asset.as_ref()).expect("embedded asset").data;
            let text = String::from_utf8_lossy(&bytes);
            for forbidden in [
                "/api/insights",
                "/api/reports",
                "/api/manage",
                "ReactQueryDevtools",
                "TanStackRouterDevtools",
            ] {
                assert!(
                    !text.contains(forbidden),
                    "selected embedded asset {asset} contains {forbidden}"
                );
            }
        }
    }

    #[tokio::test]
    async fn unknown_api_paths_never_fall_through_to_spa_html() {
        for path in [
            "/api",
            "/api?x=1",
            "/api/unknown",
            "/api/manage/tasks",
            "/internal",
            "/internal/v1/notification-events",
        ] {
            let response = serve(path.parse::<Uri>().expect("uri")).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(
                response.headers().get("content-type").and_then(|value| value.to_str().ok()),
                Some("application/json"),
                "{path}"
            );
            let body = to_bytes(response.into_body(), usize::MAX).await.expect("body");
            assert!(!body.windows(5).any(|window| window == b"<html"), "{path}");
        }
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
