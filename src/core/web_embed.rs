use axum::{
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use include_dir::{Dir, include_dir};
use tracing::{debug, info, warn};

// 嵌入 dist 目录到二进制文件中
// 路径相对于 Cargo.toml 文件位置
static WEB_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

/// 静态文件服务处理器
/// 开发环境：代理到 Vite 开发服务器
/// 生产环境：使用嵌入的静态文件
pub async fn web_embed_file_handler(uri: Uri) -> impl IntoResponse {
    let is_enabled = std::env::var("WEB_EMBED_ENABLED").unwrap_or_else(|_| "false".to_string());
    info!("Web embed is enabled: {}", is_enabled);
    if is_enabled == "true" {
        let path = uri.path().trim_start_matches('/');
        serve_embedded_files(path).await
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Web is disabled"))
            .unwrap()
    }
}
/// 判断是否为静态资源路径
fn is_static_resource_path(path: &str) -> bool {
    // 如果路径包含文件扩展名，认为是静态资源
    if path.contains('.') {
        return true;
    }

    // 特殊的静态资源路径
    if path.starts_with("assets/")
        || path.starts_with("static/")
        || path.starts_with("public/")
        || path.starts_with("images/")
        || path.starts_with("css/")
        || path.starts_with("js/")
    {
        return true;
    }

    // 其他情况认为是 SPA 路由
    false
}

/// 使用嵌入的静态文件
async fn serve_embedded_files(path: &str) -> Response {
    debug!("[静态文件] 处理请求: {}", path);

    // 如果是根路径，直接返回 index.html
    if path.is_empty() || path == "index.html" {
        debug!("[静态文件] 返回根路径 index.html");
        return serve_embedded_index_html().await;
    }

    // 检查是否为静态资源
    let is_static = is_static_resource_path(path);
    debug!("[静态文件] 路径 '{}' 是否为静态资源: {}", path, is_static);

    if is_static {
        // 尝试获取请求的静态资源文件
        if let Some(file) = WEB_DIR.get_file(path) {
            // 根据文件扩展名设置 Content-Type
            let content_type = get_content_type(path);
            let contents = file.contents();

            debug!(
                "[静态文件] 找到嵌入文件: {}, Content-Type: {}, 大小: {} bytes",
                path,
                content_type,
                contents.len()
            );

            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .header("cache-control", "public, max-age=31536000") // 静态资源缓存1年
                .body(axum::body::Body::from(contents))
                .unwrap();
        } else {
            // 静态资源文件不存在
            warn!("[静态文件] 嵌入文件未找到: {}", path);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain; charset=utf-8")
                .body(axum::body::Body::from(format!("File not found: {}", path)))
                .unwrap();
        }
    }

    // 对于非静态资源路径（SPA 路由），返回 index.html
    // 这对 hash 路由特别重要，因为所有路由都应该返回 index.html
    debug!("[静态文件] SPA 路由，返回嵌入的 index.html: {}", path);
    serve_embedded_index_html().await
}

/// 提供嵌入的 index.html 文件
async fn serve_embedded_index_html() -> Response {
    if let Some(index_file) = WEB_DIR.get_file("index.html") {
        debug!("[静态文件] 提供嵌入的 index.html");
        Html(std::str::from_utf8(index_file.contents()).unwrap_or("")).into_response()
    } else {
        warn!("[静态文件] 嵌入的 index.html 文件未找到");

        // 如果没有嵌入的 index.html，返回一个简单的默认页面
        let default_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Screen Control App</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
                .logo { font-size: 48px; margin-bottom: 20px; }
                .info { color: #666; }
            </style>
        </head>
        <body>
            <div class="logo">🖥️</div>
            <h1>Screen Control App</h1>
            <p class="info">Web 界面正在加载中...</p>
            <p class="info">如果您看到此页面，说明静态文件可能未正确嵌入。</p>
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

/// 根据文件扩展名获取 Content-Type
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
            "map" => "application/json; charset=utf-8", // Source maps
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}
