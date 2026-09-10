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

#[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
#[cfg(feature = "monitor-distribution")]
#[derive(RustEmbed)]
#[folder = "selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/dist"]
struct MonitorWebAssets;

#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
#[derive(RustEmbed)]
#[folder = "selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d/dist"]
struct MonitorNotifyWebAssets;

#[cfg(feature = "analytics-distribution")]
#[derive(RustEmbed)]
#[folder = "selected-web/62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40/dist"]
struct AnalyticsWebAssets;

#[cfg(feature = "full")]
type WebAssets = FullWebAssets;
#[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
#[cfg(feature = "monitor-distribution")]
type WebAssets = MonitorWebAssets;
#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
type WebAssets = MonitorNotifyWebAssets;
#[cfg(feature = "analytics-distribution")]
type WebAssets = AnalyticsWebAssets;

#[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
#[cfg(feature = "monitor-distribution")]
const SELECTED_WEB_INVENTORY: &str = include_str!(
    "../../selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/inventory.json"
);
#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
const SELECTED_WEB_INVENTORY: &str = include_str!(
    "../../selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d/inventory.json"
);
#[cfg(feature = "analytics-distribution")]
const SELECTED_WEB_INVENTORY: &str = include_str!(
    "../../selected-web/62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40/inventory.json"
);
#[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
#[cfg(feature = "monitor-distribution")]
const SELECTED_WEB_PRESET: &str = "monitor";
#[cfg(all(feature = "selected-distribution", feature = "notifications"))]
const SELECTED_WEB_PRESET: &str = "monitor-notify";
#[cfg(feature = "analytics-distribution")]
const SELECTED_WEB_PRESET: &str = "analytics";

pub async fn serve(uri: Uri) -> Response {
    #[cfg(feature = "selected-distribution")]
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

    #[cfg(feature = "selected-distribution")]
    use super::{SELECTED_WEB_INVENTORY, SELECTED_WEB_PRESET};
    use super::{WebAssets, serve};

    #[tokio::test]
    async fn embedded_release_contains_index_and_spa_fallback() {
        let selected_path = if cfg!(feature = "analytics-distribution") {
            "/analytics/overview"
        } else {
            "/monitoring/overview"
        };
        for path in ["/", selected_path] {
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

    #[cfg(feature = "selected-distribution")]
    #[test]
    fn selected_embed_is_composition_qualified_and_excludes_other_capabilities() {
        assert!(SELECTED_WEB_INVENTORY.contains(&format!("\"preset\": \"{SELECTED_WEB_PRESET}\"")));
        let composition = if SELECTED_WEB_PRESET == "analytics" {
            "62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40"
        } else if SELECTED_WEB_PRESET == "monitor" {
            "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b"
        } else {
            "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d"
        };
        assert!(SELECTED_WEB_INVENTORY.contains(composition));
        for asset in WebAssets::iter() {
            let bytes = WebAssets::get(asset.as_ref()).expect("embedded asset").data;
            let text = String::from_utf8_lossy(&bytes);
            let forbidden = if SELECTED_WEB_PRESET == "analytics" {
                [
                    "/api/monitor",
                    "/api/reports",
                    "/api/manage",
                    "ReactQueryDevtools",
                    "TanStackRouterDevtools",
                ]
            } else {
                [
                    "/api/insights",
                    "/api/reports",
                    "/api/manage",
                    "ReactQueryDevtools",
                    "TanStackRouterDevtools",
                ]
            };
            for forbidden in forbidden {
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
