# 📋 更新日志

记录 rustzen-admin 项目的重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 规划中

- [ ] JWT 身份认证实现
- [ ] RBAC 权限系统
- [ ] 数据库迁移脚本
- [ ] 用户界面完善
- [ ] 单元测试覆盖

## [0.1.0] - 2025-06-22

### 🎯 首个版本发布

这是 rustzen-admin 的首个公开版本，提供了完整的全栈开发模板。

### ✨ 核心功能

**🦀 后端服务**

- Axum Web 框架 + SQLx 数据库集成
- PostgreSQL 数据库支持
- 模块化架构设计（用户、角色、菜单、字典、日志）
- CORS 和日志中间件
- 环境变量配置管理

**⚛️ 前端应用**

- React 19 + TypeScript 5.8
- Vite 6.3 构建工具
- Ant Design Pro Components 企业级 UI
- TailwindCSS 4.1 样式系统
- SWR 数据获取 + Zustand 状态管理
- 响应式路由系统

**🛠️ 开发工具**

- Docker 容器化开发环境
- justfile 统一命令管理
- 热重载开发体验
- VSCode REST Client API 测试
- ESLint + Prettier 代码规范

### 📚 文档体系

- 完整的项目文档
- API 接口文档和测试用例
- 架构设计说明
- 开发者贡献指南
- Git 提交规范

### 🔧 配置

- MIT 开源协议
- Volta Node.js 版本管理
- TypeScript 严格模式
- 现代化工具链配置

---

## 版本说明

- **主版本号**: 不兼容的 API 修改
- **次版本号**: 向下兼容的功能性新增
- **修订版本号**: 向下兼容的问题修正

---
