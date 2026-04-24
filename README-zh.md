# rustzen-admin

面向 Rust 全栈管理后台系统的结构化 monorepo 基座。

> `rustzen-admin` 将 Axum 后端、React 前端和仓库级文档组织在同一个代码库中，强调清晰边界、可维护性，以及对 AI 协作友好的工程结构。

## 概览

`rustzen-admin` 是一个面向真实项目的开源全栈管理后台基础仓库，而不只是孤立的 UI 演示。

仓库采用 monorepo 组织方式：

- `zen-core/` 存放共享的 Rust 认证与权限能力 crate
- `zen-server/` 存放 Rust 后端应用
- `zen-web/` 存放 React 前端应用
- `deploy/` 存放部署资产和发布支持文件
- `docs/` 存放仓库级架构与开发规范文档
- 根目录保留共享命令、工作区元信息和协作入口文档

这种布局让后端、前端和仓库规则保持明确边界，使整个代码库更容易理解、评审和持续演进。

## 仓库结构

```txt
.
├── zen-core/
│   ├── Cargo.toml
│   └── src/
│       ├── auth/
│       ├── permission/
│       ├── error.rs
│       └── lib.rs
├── zen-server/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
│       ├── features/
│       │   ├── auth/
│       │   ├── dashboard/
│       │   └── system/
│       ├── infra/
│       ├── common/
│       └── middleware/
├── zen-web/
│   └── src/
│       ├── routes/
│       ├── api/
│       ├── components/
│       │   └── base-layout/
│       └── store/
├── deploy/
│   ├── sql/
│   │   └── repair_menu_schema.sql
│   ├── binary.Dockerfile
│   ├── release.Dockerfile
│   ├── runtime.Dockerfile
│   └── rustzen-admin.service
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## 文档入口

- [CHANGELOG.md](./CHANGELOG.md)：版本说明与破坏性变更（升级请先读）
- [docs/README.md](./docs/README.md)：文档体系入口与放置规则
- [AGENTS.md](./AGENTS.md)：仓库级协作规则
- [zen-server/AGENTS.md](./zen-server/AGENTS.md)：后端入口指南
- [zen-web/AGENTS.md](./zen-web/AGENTS.md)：前端入口指南
- [docs/architecture.md](./docs/architecture.md)：仓库结构、边界与命令入口
- [docs/project-map.md](./docs/project-map.md)：入口文件与高频改动路径索引
- [docs/backend-guide.md](./docs/backend-guide.md)：后端分层、命名、数据库与错误处理规则
- [docs/frontend-guide.md](./docs/frontend-guide.md)：前端路由、请求、状态与 UI 规则
- [docs/deployment-guide.md](./docs/deployment-guide.md)：部署与运行时配置规则
- [docs/permission-guide.md](./docs/permission-guide.md)：权限模型与使用规则

## 常用命令

```bash
just dev-server
just dev-web
just check
just build
just build-binary
just build-release
just build-image
```

## 说明

- `README.md` 与 `AGENTS.md` 仅作为轻量入口文档。
- 产品定义、UI 设计更新、开发流程与进度日志等持续执行信息，按需维护在 `docs/` 下对应目录，不作为默认入口阅读内容。
