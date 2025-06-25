# 退出登录功能实现说明

## 概述

实现了完整的用户退出登录功能，包括清除服务端权限缓存和客户端 token 存储。

## 后端实现

### 1. 权限缓存管理优化

#### 缓存过期机制

- 权限缓存有效期：24 小时
- 自动过期检查：每次获取缓存时自动检查过期状态
- 过期自动清理：过期的缓存会被自动移除

#### 关键方法更新

**`UserPermissionCache`** 新增方法：

```rust
// 检查缓存是否已过期
pub fn is_expired(&self) -> bool

// 获取缓存剩余有效时间（秒）
pub fn remaining_seconds(&self) -> i64
```

**`PermissionService`** 优化：

```rust
// 获取缓存时自动检查过期状态
pub fn get_cached_permissions(user_id: i64) -> Option<UserPermissionCache>

// 刷新用户权限缓存
pub async fn refresh_user_permissions(pool: &PgPool, user_id: i64) -> Result<(), ServiceError>

// 清除用户权限缓存（退出登录时调用）
pub fn clear_user_cache(user_id: i64)
```

### 2. 退出登录处理逻辑

#### 路由定义

```rust
// 受保护的认证路由
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(get_user_info_handler))
        .route("/logout", get(logout_handler))  // GET请求方式
}
```

#### 处理函数

```rust
async fn logout_handler(Extension(claims): Extension<Claims>) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("User {} ({}) is logging out", claims.username, claims.user_id);

    // 清除用户权限缓存
    PermissionService::clear_user_cache(claims.user_id);

    tracing::info!("Successfully logged out user {} ({})", claims.username, claims.user_id);

    Ok(ApiResponse::success(()))
}
```

### 3. 用户信息获取优化

优化了 `get_user_info` 方法的缓存处理逻辑：

- **有效缓存**：直接返回用户信息
- **缓存过期或不存在**：自动从数据库重新加载并缓存权限信息，而不是要求重新登录

```rust
// 缓存不存在或已过期，重新加载权限并缓存
tracing::info!("Permission cache not found or expired for user {}, refreshing from database", claims.user_id);

// 并行查询角色信息、权限和菜单
let (roles_result, permissions_result, menus_result) = tokio::join!(
    UserRepository::get_user_role_infos(pool, user.id),
    UserRepository::get_user_permissions(pool, user.id),
    Self::get_user_menus(pool, user.id)
);

// 重新缓存用户权限信息
PermissionService::cache_user_permissions(user.id, permissions);
```

## 前端实现

### 1. 认证相关类型定义

在 `frontend/src/types/api.d.ts` 中添加：

```typescript
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
}

export interface UserInfo {
  id: number;
  username: string;
  real_name?: string;
  avatar_url?: string;
  roles: Array<RoleInfo>;
  menus: Array<MenuInfo>;
}
```

### 2. 认证 API 服务

创建 `frontend/src/services/auth.ts`：

```typescript
export class AuthAPI {
  // 用户退出登录
  static async logout() {
    try {
      // 调用后端logout接口清除服务端缓存
      await request.post<void>(`${this.BASE_URL}/logout`);
    } catch (error) {
      console.warn("调用后端logout接口失败:", error);
    } finally {
      // 无论后端调用是否成功，都要清除本地token
      localStorage.removeItem("token");
      // 清除其他本地缓存数据
      localStorage.removeItem("userInfo");
    }
  }
}
```

### 3. 自动认证头支持

更新 `frontend/src/services/api.ts`：

```typescript
// 获取认证头
function getAuthHeaders(): Record<string, string> {
  const token = localStorage.getItem("token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

// 核心请求函数中自动添加认证头
const config: RequestInit = {
  ...options,
  headers: {
    ...defaultHeaders,
    ...getAuthHeaders(), // 自动添加认证头
    ...options.headers,
  },
};
```

## 缓存策略说明

### 1. 权限检查逻辑

```rust
pub async fn check_permission(
    pool: &PgPool,
    user_id: i64,
    required_permission: &str,
) -> Result<bool, ServiceError> {
    // 首先尝试从缓存获取（会自动检查过期并清理）
    if let Some(cache) = Self::get_cached_permissions(user_id) {
        return Ok(cache.has_permission(required_permission));
    }

    // 没有缓存，要求用户重新登录
    tracing::warn!("No valid permission cache found for user {}, require re-login", user_id);
    Err(ServiceError::InvalidCredentials)
}
```

### 2. 用户信息获取逻辑

- **缓存命中且未过期**：直接返回缓存的权限信息
- **缓存过期或不存在**：重新从数据库加载并更新缓存
- **权限检查场景**：缓存未命中时要求重新登录

### 3. 退出登录逻辑

- **服务端**：清除用户权限缓存
- **客户端**：清除本地存储的 token 和用户信息
- **容错处理**：即使服务端调用失败，也要清除客户端数据

## API 测试

### 退出登录测试流程

1. **登录获取 token**

   ```http
   POST /api/auth/login
   ```

2. **获取用户信息**（验证 token 有效）

   ```http
   GET /api/auth/me
   Authorization: Bearer {token}
   ```

3. **退出登录**（清除缓存）

   ```http
   GET /api/auth/logout
   Authorization: Bearer {token}
   ```

4. **再次获取用户信息**（应该需要重新登录）
   ```http
   GET /api/auth/me
   Authorization: Bearer {token}
   ```

## 安全考虑

1. **缓存过期**：24 小时自动过期，防止权限信息过度缓存
2. **主动清理**：退出登录时主动清除服务端缓存
3. **客户端清理**：确保本地 token 和缓存数据被完全清除
4. **容错处理**：网络失败时仍要清除客户端数据

## 性能优化

1. **缓存命中**：避免频繁数据库查询
2. **并行查询**：角色、权限、菜单信息并行获取
3. **自动刷新**：缓存过期时自动刷新，用户无感知
4. **内存管理**：过期缓存自动清理，避免内存泄漏
