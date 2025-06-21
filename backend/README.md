# Rustzen Admin Backend

基于 Rust + Axum + SQLx 构建的后端服务。

## 🔒 环境配置（重要安全提醒）

**⚠️ 安全警告：绝对不要将包含真实凭据的 `.env` 文件提交到版本控制系统！**

### 配置步骤：

1. **复制环境变量模板**：

   ```bash
   cp .env.example .env
   ```

2. **编辑 `.env` 文件**，填入您的真实配置：

   ```env
   # PostgreSQL Database URL - 替换为您的真实数据库连接信息
   DATABASE_URL="postgresql://your_username:your_password@your_host:5432/your_database"

   # Server configuration
   APP_HOST="0.0.0.0"
   APP_PORT="3001"

   # JWT Secret - 生成一个安全的随机字符串
   JWT_SECRET="your-super-secret-jwt-key-change-this-in-production"
   JWT_EXPIRES_IN="1d"

   # Logging level
   RUST_LOG="backend=debug,tower_http=debug,axum::rejection=trace"
   ```

3. **验证 `.gitignore` 配置**：
   确保 `.env` 文件已被添加到 `.gitignore` 中，防止意外提交。

### 生产环境部署建议：

- 使用环境变量或密钥管理服务（如 AWS Secrets Manager、Azure Key Vault）
- 为不同环境（开发、测试、生产）使用不同的数据库
- 定期轮换 JWT 密钥和数据库凭据
- 启用数据库连接加密（SSL/TLS）

## 依赖说明

项目使用了以下**最新版本**的核心依赖：

- `axum = "0.8"` - Web 框架
- `sqlx = "0.8"` - 数据库操作工具包
- `tokio = "1"` - 异步运行时
- `tower-http = "0.6"` - HTTP 中间件
- `dotenvy = "0.15"` - 环境变量加载
- `thiserror = "2.0"` - 错误处理
- `chrono = "0.4"` - 日期时间处理
- `uuid = "1.17"` - UUID 生成

## 运行项目

1. **确保 PostgreSQL 数据库已启动并可访问**
2. **配置环境变量**（按照上述步骤）
3. **运行项目**：
   ```bash
   cargo run
   ```

## 数据库连接

项目已集成 PostgreSQL 数据库连接池，支持：

- 连接池管理
- 连接超时配置
- 自动重连
- 连接健康检查

数据库连接池在应用启动时自动创建，并通过 Axum 的 `Extension` 层注入到各个路由处理函数中。

## API 测试

启动服务后，可以访问以下端点测试数据库连接：

```bash
curl http://localhost:3001/api/sys/user/
```

该端点会测试数据库连接并返回模拟用户数据。

## 开发流程

1. **克隆项目后**，首先复制环境变量模板：

   ```bash
   cp .env.example .env
   ```

2. **编辑 `.env` 文件**，填入您的配置

3. **永远不要提交 `.env` 文件**到版本控制

4. **如果需要更新配置模板**，修改 `.env.example` 文件
