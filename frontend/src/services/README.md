# API 服务使用指南

## 🏗️ 统一的模块化架构

经过重组后，所有API服务都采用统一的模块化管理方式：

```
services/
├── api.ts                    # 基础API工具
├── auth/
│   └── index.ts             # 认证API服务
├── system/
│   ├── index.ts             # 统一导出
│   ├── user.ts              # 用户管理API
│   ├── role.ts              # 角色管理API
│   ├── menu.ts              # 菜单管理API
│   ├── dict.ts              # 字典管理API
│   └── log.ts               # 日志管理API
└── index.ts                  # 全局统一导出

types/
├── api.d.ts                  # 基础API类型
├── auth.d.ts                 # 认证模块类型
├── system.d.ts               # 系统管理类型
└── ...                       # 其他域类型（未来扩展）
```

## 🎯 设计原则

1. **统一类型管理**：所有模块都使用全局类型声明方式
2. **业务域分组**：按功能域组织API服务（auth、system、business等）
3. **统一前缀处理**：所有 API 请求都在基础层自动添加 `/api` 前缀
4. **类型层次清晰**：每个域都有独立的类型声明文件
5. **向后兼容**：保持原有的导入方式不变
6. **可扩展性**：为未来新增业务域预留空间

## 📦 类型使用方式

### 统一的类型导入模式

```typescript
// 认证相关类型
import type {
  LoginRequest,
  LoginResponse,
  UserInfoResponse
} from "Auth";

// 系统管理类型
import type { System } from "System";
const user: System.User.Item = { ... };
const role: System.Role.Item = { ... };

// 基础API类型
import type { ApiResponse, OptionItem } from "Api";
```

### API服务导入方式

```typescript
// 方式1：按需导入（推荐）
import { userAPI, roleAPI, authAPI } from "@/services";

// 方式2：按域导入
import { authAPI } from "@/services/auth";
import { userAPI } from "@/services/system";

// 方式3：默认导入
import api from "@/services";
// 使用：api.auth.login(), api.system.user.getUserList()

// 方式4：域级导入
import systemAPI from "@/services/system";
// 使用：systemAPI.user.getUserList()
```

## 💡 使用示例

### 认证相关

```typescript
import { authAPI } from "@/services";
import type { LoginRequest, UserInfoResponse } from "Auth";

// 登录
const loginData: LoginRequest = {
  username: "admin",
  password: "123456"
};
const response = await authAPI.login(loginData);

// 获取用户信息
const userInfo: UserInfoResponse = await authAPI.getUserInfo();
```

### 系统管理

```typescript
import { userAPI } from "@/services";
import type { System } from "System";

// 获取用户列表
const params: System.User.QueryParams = {
  current: 1,
  pageSize: 10,
  username: "admin"
};
const response = await userAPI.getUserList(params);

// 创建用户
const userData: System.User.CreateRequest = {
  username: "newuser",
  email: "user@example.com",
  password: "123456",
  roleIds: [1, 2]
};
const newUser = await userAPI.createUser(userData);
```

### ProTable集成

```typescript
import { proTableRequest } from "@/services";
import type { System } from "System";

<ProTable<System.User.Item>
  request={(params) => proTableRequest("/system/users", params)}
  columns={columns}
/>
```

### SWR集成

```typescript
import useSWR from "swr";
import { swrFetcher, userAPI } from "@/services";
import type { System } from "System";

// 获取用户列表
const { data, error } = useSWR<System.User.ListResponse>(
  userAPI.urls.getUserList(),
  swrFetcher
);

// 获取单个用户
const { data: user } = useSWR<System.User.Item>(
  userAPI.urls.getUserById(userId),
  swrFetcher
);
```

## 🚀 扩展指南

### 添加新的业务域

当需要添加新的业务域（如订单管理）时：

1. **创建类型定义**：
```typescript
// src/types/business.d.ts
declare module "Business" {
  export namespace Order {
    export interface Item {
      id: number;
      orderNo: string;
      amount: number;
      status: OrderStatus;
      // ...
    }
    
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      orderNo?: string;
      status?: string;
    }
    
    export interface CreateRequest {
      // ...
    }
    
    export interface ListResponse {
      list: Item[];
      total: number;
      page: number;
      pageSize: number;
    }
  }
  
  export namespace Product {
    // ...
  }
}
```

2. **创建域目录**：
```bash
mkdir src/services/business
```

3. **创建API服务**：
```typescript
// src/services/business/order.ts
import { request } from '../api';
import type { Business } from 'Business';

export const orderAPI = {
  getOrderList: (params?: Business.Order.QueryParams) =>
    request.get<Business.Order.ListResponse>('/business/orders', params),
  
  getOrderById: (id: number) =>
    request.get<Business.Order.Item>(`/business/orders/${id}`),
  
  createOrder: (data: Business.Order.CreateRequest) =>
    request.post<Business.Order.Item>('/business/orders', data),
  
  // URL生成器（SWR使用）
  urls: {
    getOrderById: (id: number) => `/business/orders/${id}`,
    getOrderList: () => '/business/orders',
  },
};
```

4. **创建域导出**：
```typescript
// src/services/business/index.ts
export { orderAPI } from './order';
export { productAPI } from './product';

export default {
  order: orderAPI,
  product: productAPI,
};
```

5. **更新全局导出**：
```typescript
// src/services/index.ts
export { orderAPI, productAPI } from './business';
```

## 🌟 架构优势

1. **一致性**：所有模块都使用相同的类型管理方式
2. **可发现性**：类型在全局可见，IDE智能提示更好
3. **层次清晰**：`Auth.LoginRequest` vs `System.User.Item` vs `Business.Order.Item`
4. **扩展性强**：新增域只需添加新的 `.d.ts` 文件
5. **维护性好**：每个域的类型集中管理，大小适中
6. **向后兼容**：现有代码无需修改

## 🔧 可用的API模块

### 认证域
- `authAPI` - 用户登录、注册、登出、获取用户信息

### 系统管理域
- `userAPI` - 用户管理
- `roleAPI` - 角色管理
- `menuAPI` - 菜单管理
- `dictAPI` - 字典管理
- `logAPI` - 日志管理

每个模块都包含完整的 CRUD 操作和 SWR 支持。

## 📝 类型对照表

| 模块 | 类型声明文件 | 命名空间 | 示例类型 |
|------|-------------|----------|----------|
| 基础API | `api.d.ts` | `Api` | `Api.ApiResponse<T>` |
| 认证 | `auth.d.ts` | `Auth` | `Auth.LoginRequest` |
| 系统管理 | `system.d.ts` | `System` | `System.User.Item` |
| 业务模块 | `business.d.ts` | `Business` | `Business.Order.Item` |

这种统一的模块化架构为项目提供了清晰的结构和良好的可扩展性！
