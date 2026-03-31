# 项目规范

> 这份文档只保留架构索引、目录约定和命令入口。

## 目录约定

- `server/src/features/`: Rust 后端业务模块
- `server/src/infra/`: 配置、数据库、JWT、密码、权限等基础设施
- `server/src/common/`: 跨模块复用的工具和类型
- `server/src/middleware/`: Axum 中间件
- `server/migrations/`: 数据库迁移
- `web/`: React 前端
- `docs/`: 规范文档

目标布局：

```txt
.
├── Cargo.toml
├── Cargo.lock
├── pnpm-workspace.yaml
server/
  Cargo.toml
  migrations/
  src/
    features/
      user/
        mod.rs
        handler.rs
        service.rs
        repo.rs
        types.rs
    infra/
    common/
    middleware/
web/
docs/
justfile
```

## 分层约定

- `features/<feature>/mod.rs`: 路由入口，负责组合 handler
- `handler.rs`: HTTP 参数提取和响应返回
- `service.rs`: 业务编排、校验、事务协调
- `repo.rs`: 数据库访问与持久化
- `types.rs`: row/entity 定义、请求体、响应体、查询参数、公共类型

## 权限约定

- 当前默认权限检查模式是 `Require("system:xxx:yyy")`
- `Any([...])` 和 `All([...])` 仅作为未来扩展预留
- 当前项目的路由权限不应以 `Any` 作为默认写法，也不应把通配权限语义混写进 `Any`

## 命名约定

- Rust、数据库字段使用 `snake_case`
- API JSON 和前端类型使用 `camelCase`
- 响应体建议统一使用 `#[serde(rename_all = "camelCase")]`
- feature 方法命名建议遵循：
  - `list_users`
  - `get_user`
  - `create_user`
  - `update_user`
  - `delete_user`

## 约束摘要

- `handler` 只处理请求和响应，不写 SQL
- 新增 feature 统一使用 `mod.rs + handler.rs` 拆分
- `service` 负责编排业务，不直接跨 feature 调 repo
- `repo` 只处理数据库访问
- `types.rs` 作为 feature 类型收敛入口；能直接 `FromRow` 的响应类型可直接查询返回
- 不手改生成文件
- schema、接口、前端类型变更必须同步
- 优先复用已有模块，保持最小改动
- 迁移/序号类文件应保持明确前缀，例如 `0101_...sql`

## 新增 Feature 流程

1. 先创建 `server/src/features/<feature>/mod.rs`、`handler.rs`、`service.rs`、`repo.rs`、`types.rs`
2. 在 `mod.rs` 暴露 `routes()`
3. `types.rs` 顶部先放 row/entity 定义；若查询结果和 API 形状一致，响应类型可直接 `FromRow`
4. `handler.rs` 只接收参数并返回响应
5. `service.rs` 负责编排和校验
6. `repo.rs` 只写 SQL
7. `types.rs` 继续承载该 feature 的请求体、响应体和查询参数
8. 同步更新前端 `web/src/api/<module>/`
9. 最后跑 `cargo check --manifest-path server/Cargo.toml` 和 `just check`

## 常用命令

```bash
just dev-server # 只启动后端
just dev-web    # 只启动前端
just check      # 后端 check + 前端 lint
just build      # 构建后端和前端
```
