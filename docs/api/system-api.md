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

- **接口**: `GET /api/system/users`
- **描述**: 分页获取用户列表
- **权限**: 需要有用户管理菜单权限

#### 查询参数

```
page=1              // 页码，默认1
page_size=20        // 每页大小，默认20
q=admin             // 搜索关键词（用户名、邮箱）
status=1            // 状态筛选 1:正常 0:禁用
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "username": "admin",
        "email": "admin@example.com",
        "realName": "管理员",
        "avatarUrl": null,
        "status": 1,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z",
        "roles": [
          {
            "id": 1,
            "name": "超级管理员",
            "code": "super_admin"
          }
        ]
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 20
  }
}
```

### 获取用户详情

- **接口**: `GET /api/system/users/{id}`
- **描述**: 获取用户详细信息
- **权限**: 需要有用户管理菜单权限

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "id": 1,
    "username": "admin",
    "email": "admin@example.com",
    "realName": "管理员",
    "avatarUrl": null,
    "status": 1,
    "createdAt": "2024-01-01T00:00:00Z",
    "updatedAt": "2024-01-01T00:00:00Z",
    "roles": [
      {
        "id": 1,
        "name": "超级管理员",
        "code": "super_admin"
      }
    ]
  }
}
```

### 创建用户

- **接口**: `POST /api/system/users`
- **描述**: 创建新用户
- **权限**: 需要有用户管理菜单权限

#### 请求参数

```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "password": "password123",
  "realName": "新用户",
  "status": 1,
  "roleIds": [1, 2]
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
        "name": "编辑员",
        "code": "editor"
      }
    ]
  }
}
```

### 更新用户

- **接口**: `PUT /api/system/users/{id}`
- **描述**: 更新用户信息
- **权限**: 需要有用户管理菜单权限

#### 请求参数

```json
{
  "username": "updateduser",
  "email": "updated@example.com",
  "realName": "更新的用户",
  "status": 1,
  "roleIds": [1, 2]
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "用户更新成功",
  "data": {
    "id": 2,
    "username": "updateduser",
    "email": "updated@example.com",
    "realName": "更新的用户",
    "status": 1,
    "updatedAt": "2024-01-01T12:00:00Z",
    "roles": [
      {
        "id": 1,
        "name": "超级管理员",
        "code": "super_admin"
      },
      {
        "id": 2,
        "name": "编辑员",
        "code": "editor"
      }
    ]
  }
}
```

### 删除用户

- **接口**: `DELETE /api/system/users/{id}`
- **描述**: 软删除用户
- **权限**: 需要有用户管理菜单权限

### 获取用户选项

- **接口**: `GET /api/system/users/options`
- **描述**: 获取用户选项列表
- **权限**: 需要有用户管理菜单权限

#### 查询参数

```
q=admin             // 搜索关键词
limit=50            // 返回数量限制，默认为50
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "value": 1,
      "label": "管理员(admin)"
    }
  ]
}
```

---

## 🛡️ 角色管理

### 获取角色列表

- **接口**: `GET /api/system/roles`
- **描述**: 获取所有角色
- **权限**: 需要有角色管理菜单权限

#### 查询参数

```
page=1              // 页码，默认为1
page_size=20        // 每页大小，默认为20
q=admin             // 搜索关键词（角色名称、编码）
status=1            // 状态筛选 1:正常 0:禁用
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "name": "超级管理员",
        "code": "super_admin",
        "description": "系统超级管理员",
        "status": 1,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 20
  }
}
```

### 获取角色详情

- **接口**: `GET /api/system/roles/{id}`
- **描述**: 获取角色详细信息
- **权限**: 需要有角色管理菜单权限

### 创建角色

- **接口**: `POST /api/system/roles`
- **描述**: 创建新角色
- **权限**: 需要有角色管理菜单权限

#### 请求参数

```json
{
  "name": "测试角色",
  "code": "test_role",
  "description": "这是一个测试角色",
  "status": 1
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "角色创建成功",
  "data": {
    "id": 2,
    "name": "测试角色",
    "code": "test_role",
    "description": "这是一个测试角色",
    "status": 1,
    "createdAt": "2024-01-01T12:00:00Z"
  }
}
```

### 更新角色

- **接口**: `PUT /api/system/roles/{id}`
- **描述**: 更新角色信息
- **权限**: 需要有角色管理菜单权限

### 删除角色

- **接口**: `DELETE /api/system/roles/{id}`
- **描述**: 软删除角色
- **权限**: 需要有角色管理菜单权限

### 获取角色菜单权限

- **接口**: `GET /api/system/roles/{id}/menus`
- **描述**: 获取角色菜单权限
- **权限**: 需要有角色管理菜单权限

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": [1, 2, 3, 4, 5]
}
```

