# rustzen-admin

Rust + React 管理后台模板。当前仓库是迁移中的 monorepo，后端在 `server/`，前端在 `web/`。

## 规范入口

- [AGENTS.md](./AGENTS.md): 协作规则和当前仓库约定
- [docs/architecture.md](./docs/architecture.md): 项目规范和分层约定

## 常用命令

```bash
just dev-server # 只启动后端
just dev-web    # 只启动前端
just check      # 后端 check + 前端 lint
just build      # 构建后端和前端
```

## 目录

```txt
.
├── server/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
│       ├── features/
│       │   └── user/
│       │       ├── mod.rs
│       │       ├── handler.rs
│       │       ├── service.rs
│       │       ├── repo.rs
│       │       └── types.rs
│       ├── infra/
│       ├── common/
│       └── middleware/
├── web/
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── pnpm-workspace.yaml
```
