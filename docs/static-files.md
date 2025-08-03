# 静态文件服务配置

## 🎯 概述

RustZen Admin 已配置支持前端静态文件服务，可以将 React 应用和 Rust API 集成在同一个服务器上。

## 📁 目录结构

```
rustzen-admin/
├── src/core/app.rs          # 静态文件服务配置
├── web/dist/                # 前端构建产物
├── build-frontend.sh        # 前端构建脚本
└── justfile                 # 构建任务配置
```

## 🔧 配置详情

### 1. Cargo.toml 依赖

```toml
# 用于 CORS、日志中间件和静态文件服务
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }
```

### 2. 应用配置 (src/core/app.rs)

```rust
use tower_http::{cors::CorsLayer, services::ServeDir};

// 静态文件服务配置
let static_service = ServeDir::new("web/dist")
    .not_found_service(ServeDir::new("web/dist").append_index_html_on_directories(true));

// 路由配置
let app = Router::new()
    .route("/api/summary", get(summary))
    .nest("/api", public_api.merge(protected_api))
    // 静态文件服务 - 放在最后作为 fallback
    .fallback_service(static_service)
    .layer(cors)
    .with_state(pool);
```

## 🚀 使用方法

### 1. 构建前端

```bash
# 方法一：使用 justfile
just build-web

# 方法二：使用脚本
./build-frontend.sh

# 方法三：手动构建
cd web && pnpm build
```

### 2. 启动全栈服务

```bash
# 构建前端并启动后端（推荐）
just serve

# 或者分步执行
just build-web
cargo run
```

## 🌐 访问地址

- **前端应用**: http://localhost:8000/
- **API 接口**: http://localhost:8000/api/
- **API 文档**: http://localhost:8000/api/summary

## 📝 路由优先级

1. **API 路由**: `/api/*` - 后端 API 接口
2. **静态文件**: `/*` - 前端静态资源和 SPA 路由

## ✨ 特性

- ✅ **SPA 路由支持**: 前端路由自动 fallback 到 `index.html`
- ✅ **统一端口**: 前后端使用同一端口，避免 CORS 问题
- ✅ **生产就绪**: 使用 `tower-http::services::ServeDir` 高效服务静态文件
- ✅ **缓存优化**: 静态资源支持浏览器缓存
- ✅ **错误处理**: 404 错误自动重定向到前端应用