### 设置角色菜单权限

- **接口**: `PUT /api/system/roles/{id}/menus`
- **描述**: 设置角色菜单权限
- **权限**: 需要有角色管理菜单权限

#### 请求参数

```json
[1, 2, 3, 4, 5]
```

### 获取角色选项

- **接口**: `GET /api/system/roles/options`
- **描述**: 获取角色选项列表
- **权限**: 需要有角色管理菜单权限

---

## 📋 菜单管理

### 获取菜单列表

- **接口**: `GET /api/system/menus`
- **描述**: 获取菜单列表
- **权限**: 需要有菜单管理菜单权限

#### 查询参数

```
q=System          // 搜索关键词
status=1          // 菜单状态
menu_type=1       // 菜单类型（1=目录，2=菜单，3=按钮）
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "title": "系统管理",
        "name": "System",
        "path": "/system",
        "component": "Layout",
        "icon": "system",
        "parentId": null,
        "sortOrder": 1,
        "menuType": 1,
        "status": 1,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z",
        "children": [
          {
            "id": 2,
            "title": "用户管理",
            "name": "User",
            "path": "/system/user",
            "component": "system/user/index",
            "icon": "user",
            "parentId": 1,
            "sortOrder": 1,
            "menuType": 2,
            "status": 1,
            "children": []
          }
        ]
      }
    ]
  }
}
```

### 获取菜单详情

- **接口**: `GET /api/system/menus/{id}`
- **描述**: 获取菜单详细信息
- **权限**: 需要有菜单管理菜单权限

### 创建菜单

- **接口**: `POST /api/system/menus`
- **描述**: 创建新菜单
- **权限**: 需要有菜单管理菜单权限

#### 请求参数

```json
{
  "title": "新菜单",
  "name": "NewMenu",
  "path": "/new-menu",
  "component": "NewMenuComponent",
  "icon": "menu-icon",
  "parentId": null,
  "sortOrder": 1,
  "menuType": 2,
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
    "title": "新菜单",
    "name": "NewMenu",
    "path": "/new-menu",
    "component": "NewMenuComponent",
    "icon": "menu-icon",
    "parentId": null,
    "sortOrder": 1,
    "menuType": 2,
    "status": 1,
    "createdAt": "2024-01-01T12:00:00Z"
  }
}
```

### 更新菜单

- **接口**: `PUT /api/system/menus/{id}`
- **描述**: 更新菜单信息
- **权限**: 需要有菜单管理菜单权限

### 删除菜单

- **接口**: `DELETE /api/system/menus/{id}`
- **描述**: 软删除菜单
- **权限**: 需要有菜单管理菜单权限

### 获取菜单选项

- **接口**: `GET /api/system/menus/options`
- **描述**: 获取菜单选项列表
- **权限**: 需要有菜单管理菜单权限

---

## 📋 字典管理

### 获取字典列表

