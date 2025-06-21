# Rustzen Admin API 文档

## 🌐 基础信息

### Base URLs

- **开发环境**: `http://localhost:3001`
- **生产环境**: `https://your-domain.com`

### API 前缀

所有业务接口都使用 `/api` 作为前缀。

## 🔐 认证方式

使用 JWT (JSON Web Token) 进行身份认证：

```http
Authorization: Bearer <your-jwt-token>
```

## 📋 标准响应格式

所有接口都使用统一的响应格式：

```typescript
interface ApiResponse<T> {
  code: number; // 状态码：200 成功，其他为错误码
  message: string; // 响应消息
  data?: T; // 响应数据（可选）
}
```

### 成功响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "id": 1,
    "userName": "admin"
  }
}
```

### 错误响应示例

```json
{
  "code": 500,
  "message": "数据库连接失败",
  "data": null
}
```

## 🛠️ 系统管理接口

### 用户管理

#### 获取用户列表

```http
GET /api/sys/user
```

**响应示例**:

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "id": 1,
      "userName": "Admin",
      "roleIds": [1]
    }
  ]
}
```

### 角色管理

#### 获取角色列表

```http
GET /api/sys/role
```

### 菜单管理

#### 获取菜单列表

```http
GET /api/sys/menu
```

### 字典管理

#### 获取字典列表

```http
GET /api/sys/dict
```

### 日志管理

#### 获取日志列表

```http
GET /api/sys/log
```

## 🔧 工具接口

### 健康检查

```http
GET /health
```

**响应示例**:

```json
{
  "status": "ok",
  "message": "Rustzen Admin Backend is running",
  "version": "0.1.0"
}
```

### 根路径信息

```http
GET /
```

**响应示例**:

```json
{
  "message": "Welcome to Rustzen Admin API",
  "endpoints": {
    "health": "/health",
    "api": "/api"
  }
}
```

## 📝 路径约定

- 所有 API 路径**不使用**尾部斜杠（如 `/api/sys/user` 而不是 `/api/sys/user/`）
- 使用小写字母和连字符分隔单词
- 资源名称使用复数形式（如 `users` 而不是 `user`，但当前为了保持一致性暂时使用单数）

## 🧪 接口测试

推荐使用 VSCode REST Client 插件进行接口测试：

1. 安装插件：`REST Client by Huachao Mao`
2. 打开项目中的 `docs/api.http` 文件
3. 点击请求上方的 "Send Request" 按钮即可测试

## 🚀 快速开始

1. 启动后端服务：

   ```bash
   cd backend
   cargo run
   ```

2. 测试健康检查：

   ```bash
   curl http://localhost:3001/health
   ```

3. 测试用户接口：
   ```bash
   curl http://localhost:3001/api/sys/user
   ```
