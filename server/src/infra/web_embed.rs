use axum::{
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use include_dir::{Dir, include_dir};
use tracing::{debug, info, warn};

use crate::core::config::CONFIG;

// embed dist directory into the binary file
// path is relative to the Cargo.toml file location
static WEB_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

/// static file service handler
/// development environment: proxy to Vite development server
/// production environment: use embedded static files
pub async fn web_embed_file_handler(uri: Uri) -> impl IntoResponse {
    let is_enabled = CONFIG.web_embed_enabled;
    info!("Web embed is enabled: {}", is_enabled);
    if is_enabled {
        let path = uri.path().trim_start_matches('/');
        serve_embedded_files(path).await
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Web is disabled"))
            .unwrap()
    }
}
/// check if the path is a static resource path
fn is_static_resource_path(path: &str) -> bool {
    // if the path contains a file extension, it is a static resource
    if path.contains('.') {
        return true;
    }

    // special static resource paths
    if path.starts_with("assets/")
        || path.starts_with("static/")
        || path.starts_with("public/")
        || path.starts_with("images/")
        || path.starts_with("css/")
        || path.starts_with("js/")
    {
        return true;
    }

    // other cases are considered SPA routes
    false
}

/// use embedded static files
async fn serve_embedded_files(path: &str) -> Response {
    debug!("[static file] handle request: {}", path);

    // if the path is the root path, return index.html
    if path.is_empty() || path == "index.html" {
        debug!("[static file] return root path index.html");
        return serve_embedded_index_html().await;
    }

    // check if the path is a static resource
    let is_static = is_static_resource_path(path);
    debug!("[static file] path '{}' is static resource: {}", path, is_static);

    if is_static {
        // try to get the static resource file
        if let Some(file) = WEB_DIR.get_file(path) {
            // set Content-Type based on the file extension
            let content_type = get_content_type(path);
            let contents = file.contents();

            debug!(
                "[static file] find embedded file: {}, Content-Type: {}, size: {} bytes",
                path,
                content_type,
                contents.len()
            );

            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .header("cache-control", "public, max-age=604800") // static resource cache 1 week
                .body(axum::body::Body::from(contents))
                .unwrap();
        } else {
            // static resource file not found
            warn!("[static file] embedded file not found: {}", path);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain; charset=utf-8")
                .body(axum::body::Body::from(format!("File not found: {}", path)))
                .unwrap();
        }
    }

    // for non-static resource paths (SPA routes), return index.html
    // this is especially important for hash routes, because all routes should return index.html
    debug!("[static file] SPA routes, return embedded index.html: {}", path);
    serve_embedded_index_html().await
}

/// serve embedded index.html file
async fn serve_embedded_index_html() -> Response {
    if let Some(index_file) = WEB_DIR.get_file("index.html") {
        debug!("[static file] serve embedded index.html");
        Html(std::str::from_utf8(index_file.contents()).unwrap_or("")).into_response()
    } else {
        warn!("[static file] embedded index.html file not found");

        // if there is no embedded index.html, return a simple default page
        let default_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>RustZen Admin</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
                .logo { font-size: 48px; margin-bottom: 20px; }
                .info { color: #666; }
            </style>
        </head>
        <body>
            <div class="logo">🖥️</div>
            <h1>RustZen Admin</h1>
            <p class="info">Web interface is loading...</p>
            <p class="info">If you see this page, it means the static files may not be correctly embedded.</p>
        </body>
        </html>
        "#;

        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .body(axum::body::Body::from(default_html))
            .unwrap()
    }
}

/// get Content-Type based on the file extension
fn get_content_type(path: &str) -> &'static str {
    if let Some(extension) = path.split('.').last() {
        match extension.to_lowercase().as_str() {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" | "mjs" => "application/javascript; charset=utf-8",
            "jsx" => "application/javascript; charset=utf-8",
            "ts" => "application/typescript; charset=utf-8",
            "tsx" => "application/typescript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "eot" => "application/vnd.ms-fontobject",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "pdf" => "application/pdf",
            "xml" => "application/xml; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "map" => "application/json; charset=utf-8", // source maps
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}
