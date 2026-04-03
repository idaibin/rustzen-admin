# rustzen-admin

面向 Rust 全栈管理后台系统的结构化 monorepo 基座。

> `rustzen-admin` 将 Axum 后端、React 前端和仓库级文档组织在同一个代码库中，强调清晰边界、可维护性，以及对 AI 协作友好的工程结构。

## 概览

`rustzen-admin` 是一个面向真实项目的开源全栈管理后台基础仓库，而不只是孤立的 UI 演示。

仓库采用 monorepo 组织方式：

- `server/` 存放 Rust 后端应用
- `web/` 存放 React 前端应用
- `docs/` 存放仓库级架构与开发规范文档
- 根目录保留共享命令、工作区元信息和协作入口文档

这种布局让后端、前端和仓库规则保持明确边界，使整个代码库更容易理解、评审和持续演进。

## 为什么是这个仓库

很多管理后台仓库更关注尽快把页面跑起来，但随着功能、权限和数据流逐步增长，维护成本也会迅速上升。

`rustzen-admin` 围绕另一种目标来构建：

- 明确的后端与前端边界
- 面向特性的后端组织方式
- 仓库级文档与协作规则
- 代码、契约与文档的同步变更
- 更适合贡献者与 AI 工具协作的结构

## 仓库结构

```txt
.
├── server/
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
├── web/
│   └── src/
│       ├── routes/
│       ├── api/
│       ├── components/
│       │   └── base-layout/
│       └── stores/
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── pnpm-workspace.yaml
```

## 文档入口

- [AGENTS.md](./AGENTS.md)：仓库级协作规则
- [server/AGENTS.md](./server/AGENTS.md)：后端入口指南
- [web/AGENTS.md](./web/AGENTS.md)：前端入口指南
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
```

## 项目原则

- 清晰的仓库边界
- 最小化的根目录职责
- 单一职责的文档组织
- 优先可维护性，而不是补丁式堆叠
- 明确的架构约定
- 对 AI 协作友好的工程结构

## 当前状态

仓库仍处于持续重构和整理阶段。

当前重点包括：

- 稳定 monorepo 布局
- 对齐后端、前端与文档
- 持续收敛仓库级约定
- 为后续功能增长打下更稳固的长期基础
