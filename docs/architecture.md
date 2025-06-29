# 🏗️ rustzen-admin 功能架构文档（Feature Architecture）

本项目为 rustzen-admin，采用 Rust（Axum）+ React（Vite）+ Zustand + SWR + TailwindCSS + Ant Design ProComponents 技术栈构建，目标是构建一套完整的、可部署的现代化全栈管理后台系统。支持扩展 Web3、Tauri 客户端模块，兼顾稳定性、性能与扩展性。

> 💡 **开发指南**: 如需了解开发流程、代码规范和技术架构，请参考 [Contributing Guide](./development/CONTRIBUTING.md)

---

## 📁 系统模块（System）

| 模块     | 状态      | 子功能                 | 描述                   |
| -------- | --------- | ---------------------- | ---------------------- |
| 用户管理 | ✅ 已实现 | 用户增删改查、重置密码 | 系统用户基础信息管理   |
| 角色管理 | ✅ 已实现 | 角色维护、权限绑定     | RBAC 权限系统核心      |
| 菜单管理 | ✅ 已实现 | 菜单结构、权限控制     | 路由与前端菜单控制     |
| 字典管理 | ✅ 已实现 | 枚举配置、字典项维护   | 通用字段配置项         |
| 日志管理 | ✅ 已实现 | 登录日志、操作日志     | 审计、调试与记录功能   |
| 部门管理 | 🔄 规划中 | 层级组织结构           | 用户组织归属与权限划分 |
| 参数设置 | 🔄 规划中 | 系统键值参数配置       | 系统运行控制项         |
| 文件管理 | ⏳ 可选   | 上传、预览、删除       | 静态资源存储支持       |

---

## 🔐 权限与身份模块（Auth）

| 功能         | 状态      | 描述                        |
| ------------ | --------- | --------------------------- |
| 登录认证     | ✅ 已实现 | 用户登录、JWT Token 发放    |
| 用户注册     | ✅ 已实现 | 支持用户注册和冲突检测      |
| 当前用户信息 | ✅ 已实现 | 获取当前用户、权限、菜单    |
| 权限中间件   | ✅ 已实现 | 后端权限校验 + 前端按钮控制 |
| Token 刷新   | 🔄 规划中 | 可选功能，自动续签          |

---

## 📝 表单与流程模块（拓展）

| 模块     | 状态    | 子功能               | 描述               |
| -------- | ------- | -------------------- | ------------------ |
| 动态表单 | ⏳ 可选 | 配置表单字段、布局   | 用于低代码场景     |
| 审批流程 | ⏳ 可选 | 流程设计器、节点配置 | 审批流/工作流设计  |
| 通知中心 | ⏳ 可选 | 系统/审批提醒消息    | 消息推送与读取状态 |

---

## 🧰 工具类模块（可选）

| 功能     | 状态    | 描述                              |
| -------- | ------- | --------------------------------- |
| 缓存管理 | ⏳ 可选 | Redis 缓存操作、清理、状态监控    |
| 系统监控 | ⏳ 可选 | 资源监控：CPU、内存、数据库连接等 |
| 定时任务 | ⏳ 可选 | Crontab 执行器与执行记录          |
| 多语言   | ⏳ 可选 | 多语言配置与切换                  |
| 暗色模式 | ⏳ 可选 | UI 主题切换，支持 dark class      |

---

## 🖥️ 客户端控制模块（Tauri 桌面拓展）

| 类型     | 状态    | 功能                 | 描述                       |
| -------- | ------- | -------------------- | -------------------------- |
| 控制端   | ⏳ 可选 | 添加设备、页面、场景 | 控制远端显示端             |
| 显示端   | ⏳ 可选 | 接收指令、展示页面   | 实时响应控制端信号         |
| 通信协议 | ⏳ 可选 | HTTP + WebSocket     | 控制端与显示端双向通信桥梁 |

---

## 📌 模块结构建议（Modules）

每个功能模块建议具有以下结构：

```
modules/
├── user/
│   ├── model.rs      # 类型结构体定义（序列化）
│   ├── repo.rs       # 数据库操作逻辑
│   ├── service.rs    # 业务组合逻辑
│   ├── routes.rs     # 路由注册与 handler
│   └── mod.rs        # 模块导出
```

前端对应：

```
src/pages/system/user/
├── index.tsx         # 列表页
├── form.tsx          # 编辑弹窗
├── service.ts        # 接口定义
├── hook.ts           # 业务逻辑封装
└── types.ts          # 类型定义（或全局 declare）
```

> 🔧 **技术实现细节**: 查看 [Contributing Guide](./development/CONTRIBUTING.md) 了解具体的开发规范和技术架构

---

## ✅ 当前开发状态 (v0.1.0)

### 🎯 已完成功能

1. ✅ **用户管理** (user) - 完整的 CRUD 操作，角色分配
2. ✅ **登录/注册模块** (auth) - JWT 认证，密码加密
3. ✅ **角色权限管理** (role, menu) - RBAC 权限控制
4. ✅ **字典管理** (dict) - 基础数据字典功能
5. ✅ **日志系统** (log) - 操作日志记录和查询
6. ✅ **Options API** - 统一的下拉选项接口

### 🚀 推荐开发优先级（v1.0）

1. 🔄 **部门管理** (department) - 组织架构支持
2. 🔄 **参数设置** (settings) - 系统配置管理
3. ⏳ **文件上传** (upload) - 文件存储功能
4. ⏳ **系统监控** (monitor) - 资源监控面板
5. ⏳ **客户端控制** (client/tauri) - 桌面应用支持

### 📋 后续迭代规划

- 缓存管理、定时任务
- 审批流程、动态表单
- 多语言、主题切换
- WebSocket 实时通信

---

## 🧠 未来扩展建议

### 🌐 Web3 集成

- 钱包登录（MetaMask、WalletConnect）
- 合约交互和资产管理
- 区块链数据探索

### 📱 多终端支持

- **Tauri 桌面应用** - 跨平台原生体验
- **移动端 PWA** - 响应式 Web 应用
- **Electron 替代方案** - 传统桌面应用

### 🔌 插件化架构

- 模块动态加载
- 权限自动接入
- 第三方集成支持

---

> 💡 **开发建议**: 本文档为 AI 辅助开发的功能规划索引，建议结合 Cursor、Copilot、CodeWhisperer 等工具进行模块生成和语义补全。详细的开发流程请参考 [Contributing Guide](./development/CONTRIBUTING.md)。
