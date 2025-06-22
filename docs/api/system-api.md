# 🚀 极简版 API 接口文档

## 📋 接口概述

本文档描述 rustzen-admin 的 system API 接口。

**设计原则**：

- 最简单的 CRUD 操作
- 基于角色的菜单权限控制
- 统一的响应格式（驼峰命名）
- RESTful 风格

---

## 🔐 认证接口

### 用户登录

- **接口**: `POST /api/auth/login`
- **描述**: 用户登录获取 JWT Token
- **权限**: 无需认证

#### 请求参数

```json
{
  "username": "admin",
  "password": "password123"
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "登录成功",
  "data": {
    "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expiresIn": 7200,
    "userInfo": {
      "id": 1,
      "username": "admin",
      "realName": "管理员",
      "roles": ["管理员"]
    }
  }
}
```

### 用户登出

- **接口**: `POST /api/auth/logout`
- **描述**: 用户登出
- **权限**: 需要认证

### 获取用户信息

- **接口**: `GET /api/auth/userinfo`
- **描述**: 获取当前用户信息和菜单
- **权限**: 需要认证

#### 响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": {
    "user": {
      "id": 1,
      "username": "admin",
      "realName": "管理员",
      "email": "admin@example.com",
      "avatarUrl": "https://example.com/avatar.jpg",
      "status": 1,
      "lastLoginAt": "2024-01-01T12:00:00Z"
    },
    "menus": [
      {
        "id": 1,
        "title": "系统管理",
        "path": "/system",
        "icon": "setting",
        "sortOrder": 1,
        "children": [
          {
            "id": 2,
            "title": "用户管理",
            "path": "/system/users",
            "icon": "user",
            "sortOrder": 1
          }
        ]
      }
    ]
  }
}
```

---

## 👥 用户管理

### 获取用户列表

- **接口**: `GET /api/sys/users`
- **描述**: 分页获取用户列表
- **权限**: 需要有用户管理菜单权限

#### 查询参数

```
page=1              // 页码，默认1
pageSize=10         // 每页大小，默认10
username=admin      // 用户名筛选
status=1           // 状态筛选 1:正常 2:禁用
```

#### 响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": {
    "list": [
      {
        "id": 1,
        "username": "admin",
        "email": "admin@example.com",
        "realName": "管理员",
        "avatarUrl": "https://example.com/avatar.jpg",
        "status": 1,
        "lastLoginAt": "2024-01-01T12:00:00Z",
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T12:00:00Z",
        "roles": [
          {
            "id": 1,
            "roleName": "管理员"
          }
        ]
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 10
  }
}
```

### 创建用户

- **接口**: `POST /api/sys/users`
- **描述**: 创建新用户
- **权限**: 需要有用户管理菜单权限

#### 请求参数

```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "realName": "新用户",
  "password": "password123",
  "status": 1,
  "roleIds": [2]
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "用户创建成功",
  "data": {
    "id": 2,
    "username": "newuser",
    "email": "newuser@example.com",
    "realName": "新用户",
    "status": 1,
    "createdAt": "2024-01-01T12:00:00Z",
    "roles": [
      {
        "id": 2,
        "roleName": "编辑员"
      }
    ]
  }
}
```

### 更新用户

- **接口**: `PUT /api/sys/users/{id}`
- **描述**: 更新用户信息
- **权限**: 需要有用户管理菜单权限

#### 请求参数

```json
{
  "email": "updated@example.com",
  "realName": "更新用户",
  "status": 1,
  "roleIds": [1, 2]
}
```

### 删除用户

- **接口**: `DELETE /api/sys/users/{id}`
- **描述**: 软删除用户
- **权限**: 需要有用户管理菜单权限

---

## 🛡️ 角色管理

### 获取角色列表

- **接口**: `GET /api/sys/roles`
- **描述**: 获取所有角色
- **权限**: 需要有角色管理菜单权限

#### 响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": {
    "list": [
      {
        "id": 1,
        "roleName": "管理员",
        "description": "系统管理员",
        "status": 1,
        "userCount": 2,
        "menuCount": 10,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T12:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 10
  }
}
```

### 创建角色

- **接口**: `POST /api/sys/roles`
- **描述**: 创建新角色
- **权限**: 需要有角色管理菜单权限

#### 请求参数

```json
{
  "roleName": "编辑员",
  "description": "内容编辑员",
  "status": 1,
  "menuIds": [1, 2, 3]
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "角色创建成功",
  "data": {
    "id": 2,
    "roleName": "编辑员",
    "description": "内容编辑员",
    "status": 1,
    "createdAt": "2024-01-01T12:00:00Z"
  }
}
```

### 分配角色菜单

- **接口**: `POST /api/sys/roles/{id}/menus`
- **描述**: 为角色分配菜单权限
- **权限**: 需要有角色管理菜单权限

#### 请求参数

```json
{
  "menuIds": [1, 2, 3, 4]
}
```

---

## 📋 菜单管理

### 获取菜单树

- **接口**: `GET /api/sys/menus`
- **描述**: 获取菜单树形结构
- **权限**: 需要有菜单管理菜单权限

#### 响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": [
    {
      "id": 1,
      "parentId": 0,
      "title": "系统管理",
      "path": "/system",
      "component": "Layout",
      "icon": "setting",
      "sortOrder": 1,
      "status": 1,
      "createdAt": "2024-01-01T00:00:00Z",
      "updatedAt": "2024-01-01T12:00:00Z",
      "children": [
        {
          "id": 2,
          "parentId": 1,
          "title": "用户管理",
          "path": "/system/users",
          "component": "UserList",
          "icon": "user",
          "sortOrder": 1,
          "status": 1,
          "createdAt": "2024-01-01T00:00:00Z",
          "updatedAt": "2024-01-01T12:00:00Z",
          "children": []
        }
      ]
    }
  ]
}
```

