# ⚙️ Options API 接口规范

## 1. 概述

本规范定义了 `rustzen-admin` 项目中，用于前端下拉选择、联想搜索、表单联动等场景的通用 "Options" 接口。

此接口旨在为各个资源（如角色、用户、菜单）提供一个轻量级的、统一的数据获取方式，返回 `[{ label, value }]` 格式的键值对数组。

## 2. 设计原则

- **统一路径**: 每个需要提供 Options 的资源，都在其标准 API 路径下提供一个 `/options` 子路径。
- **标准结构**: 接口响应体中的 `data` 字段固定为 `OptionItem[]` 类型，其中 `OptionItem` 包含 `label` 和 `value`。
- **轻量高效**: 仅返回前端展示和研究所需的最少字段，避免传输不必要的数据。
- **独立权限**: Options 接口拥有自己独立的权限点（如 `system:roles:options`），与父级资源的 CRUD 权限解耦，以实现更精细化的访问控制。

---

## 3. 接口定义

### 3.1. HTTP 请求

- **方法**: `GET`
- **路径格式**: `/api/system/{resource}/options`

**路径示例**:

| 资源 | 接口路径                    | 说明                   |
| :--- | :-------------------------- | :--------------------- |
| 角色 | `/api/system/roles/options` | 获取所有角色的 Options |
| 用户 | `/api/system/users/options` | 获取所有用户的 Options |
| 菜单 | `/api/system/menus/options` | 获取所有菜单的 Options |

### 3.2. Query 参数

| 参数名   | 类型   | 必选 | 说明                                                                                                           | 示例               |
| :------- | :----- | :--- | :------------------------------------------------------------------------------------------------------------- | :----------------- |
| `q`      | string | 否   | 用于模糊搜索的关键字，通常匹配 `label` 字段。                                                                  | `?q=管理员`        |
| `limit`  | number | 否   | 限制返回的数据条数，用于性能优化，默认不限制或由后端决定。                                                     | `?limit=10`        |
| `status` | string | 否   | 根据状态筛选。**默认仅返回"启用"状态的资源**。可传递特定状态值（如 `disabled`）或 `all` 来获取所有状态的资源。 | `?status=disabled` |

### 3.3. 响应结构

遵循项目统一的 `ApiResponse` 格式。

#### OptionItem 类型

```typescript
interface OptionItem {
  label: string; // 显示的文本
  value: string | number; // 唯一标识符
  [key: string]: any; // 可选，允许附加其他元数据
}
```

#### 成功响应示例

```json
{
  "code": 200,
  "message": "获取成功",
  "data": [
    { "label": "超级管理员", "value": 1 },
    { "label": "内容编辑员", "value": 2 }
  ],
  "timestamp": "2024-07-30T10:00:00Z"
}
```

---

## 4. 权限控制

### 4.1. 原子化权限设计

为了实现精细化的访问控制，Options 接口不应简单地跟随其父资源的读取权限，而应拥有一个独立的、原子化的权限。

**设计原则**：一个操作（如获取下拉列表）对应一个独立的权限。

**权限标识符示例**：

| 资源 | Options 接口权限       | 完整列表权限        |
| :--- | :--------------------- | :------------------ |
| 角色 | `system:roles:options` | `system:roles:list` |
| 用户 | `system:users:options` | `system:users:list` |
| 菜单 | `system:menus:options` | `system:menus:list` |

这种设计分离了"查看管理页面"和"在其他页面引用该资源"这两种不同的场景。

### 4.2. 场景示例：用户管理员

假设有一个"用户管理员"角色，他的职责是管理用户，包括为用户分配角色。但是，他不应该有权限管理角色本身（增删改查角色）。

在这种情况下，我们为该角色分配如下权限：

- `system:users:list`, `system:users:edit` (管理用户)
- `system:roles:options` (在编辑用户时，能够获取角色下拉列表)

他 **不应** 被授予 `system:roles:list` 权限，因此他将无法访问"角色管理"页面。

### 4.3. 后端实现

在后端，保护 Options 接口的中间件应检查其特定的 `options` 权限。

```rust
// 伪代码: 检查用户是否有权限访问 'GET /api/system/roles/options'
let required_permission = "system:roles:options"; // 注意：不再是 :list
check_permission(user, required_permission).await?;
```

---

## 5. 错误响应

错误响应遵循项目统一格式。

### 权限不足 (403 Forbidden)

```json
{
  "code": 403,
  "message": "权限不足，无法访问该资源",
  "data": null,
  "timestamp": "2024-07-30T10:00:00Z"
}
```

### 资源未找到 (404 Not Found)

这通常发生在 `resource` 路径不正确时。

```json
{
  "code": 404,
  "message": "请求的资源未找到",
  "data": null,
  "timestamp": "2024-07-30T10:00:00Z"
}
```
