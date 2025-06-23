# rustzen-admin：Rust + React 管理后台模板

开发管理后台时，经常需要重复实现相同的功能模式：用户认证、权限管理、CRUD 操作等基础功能。**rustzen-admin** 是一个开源模板，使用 Rust 作为后端、React 作为前端，提供这些基础功能的完整实现。

## 当前实现功能

这是一个正在开发中的模板项目，以下是目前已经实现的功能：

### ✅ 后端 (Rust + Axum + PostgreSQL)

- **用户认证**：基于 JWT 的登录系统，使用 bcrypt 密码加密
- **权限控制**：用户、角色、权限的完整管理体系
- **菜单系统**：支持层级结构的菜单管理和权限控制
- **操作日志**：基础的操作记录和安全审计功能
- **选项接口**：统一的下拉框和选择器数据接口

### ✅ 前端 (React 19 + TypeScript + Ant Design)

- **认证流程**：完整的登录/登出和令牌管理
- **用户管理**：用户的增删改查和角色分配
- **角色管理**：权限分配和菜单访问控制
- **菜单管理**：树形结构编辑和组织管理
- **响应式界面**：基于 Ant Design 的响应式设计

### 🔄 开发中功能

- 部门/组织架构管理
- 系统配置管理
- 文件上传功能

## 技术架构

### 后端结构

```
backend/src/features/
├── auth/           # 认证和授权
├── system/
│   ├── user/      # 用户管理
│   ├── role/      # 角色管理
│   ├── menu/      # 菜单管理
│   └── log/       # 操作日志
```

每个功能模块都遵循一致的分层模式：

- **Model**：数据结构和验证
- **Repository**：数据库操作（使用 SQLx）
- **Service**：业务逻辑处理
- **Routes**：HTTP 请求处理

### 前端结构

```
frontend/src/
├── pages/          # 页面组件
├── services/       # API 通信
├── types/          # TypeScript 类型定义
└── layouts/        # 布局组件
```

## 技术选型理由

### 为什么选择 Rust 作为后端？

- **内存安全**：编译期检查避免常见运行时错误
- **性能表现**：相比 Node.js 等方案资源占用更少
- **类型安全**：强类型系统帮助早期发现问题
- **生态支持**：SQLx 数据库操作、Axum Web 框架

### 为什么选择 React + TypeScript？

- **开发体验**：前端开发者熟悉的开发模式
- **类型安全**：与后端模型的端到端类型检查
- **组件生态**：Ant Design 提供企业级组件
- **现代工具**：Vite 提供快速的开发构建

## 快速开始

### 环境要求

```bash
# 必需工具
rust --version     # >= 1.70
node --version     # >= 20
docker --version   # >= 20.10
```

### 安装步骤

```bash
git clone https://github.com/idaibin/rustzen-admin.git
cd rustzen-admin

# 配置环境变量
cp backend/.env.example backend/.env

# 安装 just 命令工具
cargo install just

# 启动开发环境
just dev
```

启动后包含：

- PostgreSQL 数据库（Docker）
- Rust 后端服务 `http://localhost:8000`
- React 前端应用 `http://localhost:5173`

默认登录账号：`admin` / `admin123`

## 开发体验

### 统一命令管理

```bash
just dev       # 启动开发环境
just build     # 构建生产版本
just test      # 运行测试
just clean     # 清理构建缓存
```

### API 测试

项目包含 REST Client 文件，可直接在 VSCode 中测试 API：

```http
### 登录
POST http://localhost:8000/api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

## 性能表现

基础负载测试显示在典型管理后台使用场景下有合理的性能表现：

| 指标       | 结果               |
| ---------- | ------------------ |
| 冷启动时间 | ~1 秒              |
| 内存占用   | ~30-50MB           |
| 并发处理   | 可良好处理数百并发 |

_注：这是开发环境下的指标，生产环境性能取决于部署配置。_

## 适用场景

### 适合的项目类型：

- 企业内部管理工具
- SaaS 产品后台管理
- 内容管理系统
- 中小规模应用系统

### 建议考虑其他方案的情况：

- 需要快速原型开发（有学习成本）
- 团队没有 Rust 经验且无学习时间
- 项目需要大量实时功能
- 需要成熟生态和大量插件支持

## 参与贡献

欢迎参与项目贡献！特别需要帮助的领域：

- **文档完善**：改进安装指南和教程
- **测试扩展**：增加测试覆盖率
- **界面优化**：提升用户界面体验
- **功能增强**：添加常用的管理后台功能

### 贡献流程

1. Fork 仓库
2. 创建功能分支
3. 进行开发修改
4. 如适用，添加测试
5. 提交 Pull Request

## 当前局限性

坦诚说明这个模板目前还没有包含的功能：

- 无实时通知功能
- 文件上传处理较简单
- 前端错误处理较基础
- 无国际化支持
- 移动端优化有限

## 发展规划

### 短期目标

- 完成部门管理功能
- 改进错误处理机制
- 增加全面的测试
- 完善项目文档

### 长期规划

- WebSocket 实时功能支持
- 插件架构设计
- 移动端 PWA 增强
- 高级数据可视化组件

## 学习资源

如果你对 Rust 或这个技术栈还不熟悉，推荐以下学习资源：

**Rust Web 开发：**

- [Rust 程序设计语言](https://kaisery.github.io/trpl-zh-cn/)
- [Axum 文档](https://docs.rs/axum/)
- [SQLx 指南](https://github.com/launchbadge/sqlx)

**React TypeScript：**

- [React 文档](https://react.dev/)
- [TypeScript 手册](https://www.typescriptlang.org/docs/)
- [Ant Design 文档](https://ant.design/)

## 相关链接

- **项目仓库**：https://github.com/idaibin/rustzen-admin
- **项目文档**：https://github.com/idaibin/rustzen-admin/tree/main/docs
- **问题反馈**：https://github.com/idaibin/rustzen-admin/issues
- **讨论交流**：https://github.com/idaibin/rustzen-admin/discussions

---

这个模板旨在为管理后台开发提供可靠的基础，同时展示 Rust 在 Web 后端开发中的有效应用。欢迎你的反馈和贡献！

**你在 Rust Web 开发方面有什么经验？遇到过哪些挑战或收获？**

**标签**：#Rust #React #Web 开发 #开源项目 #管理后台
