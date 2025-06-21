# --- 阶段 1: 构建前端 ---
FROM node:24-alpine AS frontend-builder

# 设置工作目录
WORKDIR /app/frontend

# 复制前端相关的文件并安装依赖
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile

# 复制前端源码并构建
COPY frontend/ ./
RUN pnpm build


# --- 阶段 2: 构建后端 ---
FROM rust:1-alpine AS backend-builder

# 设置工作目录
WORKDIR /app/backend

# 创建一个无功能的 cargo 项目来缓存依赖
RUN cargo init --bin .
COPY backend/Cargo.toml ./
RUN cargo build --release --locked && rm -rf src target/release/backend

# 复制后端源码并构建
COPY backend/ ./
RUN cargo build --release --locked


# --- 阶段 3: 创建最终的生产镜像 ---
FROM alpine:latest

# 设置工作目录
WORKDIR /app

# 安装必要的运行时依赖
RUN apk --no-cache add libc6-compat

# 从前端构建阶段复制静态文件
COPY --from=frontend-builder /app/frontend/dist /app/public

# 从后端构建阶段复制编译好的二进制文件
COPY --from=backend-builder /app/backend/target/release/backend /app/backend

# 暴露端口
EXPOSE 8000

# 设置环境变量，指向静态文件目录
ENV RUSTZEN_STATIC_PATH=/app/public

# 运行应用
CMD ["/app/backend"] 