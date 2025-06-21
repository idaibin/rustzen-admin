📚 项目文档：rustzen-admin

rustzen-admin 是一套基于 Rust + React 的全栈后台管理系统模板，旨在提供现代化、可扩展、高性能的全栈开发架构，适用于中后台管理系统、内部工具平台或 SaaS 项目的起步开发。

---

## 📌 项目定位

- **项目名称**：rustzen-admin
- **项目目标**：打造一套清晰、稳健、现代的 Rust 全栈后台管理系统模板，适合个人开发者、中小团队使用，也适合作为个人技术展示和组件积累的基础工程。
- **关键特性**：
  - 前后端分离开发，支持代理与集成打包
  - Rust 端完全可部署为独立二进制
  - 前端采用现代化 Vite 构建体系
  - 支持桌面客户端扩展（Tauri）

---

## 🧱 技术栈说明

### 后端（backend/）

| 组件       | 技术选型    | 说明                  |
| ---------- | ----------- | --------------------- |
| 语言/框架  | Rust + Axum | 高性能异步 Web 框架   |
| 数据库访问 | SQLx        | 异步、支持 PostgreSQL |
| 日志系统   | tracing     | 结构化日志记录        |
| 认证授权   | JWT         | 未来支持 OAuth2       |
| 数据库迁移 | sqlx-cli    | 数据库版本管理        |

### 前端（frontend/）

| 组件     | 技术选型                   | 说明                  |
| -------- | -------------------------- | --------------------- |
| 构建工具 | Vite                       | 现代化构建工具        |
| 组件库   | Ant Design + ProComponents | 企业级 UI 组件        |
| 样式系统 | TailwindCSS + @emotion/css | 原子化 CSS + 组件样式 |
| 状态管理 | React Hooks + Context      | 可选 Zustand          |
| 接口通信 | Fetch / Axios              | 可配置                |
| UI 特性  | 暗色模式、国际化           | 预留支持              |

### 客户端（desktop/）

| 组件 | 技术选型                     | 说明         |
| ---- | ---------------------------- | ------------ |
| 框架 | Tauri                        | 可选         |
| 用途 | 展示端、多屏控制、离线运行等 | 桌面应用场景 |

---

## 🏗️ 架构模块规划

┌────────────┐ HTTP API ┌────────────┐
│ 前端 UI │ ───────────────────▶ │ 后端 API │
└────────────┘ └────────────┘
▲ │
│ ▼
静态资源部署 PostgreSQL
│
▼
可选嵌入 Tauri

### 后端模块划分（Rust）

• auth/：登录、注册、JWT 验证
• user/：用户管理、分页、状态切换
• role/：角色管理、权限绑定
• menu/：菜单与权限路由
• settings/：系统设置（预留）
• log/：接口访问日志、操作日志
• utils/：工具库（分页、加密等）
• middleware/：CORS、鉴权中间件

### 前端模块划分（React）

• pages/login：登录页
• pages/dashboard：主页与概览
• pages/system/users：用户管理页
• pages/system/roles：角色管理
• pages/system/logs：日志列表
• components/：公共组件（表单、弹窗等）
• hooks/：通用逻辑（如 useRequest）
• utils/：格式化函数、权限判断等

⸻

🚦 开发流程建议 1. 克隆项目并初始化依赖 2. 配置 .env 和数据库参数 3. 启动后端服务，执行数据库迁移 4. 启动前端服务，配置代理到后端 5. 开发模块功能，前后端联调 6. 使用脚本或手动打包后端与前端资源

⸻

## 📁 目录结构概览

rustzen-admin/
├── backend/ # Rust 后端服务
│ ├── src/
│ ├── migrations/
│ └── Cargo.toml
├── frontend/ # React 前端管理系统
├── desktop/ # 可选 Tauri 客户端
├── docker/ # Docker 配置（可选）
├── docs/ # 当前文档目录
├── scripts/ # 启动 / 构建脚本
├── .env.example # 环境变量模板
└── README.md

⸻

## 📌 后续规划（Roadmap）

• 菜单权限配置 UI
• JWT 刷新机制 + 多端登录支持
• 上传与文件管理模块
• 系统设置（Logo、标题等）配置面板
• 支持 WebSocket 的推送机制
• Web3 钱包登录（可选扩展）
• 移动端适配与响应式支持（中期）
• 完整的集成测试 / E2E 测试
• GraphQL 接口支持（可选）

⸻

## 📖 文档说明

该文档旨在为开发者或 AI 辅助工具提供项目上下文，未来还将细分模块文档，包括：
• auth.md：登录认证模块说明
• api.md：后端接口设计
• deploy.md：部署方案与环境配置
• db-schema.md：数据库结构文档
• tauri.md：客户端运行机制与接口设计

欢迎你根据实际开发进度进行完善与补充。
