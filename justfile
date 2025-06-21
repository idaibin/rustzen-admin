# justfile - 项目统一命令入口

# 开发模式：一键同时启动后端 + 前端
dev:
  just backend-dev & just frontend-dev

# 启动 Rust 后端（支持热重载）
backend-dev:
  cd backend && cargo watch -x run

# 启动前端（Vite 开发模式）
frontend-dev:
  cd frontend && pnpm dev

# 构建全部（生产环境）
build:
  just backend-build && just frontend-build

# 构建 Rust 后端 release
backend-build:
  cd backend && cargo build --release

# 构建前端生产包
frontend-build:
  cd frontend && pnpm build

# 清理构建输出
clean:
  rm -rf backend/target frontend/dist desktop/src-tauri/target

# 构建桌面客户端（可选）
tauri-build:
  cd frontend && pnpm tauri build

# 启动桌面客户端开发模式（可选）
tauri-dev:
  cd frontend && pnpm tauri dev

# Docker 构建镜像（预留）
docker-build:
  docker build -t rustzen-admin .

# Docker 推送镜像（预留）
docker-push:
  docker push your-registry/rustzen-admin