- **接口**: `GET /api/system/dict`
- **描述**: 获取字典列表
- **权限**: 需要有字典管理菜单权限

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "id": 1,
      "dictType": "user_status",
      "dictLabel": "启用",
      "dictValue": "1",
      "sortOrder": 1,
      "status": 1,
      "remark": "用户状态-启用"
    },
    {
      "id": 2,
      "dictType": "user_status",
      "dictLabel": "禁用",
      "dictValue": "0",
      "sortOrder": 2,
      "status": 1,
      "remark": "用户状态-禁用"
    }
  ]
}
```

### 获取字典选项

- **接口**: `GET /api/system/dict/options`
- **描述**: 获取字典选项列表
- **权限**: 需要有字典管理菜单权限

#### 查询参数

```
dict_type=user_status  // 字典类型
q=启用                 // 搜索关键词
limit=50               // 返回数量限制
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "value": "1",
      "label": "启用"
    },
    {
      "value": "0",
      "label": "禁用"
    }
  ]
}
```

---

## 📋 操作日志

### 获取日志列表

- **接口**: `GET /api/system/log`
- **描述**: 分页获取操作日志
- **权限**: 需要有日志管理菜单权限

#### 查询参数

```
page=1              // 页码
page_size=20        // 每页大小
q=登录成功          // 搜索关键词（日志消息）
level=INFO          // 日志级别
```

#### 响应示例

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "level": "INFO",
        "message": "用户登录成功",
        "userId": 1,
        "ipAddress": "192.168.1.100",
        "createdAt": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 20
  }
}
```

### 获取日志详情

- **接口**: `GET /api/system/log/{id}`
- **描述**: 获取日志详细信息
- **权限**: 需要有日志管理菜单权限

### 创建日志记录

- **接口**: `POST /api/system/log`
- **描述**: 创建新日志记录
- **权限**: 需要有日志管理菜单权限

#### 请求参数

```json
{
  "level": "INFO",
  "message": "这是一条测试日志",
  "userId": 1,
  "ipAddress": "192.168.1.100"
}
```

#### 响应示例

```json
{
  "code": 200,
  "message": "日志记录创建成功",
  "data": {
    "id": 1,
    "level": "INFO",
    "message": "这是一条测试日志",
    "userId": 1,
    "ipAddress": "192.168.1.100",
    "createdAt": "2024-01-01T12:00:00Z"
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
    "items": [
      /* 数据列表 */
    ],
    "total": 100,
    "page": 1,
    "pageSize": 20
  },
  "timestamp": 1672531200
}
```

---

## 附录：全局错误码规范

为了确保 API 的一致性和可扩展性，所有接口均遵循统一的 5 位数错误码规范 `A-BB-CC`。

- **A (1 位)**: **错误级别**

  - `1`: **业务逻辑错误 (Business Error)** - 由用户输入或操作直接导致的可预期失败 (例如: 用户名已存在, 权限不足)。
  - `2`: **系统级错误 (System Error)** - 由内部服务问题导致，用户无法直接解决 (例如: 数据库查询失败)。

- **BB (2 位)**: **模块标识**

  - `00`: 通用/系统模块
  - `01`: 认证模块 (Auth)
  - `02`: 用户管理模块 (User)
  - `03`: 角色管理模块 (Role)
  - `04`: 菜单管理模块 (Menu)

- **CC (2 位)**: **具体错误序号** (在模块内唯一)

---

### 错误码总表

| 错误码          | 模块 | 错误类型              | 说明                                                                     |
| :-------------- | :--- | :-------------------- | :----------------------------------------------------------------------- |
| **通用**        |
| `10001`         | 通用 | `NotFound`            | 请求的资源不存在。                                                       |
| `10002`         | 通用 | `InvalidOperation`    | 操作无效，通常因为违反了某项业务规则（例如：删除一个仍包含子项的目录）。 |
| `20001`         | 通用 | `DatabaseQueryFailed` | 服务器内部数据库操作失败。                                               |
| **认证 (Auth)** |
| `10101`         | 认证 | `InvalidCredentials`  | 用户名或密码错误。                                                       |
| `10102`         | 认证 | `InvalidToken`        | Token 无效或已过期。                                                     |
| `10103`         | 认证 | `PermissionDenied`    | 当前用户权限不足。                                                       |
| **用户 (User)** |
| `10201`         | 用户 | `UsernameConflict`    | 用户名已存在。                                                           |
| `10202`         | 用户 | `EmailConflict`       | 邮箱地址已存在。                                                         |
| **角色 (Role)** |
| `10301`         | 角色 | `RoleNameConflict`    | 角色名已存在。                                                           |
| **菜单 (Menu)** |
| `10401`         | 菜单 | `MenuTitleConflict`   | 菜单标题已存在。                                                         |

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