### 创建菜单

- **接口**: `POST /api/sys/menus`
- **描述**: 创建新菜单
- **权限**: 需要有菜单管理菜单权限

#### 请求参数

```json
{
  "parentId": 1,
  "title": "角色管理",
  "path": "/system/roles",
  "component": "RoleList",
  "icon": "team",
  "sortOrder": 2,
  "status": 1
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "菜单创建成功",
  "data": {
    "id": 3,
    "parentId": 1,
    "title": "角色管理",
    "path": "/system/roles",
    "component": "RoleList",
    "icon": "team",
    "sortOrder": 2,
    "status": 1,
    "createdAt": "2024-01-01T12:00:00Z"
  }
}
```

---

## 📋 操作日志

### 获取日志列表

- **接口**: `GET /api/sys/logs`
- **描述**: 分页获取操作日志
- **权限**: 需要有日志管理菜单权限

#### 查询参数

```
page=1              // 页码
pageSize=10         // 每页大小
username=admin      // 用户名筛选
action=LOGIN        // 操作类型筛选
startTime=2024-01-01  // 开始时间
endTime=2024-12-31    // 结束时间
```

#### 响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": {
    "list": [
      {
        "id": 1,
        "userId": 1,
        "username": "admin",
        "action": "USER_LOGIN",
        "description": "用户登录",
        "ipAddress": "192.168.1.100",
        "createdAt": "2024-01-01T12:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 10
  }
}
```

---

## 🔒 权限控制

### 权限验证逻辑

1. **用户登录** → 获取用户角色
2. **角色权限** → 通过 `role_menus` 表获取可访问菜单
3. **接口权限** → 根据请求的接口路径判断是否有对应菜单权限

### 权限中间件

```rust
// 伪代码示例
async fn check_menu_permission(user_id: i64, menu_path: &str) -> bool {
    // 1. 获取用户角色
    let roles = get_user_roles(user_id).await;

    // 2. 获取角色菜单权限
    let menus = get_role_menus(&roles).await;

    // 3. 检查是否有访问权限
    menus.iter().any(|menu| menu.path == menu_path)
}
```

---

## 📝 统一响应格式

### 成功响应

```json
{
  "code": 200,
  "message": "操作成功",
  "data": {
    /* 响应数据 */
  },
  "timestamp": 1672531200
}
```

### 错误响应

```json
{
  "code": 400,
  "message": "参数错误",
  "data": null,
  "timestamp": 1672531200
}
```

### 分页响应

```json
{
  "code": 200,
  "message": "获取成功",
  "data": {
    "list": [
      /* 数据列表 */
    ],
    "total": 100,
    "page": 1,
    "pageSize": 10
  },
  "timestamp": 1672531200
}
```

---

## ❌ 常见错误码

| 错误码 | 说明             |
| ------ | ---------------- |
| 1001   | 用户名或密码错误 |
| 1002   | Token 无效或过期 |
| 1003   | 权限不足         |
| 2001   | 用户名已存在     |
| 2002   | 邮箱已存在       |
| 3001   | 角色名已存在     |
| 4001   | 菜单标题已存在   |

---

## 🚀 开发建议

### 1. 接口设计原则

- **简单明了**：接口功能单一，参数简洁
- **统一规范**：遵循 RESTful 设计规范
- **驼峰命名**：所有字段名使用驼峰命名格式
- **权限控制**：基于菜单权限进行访问控制

### 2. 前端集成

```javascript
// 登录后获取用户菜单
const userInfo = await api.get("/api/auth/userinfo");
const menus = userInfo.data.menus;

// 根据菜单生成路由
const routes = generateRoutes(menus);

// 权限验证
const hasPermission = (menuPath) => {
  return menus.some((menu) => menu.path === menuPath);
};
```

### 3. 字段命名规范

- **数据库字段**：snake_case（如：`user_id`, `created_at`）
- **API 返回字段**：camelCase（如：`userId`, `createdAt`）
- **前端使用**：camelCase 格式，便于 JavaScript 处理

---

**设计理念**：从最简单的菜单权限开始，使用驼峰命名提升前端开发体验，根据实际需求逐步扩展功能。
