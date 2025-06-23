# 📚 rustzen-admin 中文文档中心

---

> 一个现代化的全栈管理系统模板，基于 **Rust (Axum)** 和 **React (Vite + Ant Design)** 构建。为性能、简洁和可扩展性而设计。

[English](./README.md)

---

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Language](https://img.shields.io/badge/lang-Rust%20%7C%20TypeScript-orange.svg)
![Status](https://img.shields.io/badge/status-开发中-yellow.svg)

---

## 🚀 项目简介

**`rustzen-admin`** 是一个集启动套件与架构参考于一体的全栈项目。它提供了一个基于 Rust (Axum & SQLx) 的生产级后端 API 服务，以及一个由 React (Vite & Ant Design) 驱动的现代化前端界面。

专为以下人群设计：

- 热爱 **Rust 的性能与安全**的开发者。
- 寻找**清晰、现代的全栈项目结构**的团队。
- 正在构建**内部工具、仪表盘或 SaaS 产品**的创客。

---

## ✨ 项目特性

- ✅ **全栈框架**: 一个完整的、集成的解决方案，包含 Rust 后端和 React 前端。
- ✅ **现代技术栈**: 基于 **Axum**、**SQLx**、**Vite**、**TailwindCSS** 和 **Ant Design ProComponents** 构建。
- ✅ **身份认证**: 内置开箱即用的、基于 JWT 的身份认证流程。
- ✅ **数据库集成**: 通过 SQLx 与 PostgreSQL 集成，并包含命令行迁移工具的配置。
- ✅ **容器化支持**: 包含多阶段 `Dockerfile` 和 `docker-compose.yml`，简化开发与部署。
- ✅ **高效工具链**: `justfile` 提供了覆盖整个项目的流线型命令 (`dev`, `build`, `clean`)。
- ✅ **易于扩展**: 模块化的结构设计，可以轻松添加新功能、WebSocket 端点，甚至 Tauri 桌面应用。

---

## ⚙️ 技术栈

| 层级       | 技术选型                                                                                                 |
| :--------- | :------------------------------------------------------------------------------------------------------- |
| **后端**   | Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx), PostgreSQL  |
| **前端**   | React, TypeScript, Vite, TailwindCSS, [Ant Design ProComponents](https://procomponents.ant.design/), SWR |
| **认证**   | JWT (JSON Web Tokens)                                                                                    |
| **工具链** | `just`, `pnpm`, Docker                                                                                   |
| **桌面端** | [Tauri](https://tauri.app/) (可选)                                                                       |

---

## 📦 目录结构

```text
rustzen-admin/
├── backend/         # Rust (Axum) API 服务
├── frontend/        # React (Vite) 管理后台界面
├── desktop/         # (可选) Tauri 桌面外壳
├── docker/          # Docker 配置文件
├── docs/            # 项目文档
├── justfile         # 项目的命令运行器
└── README.md
```

---

## 🛠️ 快速开始

### 环境要求

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v24+) 及 `pnpm`
- [Just](https://github.com/casey/just) 命令运行器
- [Docker](https://www.docker.com/get-started) (用于数据库)

### 安装与启动

1.  **克隆仓库:**

    ```bash
    git clone https://github.com/idaibin/rustzen-admin.git
    cd rustzen-admin
    ```

2.  **设置环境变量:**
    为后端复制示例文件，其中包含 `DATABASE_URL`。

    ```bash
    cp backend/.env.example backend/.env
    ```

3.  **安装前端依赖:**

    ```bash
    cd frontend && pnpm install && cd ..
    ```

4.  **启动项目:**
    启动数据库、后端和前端最简单的方式是使用 `just`。

    ```bash
    # 该命令会使用 docker-compose 启动数据库，
    # 然后以热重载模式启动后端和前端。
    just dev
    ```

    应用将在 `http://localhost:5173` 上可用。

---

## 🧪 API 测试

我们推荐使用 **VSCode REST Client** 进行 API 测试：

1. **安装插件**：在 VSCode 扩展中搜索 "REST Client"
2. **打开测试文件**：`docs/api/api.http`
3. **发送请求**：点击 HTTP 请求上方的 "Send Request"
4. **查看响应**：结果会在新标签页中显示

**主要优势**：

- ✅ 与 VSCode 集成
- ✅ 通过 Git 版本控制
- ✅ 适合个人开发
- ✅ 无需额外软件

查看完整指南：[`docs/api/rest-client.md`](docs/api/rest-client.md)

---

## 🤝 参与贡献

欢迎贡献！请阅读我们的 [**贡献指南**](./docs/development/CONTRIBUTING.md) 来开始。

---

## 📄 开源协议

本项目采用 MIT 协议。详情请见 [LICENSE.md](./LICENSE.md) 文件。

---

⸻

## 📚 功能模块实现状态

### 🔐 认证系统 ✅ **已实现**

| 功能           | 状态    | 说明                       |
| -------------- | ------- | -------------------------- |
| 用户登录       | ✅ 完成 | JWT 令牌认证，密码加密验证 |
| 用户注册       | ✅ 完成 | 支持用户名/邮箱冲突检测    |
| 获取用户信息   | ✅ 完成 | 包含角色信息和菜单权限     |
| JWT 认证中间件 | ✅ 完成 | 自动令牌验证和用户状态检查 |
| 密码哈希       | ✅ 完成 | bcrypt 安全密码存储        |

### 🧑‍💼 系统管理 ✅ **核心功能已实现**

| 模块         | 状态    | 说明                          |
| ------------ | ------- | ----------------------------- |
| **用户管理** | ✅ 完成 | CRUD 操作，角色分配，状态管理 |
| **角色管理** | ✅ 完成 | 角色 CRUD，菜单权限分配       |
| **菜单管理** | ✅ 完成 | 树形菜单结构，权限控制        |
| **数据字典** | ✅ 完成 | 字典项管理，选项 API          |
| **操作日志** | ✅ 完成 | 系统日志记录和查询            |

### 🔗 Options API ✅ **已实现**

| 功能         | 状态    | 说明                                    |
| ------------ | ------- | --------------------------------------- |
| 统一选项接口 | ✅ 完成 | `/api/system/{resource}/options`        |
| 权限控制     | ✅ 完成 | 原子化权限设计 (`system:roles:options`) |
| 搜索过滤     | ✅ 完成 | 支持 `q`, `limit`, `status` 参数        |
| 响应格式     | ✅ 完成 | 标准 `{ label, value }` 格式            |

### 🏗️ 架构特性 ✅ **已实现**

| 特性             | 状态    | 说明                                |
| ---------------- | ------- | ----------------------------------- |
| **模块化架构**   | ✅ 完成 | Repository-Service-Routes 三层架构  |
| **统一错误处理** | ✅ 完成 | `ServiceError` 枚举，业务错误码映射 |
| **API 响应格式** | ✅ 完成 | 统一 `ApiResponse<T>` 包装          |
| **数据库迁移**   | ✅ 完成 | 完整的数据库表结构和关系            |
| **国际化代码**   | ✅ 完成 | 全英文注释和错误消息                |
| **日志系统**     | ✅ 完成 | tracing 框架，结构化日志            |
| **CORS 配置**    | ✅ 完成 | 跨域支持，开发友好                  |

---

## 🔄 规划中功能

| 模块                  | 状态      | 说明                   |
| --------------------- | --------- | ---------------------- |
| 📁 **文件上传**       | 🔄 规划中 | 支持本地 / S3 上传     |
| ⚙️ **系统设置**       | 🔄 规划中 | 通用配置管理，主题切换 |
| 📡 **WebSocket 推送** | ⏳ 可选   | 实时通知和消息推送     |
| 🖥️ **Tauri 客户端**   | ⏳ 可选   | 桌面端管理界面         |
| 📊 **仪表板**         | 🔄 规划中 | 数据统计和可视化图表   |
| 🌍 **多语言支持**     | 🔄 规划中 | i18n 国际化框架        |
| 🎨 **主题系统**       | 🔄 规划中 | 暗黑模式，自定义主题   |

---

## 🧱 技术架构概览

```text
            +--------------------+       +------------------+
   用户端 → | React 管理前端      | →     | Rust API (Axum) |
            +--------------------+       +------------------+
                      ↓                           ↓
            +--------------------+       +------------------+
            | Ant Design Pro     |       | SQLx + PostgreSQL|
            | TailwindCSS        |       | JWT + bcrypt     |
            | SWR + TypeScript   |       | tracing + tokio  |
            +--------------------+       +------------------+
```

**核心优势**：

- 🦀 **Rust 后端**：内存安全，高性能，类型安全
- ⚛️ **React 前端**：现代化组件库，响应式设计
- 🗄️ **PostgreSQL**：ACID 事务，强一致性
- 🔐 **RBAC 权限**：基于角色的访问控制
- 📖 **完整文档**：API 文档，架构说明

---

## 📖 项目文档

- [🏗️ 架构设计](./docs/architecture.md) - 系统模块和技术架构
- [📋 API 文档](./docs/api/) - 完整的接口文档
- [⚙️ Options API](./docs/api/options-api.md) - 下拉选项接口规范
- [🛠️ 开发指南](./docs/development/) - 贡献指南和开发规范

---

## 🎯 项目目标

这个项目的目标是成为 Rust 生态中的 **现代化管理后台模板**，提供：

1. **开箱即用**：完整的 RBAC 权限系统和基础功能
2. **生产就绪**：企业级的代码质量和安全标准
3. **易于扩展**：清晰的模块化架构，便于功能扩展
4. **最佳实践**：展示 Rust + React 全栈开发的最佳实践

---

🙏 **鸣谢与参考**
• [Axum](https://github.com/tokio-rs/axum) - 现代化的 Rust Web 框架
• [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
• [Ant Design Pro](https://procomponents.ant.design/) - 企业级前端组件库
• [Tauri](https://tauri.app/) - 跨平台桌面应用框架
• 社区所有 Rust、React 的开源项目与开发者 🙌

⸻

由 [idaibin] 开发，致力于打造可落地、可维护、可成长的 Rust 全栈系统工程模板 🦀

---
