# API Service Guide

## 🏗️ Unified Modular Architecture

All API services are managed in a unified, modular way:

```
services/
├── api.ts                    # Core API utility
├── auth/
│   └── index.ts             # Authentication API service
├── system/
│   ├── index.ts             # Unified export
│   ├── user.ts              # User management API
│   ├── role.ts              # Role management API
│   ├── menu.ts              # Menu management API
│   ├── dict.ts              # Dictionary management API
│   └── log.ts               # Log management API
└── index.ts                  # Global unified export

types/
├── api.d.ts                  # Core API types
├── auth.d.ts                 # Auth module types
├── system.d.ts               # System management types
└── ...                       # Other domain types (future extension)
```

## 🎯 Design Principles

1. **Centralized Type Management**: All modules use global type declarations.
2. **Domain Grouping**: API services are organized by domain (auth, system, business, etc.).
3. **Unified Prefix Handling**: All API requests automatically use the `/api` prefix at the base layer.
4. **Clear Type Hierarchy**: Each domain has its own type declaration file.
5. **Backward Compatibility**: Existing import patterns remain valid.
6. **Extensibility**: Easy to add new business domains in the future.

## 📦 Type Usage

### Unified Type Import Pattern

```typescript
// Auth-related types
import type {
  LoginRequest,
  LoginResponse,
  UserInfoResponse
} from "Auth";

// System management types
import type { System } from "System";
const user: System.User.Item = { ... };
const role: System.Role.Item = { ... };

// Core API types
import type { ApiResponse, OptionItem } from "Api";
```

### API Service Import Patterns

```typescript
// Pattern 1: Selective import (recommended)
import { userAPI, roleAPI, authAPI } from "@/services";

// Pattern 2: Domain import
import { authAPI } from "@/services/auth";
import { userAPI } from "@/services/system";
```

## 💡 Usage Examples

### Authentication

```typescript
import { authAPI } from "@/services";
import type { LoginRequest, UserInfoResponse } from "Auth";

// Login
const loginData: LoginRequest = {
  username: "admin",
  password: "123456",
};
const response = await authAPI.login(loginData);

// Get user info
const userInfo: UserInfoResponse = await authAPI.getUserInfo();
```

### System Management

```typescript
import { userAPI } from "@/services";
import type { System } from "System";

// Get user list
const params: System.User.QueryParams = {
  current: 1,
  pageSize: 10,
  username: "admin",
};
const response = await userAPI.getUserList(params);

// Create user
const userData: System.User.CreateRequest = {
  username: "newuser",
  email: "user@example.com",
  password: "123456",
  roleIds: [1, 2],
};
const newUser = await userAPI.createUser(userData);
```

### ProTable Integration

```typescript
import { proTableRequest } from "@/services";
import type { System } from "System";

<ProTable<System.User.Item>
  request={(params) => proTableRequest("/system/users", params)}
  columns={columns}
/>;
```

### SWR Integration

```typescript
import useSWR from "swr";
import { swrFetcher, userAPI } from "@/services";
import type { System } from "System";

// Get user list
const { data, error } = useSWR<System.User.ListResponse>(
  userAPI.urls.getUserList(),
  swrFetcher
);

// Get single user
const { data: user } = useSWR<System.User.Item>(
  userAPI.urls.getUserById(userId),
  swrFetcher
);
```

## 🚀 Extension Guide

### Adding a New Business Domain

1. **Create type definitions**:

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

2. **Create the domain directory**:

```bash
mkdir src/services/business
```

3. **Create API service**:

```typescript
// src/services/business/order.ts
import { request } from "../api";
import type { Business } from "Business";

export const orderAPI = {
  getOrderList: (params?: Business.Order.QueryParams) =>
    request.get<Business.Order.ListResponse>("/business/orders", params),
  getOrderById: (id: number) =>
    request.get<Business.Order.Item>(`/business/orders/${id}`),
  createOrder: (data: Business.Order.CreateRequest) =>
    request.post<Business.Order.Item>("/business/orders", data),
  // URL generators (for SWR)
  urls: {
    getOrderById: (id: number) => `/business/orders/${id}`,
    getOrderList: () => "/business/orders",
  },
};
```

4. **Create domain export**:

```typescript
// src/services/business/index.ts
export { orderAPI } from "./order";
export { productAPI } from "./product";

export default {
  order: orderAPI,
  product: productAPI,
};
```

5. **Update global export**:

```typescript
// src/services/index.ts
export { orderAPI, productAPI } from "./business";
```

## 🌟 Architecture Advantages

1. **Consistency**: All modules use the same type management approach.
2. **Discoverability**: Types are globally visible, with better IDE support.
3. **Clear Hierarchy**: `Auth.LoginRequest` vs `System.User.Item` vs `Business.Order.Item`.
4. **Strong Extensibility**: New domains only require a new `.d.ts` file.
5. **Maintainability**: Each domain's types are centrally managed and appropriately sized.
6. **Backward Compatibility**: Existing code does not need to change.

## 🔧 Available API Modules

### Auth Domain

- `authAPI` - User login, registration, logout, get user info

### System Management Domain

- `userAPI` - User management
- `roleAPI` - Role management
- `menuAPI` - Menu management
- `dictAPI` - Dictionary management
- `logAPI` - Log management

Each module provides full CRUD operations and SWR support.

## 📝 Type Reference Table

| Module   | Type Declaration File | Namespace  | Example Type          |
| -------- | --------------------- | ---------- | --------------------- |
| Core API | `api.d.ts`            | `Api`      | `Api.ApiResponse<T>`  |
| Auth     | `auth.d.ts`           | `Auth`     | `Auth.LoginRequest`   |
| System   | `system.d.ts`         | `System`   | `System.User.Item`    |
| Business | `business.d.ts`       | `Business` | `Business.Order.Item` |

This unified modular architecture provides a clear structure and excellent scalability for the project.
