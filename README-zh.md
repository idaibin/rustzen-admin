# 📚 rustzen-admin 中文文档中心

---

> 一个现代化的全栈管理系统模板，基于 **Rust (Axum)** 和 **React (Vite + Ant Design)** 构建。为性能、简洁和可扩展性而设计。

[English](./README.md)

---

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Language](https://img.shields.io/badge/lang-Rust%20%7C%20TypeScript-orange.svg)
![Status](https://img.shields.io/badge/status-开发中-yellow.svg)

---

## 🎯 项目目标

这个项目的目标是成为 Rust 生态中的 **现代化管理后台模板**，提供：

1. **开箱即用**：完整的 RBAC 权限系统和基础功能
2. **代码质量**：良好的代码结构和安全性
3. **易于扩展**：清晰的模块化架构
4. **最佳实践**：展示 Rust + React 全栈开发模式

---

## ⚙️ 技术栈

| 层级       | 技术选型                                         |
| ---------- | ------------------------------------------------ |
| **后端**   | Rust, Axum, SQLx, PostgreSQL, Tracing            |
| **前端**   | React, TypeScript, Vite, Ant Design, TailwindCSS |
| **认证**   | JWT (JSON Web Tokens)                            |
| **工具链** | just, pnpm                                       |

---

## 📦 目录结构

```
rustzen-admin/
├── src/              # Rust (Axum) API 服务源码
├── web/              # React (Vite) 管理后台前端
├── migrations/       # 数据库迁移文件
├── docs/             # 项目文档
├── Cargo.toml        # Rust 依赖配置
├── justfile          # 项目命令运行器
└── README.md
```

---

## 🛠️ 快速开始

### 环境要求

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Node.js](https://nodejs.org/) (v24+) 及 `pnpm`
-   [Just](https://github.com/casey/just) 命令运行器

### 安装与启动

1. **克隆仓库:**

    ```bash
    git clone https://github.com/idaibin/rustzen-admin.git
    cd rustzen-admin
    ```

2. **设置环境变量:**

    ```bash
    cp .env.example .env
    # 编辑 .env 文件，配置数据库连接信息
    ```

3. **安装依赖:**

    ```bash
    # 安装 just 和 Rust 依赖
    cargo install just
    cargo install cargo-watch

    # 安装前端依赖
    cd web && pnpm install && cd ..
    ```

4. **启动项目:**

    ```bash
    just dev
    ```

    应用将在 `http://localhost:5173` 上可用。

---

## 📚 基础功能

-   **认证系统**: JWT 登录、用户信息获取、权限验证
-   **用户管理**: CRUD 操作、角色分配、状态管理
-   **角色管理**: 角色 CRUD、菜单权限分配
-   **菜单管理**: 树形菜单结构、权限控制
-   **数据字典**: 字典项管理、选项 API
-   **操作日志**: 系统日志记录和查询

---

## 📖 项目文档

-   [🏗️ 架构设计](./docs/architecture.md) - 系统模块和技术架构
-   [⚙️ 权限设计](./docs/permissions-guide.md) - 设计和使用说明

---

## 📄 开源协议

本项目采用 MIT 协议。详情请见 [LICENSE.md](./LICENSE.md) 文件。

---

由 [idaibin] 开发，致力于打造可落地、可维护、可成长的 Rust 全栈系统工程模板 🦀

---
