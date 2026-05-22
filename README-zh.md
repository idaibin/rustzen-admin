# rustzen-admin

面向 Rust 全栈管理后台系统的结构化 monorepo 基座。

> `rustzen-admin` 将 Axum 后端、React 前端和仓库级文档组织在同一个代码库中，强调清晰边界、可维护性，以及对 AI 协作友好的工程结构。

## 概览

`rustzen-admin` 是一个面向真实项目的开源全栈管理后台基础仓库，而不只是孤立的 UI 演示。

仓库采用 monorepo 组织方式：

- `zen-core/` — 共享的 Rust 认证与权限能力 crate
- `zen-server/` — Rust 后端应用
- `zen-web/` — React 前端应用
- `deploy/` — 部署资产和发布支持文件
- `docs/` — 仓库级架构与开发规范文档

## 文档与导航

- **英文主文档** → [README.md](./README.md)（仓库概览和入口链接）
- **文档体系索引** → [docs/README.md](./docs/README.md)
- **仓库架构** → [docs/architecture.md](./docs/architecture.md)（目录结构、边界、命令入口）
- **协作规则** → [AGENTS.md](./AGENTS.md)

## 说明

- 英文文档 [README.md](./README.md) 为主要维护版本，详细结构和规则以 [docs/README.md](./docs/README.md) 为入口。
- 本文件仅作为中文简介，引导中文读者快速了解项目定位和文档入口。
