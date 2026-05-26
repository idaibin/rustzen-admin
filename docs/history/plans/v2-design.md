# 1. 当前结论

rustzen-admin v2 不再继续强化 PostgreSQL-first、Web-first、传统后台模板路线。

v2 当前阶段的目标是：

> 让项目变成一个 AI-first、Local-first、SQLite-first、低复杂度、易运行、易维护的 Rust 本地运行框架。
>

这里的 AI-first 不是指集成 AI 功能，而是指：

> 项目结构、文档、模块边界和运行方式要优先适配 AI Coding，便于 ChatGPT、Cursor、Claude Code、Codex 等工具长期理解、修改和维护项目。
>

当前重点不是做 AI 产品，也不是做复杂平台，而是降低项目复杂度，让项目更容易持续迭代。

---

# 2. v2 核心目标

## 2.1 AI-first Engineering

项目结构要尽量让 AI 容易理解。

要求：

- 模块边界清晰
- 目录结构稳定
- 命名直观
- 减少隐藏逻辑
- 减少过度抽象
- 减少全局工具类和 god module
- 关键目录保留 [AGENTS.md](http://AGENTS.md) 或说明文档

## 2.2 Local-first

项目默认应该本地直接运行。

目标：

```bash
cargo run
```

或最少命令即可启动核心服务。

当前阶段避免依赖：

- PostgreSQL
- Redis
- Kafka
- Docker Compose
- 微服务

## 2.3 SQLite-first

v2 默认使用 SQLite。

原因：

- 零外部数据库依赖
- 单文件存储
- 本地开发简单
- 适合单机部署
- 适合 Desktop 后续演进
- 降低 AI Coding 的环境复杂度

PostgreSQL 暂时不作为 main 默认路线。

## 2.4 Low Complexity

v2 不追求大而全。

优先级：

1. 能跑起来
2. 结构清晰
3. AI 能看懂
4. 人能维护
5. 后续容易扩展

---

# 3. 当前不做什么

v2 当前阶段不做：

- AI Agent 平台
- AI OS
- MCP 平台
- 多 Agent 编排
- 复杂插件市场
- 分布式 Runtime
- 多数据库同时兼容
- PostgreSQL Provider
- 企业级多租户
- Redis / Kafka / 微服务

这些不是否定未来价值，而是当前阶段不引入，避免过早复杂化。

---

# 4. 分支策略

## 4.1 main

main 代表 v2 新方向：

- SQLite-first
- Local-first
- AI-first Engineering
- 允许 breaking changes
- 不强行兼容旧 PG 架构

## 4.2 legacy/pg-admin

保留当前 PostgreSQL-first 版本。

```bash
git checkout -b legacy/pg-admin
git push origin legacy/pg-admin
```

用途：

- 保留旧版本
- 方便回滚
- 作为历史产品线归档

后续只维护：

- bugfix
- security

不再新增核心功能。

## 4.3 legacy/react-router

原有 dev-react-router / feat-react-router 统一整理为：

```
legacy/react-router
```

避免分支语义混乱。

## 4.4 feature/*

新功能开发统一使用：

```
feature/xxx
```

例如：

```
feature/sqlite-storage
feature/runtime-layout
feature/readme-v2
```

---

# 5. main 分支改造方向

main 分支的第一目标不是新增功能，而是完成基础工程转向。

核心改造：

1. 移除 PostgreSQL 默认依赖
2. 引入 SQLite 默认存储
3. 简化本地启动流程
4. 重写 README 定位
5. 整理目录结构
6. 明确模块边界
7. 保留旧后台核心能力，但降低传统 RBAC 叙事

---

# 6. SQLite-first 落地方案

## 6.1 当前阶段

只实现 SQLite。

不要同时做：

- storage-pg
- 多数据库抽象
- 双 migration
- 复杂 provider system

## 6.2 配置建议

示例：

```toml
[storage]
driver = "sqlite"
database_url = "data/rustzen.db"
```

或环境变量：

```bash
RUSTZEN_STORAGE=sqlite
RUSTZEN_SQLITE_PATH=./data/rustzen.db
```

## 6.3 Migration

目录建议：

```
migrations/
└── sqlite/
    ├── 0001_init.sql
    ├── 0002_auth.sql
    └── 0003_workspace.sql
```

当前不需要：

```
migrations/postgres/
```

等需要 PG 时再新增。

## 6.4 Storage 原则

可以先不抽象复杂 provider。

当前只需要保持代码边界清晰：

```
crates/storage
```

负责：

- SQLite connection
- migration
- transaction
- repository helper

不要一开始写复杂的：

```rust
trait DatabaseProvider
trait QueryExecutor
trait StorageBackend
```

避免过度设计。

---

# 7. 目录结构调整方案

建议目标结构：

```
apps/
├── server
├── web
└── desktop        # 可暂时为空或后续引入

crates/
├── storage
├── auth
├── workspace
├── capability
├── runtime
└── config

docs/
├── project-map.md
├── architecture.md
├── deployment-guide.md
└── ai-coding-rules.md
```

## 说明

`apps/` 放可运行应用：

- server：后端服务
- web：前端应用
- desktop：后续桌面端

`crates/` 放共享能力：

- storage：SQLite 存储
- auth：登录认证
- workspace：工作空间边界
- capability：能力边界
- runtime：运行上下文
- config：配置加载

当前阶段不需要拆太细。

不要拆成：

```
workspace-core
workspace-service
workspace-repo
workspace-api
```

这会增加 AI 理解成本。

---

# 8. 权限与 Capability 边界

当前不急着彻底推翻 RBAC。

## 8.1 短期

保留原有：

- 用户
- 角色
- 权限
- 菜单

但重新理解为：

> 系统能力边界的一部分。
>

## 8.2 中期

逐步引入 capability 命名：

```
user.read
user.write
role.manage
workspace.manage
system.config
```

后续如果 Desktop 出现，再扩展：

```
filesystem.read
shell.exec
```

当前不要一开始做复杂能力权限系统。

---

# 9. README 与项目定位调整

README 需要从：

```
Rust + React Admin Template
```

调整为：

```
AI-first, local-first Rust admin/runtime framework.
```

推荐英文描述：

```
rustzen-admin is an AI-first and local-first Rust admin/runtime framework,
designed for simple local development, SQLite-first storage,
and long-term AI-assisted maintenance.
```

推荐中文描述：

```
rustzen-admin 是一个 AI-first、Local-first 的 Rust 管理端 / 运行框架，
默认采用 SQLite，强调本地易运行、结构清晰、低复杂度，以及适合 AI 长期协作开发。
```

---

# 10. 执行优先级

## P0：立即执行

- 创建 `legacy/pg-admin`
- 整理旧分支
- main 确认为 v2 方向
- README 改定位
- SQLite 作为默认存储
- 本地启动流程简化

## P1：核心重构

- 整理 apps / crates 结构
- 抽出 storage
- 抽出 config
- 明确 auth / workspace / capability 边界
- 更新 project-map
- 更新 architecture

## P2：增强可维护性

- 增加 [AGENTS.md](http://AGENTS.md)
- 增加 [ai-coding-rules.md](http://ai-coding-rules.md)
- 增加 docs-rule
- 明确模块修改规则
- 减少 util/common 滥用

## P3：以后再做

- desktop
- plugin
- sync
- PostgreSQL Provider
- enterprise deployment

---

# 11. 验收标准

v2 第一阶段完成后，应满足：

## 运行

- 不依赖 PostgreSQL
- 不依赖 Redis
- 不依赖 Docker Compose
- 本地可直接启动

## 架构

- apps / crates 边界清晰
- storage 默认 SQLite
- README 与项目定位一致
- 文档能解释当前方向

## AI-friendly

- AI 能通过 project-map 快速理解项目
- 每个核心 crate 职责清晰
- 没有明显 god module
- 没有过度抽象 provider

## 可持续

- legacy/pg-admin 已保留
- main 可大胆重构
- 后续 PG 可作为扩展再引入
