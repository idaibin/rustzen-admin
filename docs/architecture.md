# Rust 业务架构设计规范

> 技术栈：Rust · Axum · sqlx · PostgreSQL  
> 原则：够用、可读、边界清晰，不过度设计

---

## 一、结构与布局

### 1.1 整体目录结构

本项目采用 `core/` 与 `common/` 分层；按需保留 feature；无 DB 不建 `core/db.rs`，无鉴权不建 `middleware/`。

```
src/
├── main.rs              # 入口：启动、路由组装
├── core/                # 基础设施与核心能力
│   ├── mod.rs
│   ├── app.rs           # 服务启动、路由组装、中间件
│   ├── config.rs        # 配置加载（env / 文件）
│   ├── db.rs            # 连接池初始化（有 DB 时建）
│   ├── extractor.rs     # Axum 提取器（如 CurrentUser）
│   ├── jwt.rs           # JWT 生成与校验（按需）
│   ├── password.rs      # 密码哈希（按需）
│   ├── permission.rs   # 权限校验与缓存（按需）
│   └── system_info.rs   # 系统信息（如 CPU/内存，按需）
├── middleware/         # Axum 中间件
│   ├── mod.rs
│   ├── auth.rs          # 鉴权
│   └── log.rs           # 请求日志（按需）
├── common/             # 跨 feature 共用类型与工具
│   ├── mod.rs
│   ├── api.rs           # 统一响应封装（ApiResponse、AppResult）
│   ├── error.rs         # 统一错误类型（ServiceError、AppError）
│   ├── pagination.rs    # 分页参数
│   ├── router_ext.rs    # 路由扩展（如按权限注册）
│   ├── files.rs         # 文件上传等（按需）
│   └── utils/           # 仅允许单职责、边界清晰的子模块（谨慎添加）
│       └── ...
├── features/           # 业务功能，按领域拆分
│   ├── mod.rs
│   ├── auth/            # 登录、鉴权
│   │   ├── mod.rs
│   │   ├── api.rs
│   │   ├── dto.rs       # 请求/查询/响应类型
│   │   ├── model.rs
│   │   ├── repo.rs
│   │   └── service.rs
│   ├── dashboard/       # 仪表盘
│   │   ├── mod.rs
│   │   ├── api.rs
│   │   ├── dto.rs
│   │   ├── repo.rs
│   │   └── service.rs
│   └── system/          # 系统管理（用户、角色、菜单、字典、日志等）
│       ├── mod.rs
│       ├── user/
│       │   ├── mod.rs
│       │   ├── api.rs
│       │   ├── dto.rs   # 请求/查询/响应类型
│       │   ├── model.rs
│       │   ├── repo.rs
│       │   └── service.rs
│       ├── role/
│       ├── menu/
│       ├── dict/
│       └── log/
└── ...
```

- **model.rs**：与 DB 表映射的结构体。
- **dto.rs**：HTTP 请求、查询参数与响应类型（*Resp / *Vo 等）均放在此文件。

### 1.2 依赖方向

```
依赖层级（从外到内）：

main.rs
├── core/     ← 最内层，不依赖任何 feature 类型
├── common/   ← 被所有层依赖，自身不依赖 features/
├── middleware/ ← 依赖 common/error，不依赖具体 feature
└── features/
    ├── api.rs     ← 依赖 dto、service
    ├── service.rs ← 依赖 repo、model、common/error（及 core 中密码/JWT 等）
    ├── repo.rs    ← 依赖 model、common/error（不依赖 dto）
    ├── model.rs  ← 不依赖任何项目内部业务类型
    └── dto.rs     ← 请求/查询/响应结构，可 From<Model>

禁止的依赖：
✗ model 依赖 dto
✗ repo 跨 feature 调用（如 user/repo 调用 auth/repo）
✗ service 跨 feature 直调 repo（只能调本 feature 的 repo 或其它 feature 的 service）
✗ core/ 依赖任何 features/
```

## 二、各层与约定

### 2.1 文件建立规则

| 文件         | 建立条件                          | 不建的场景                         |
| ------------ | --------------------------------- | ---------------------------------- |
| `model.rs`   | 该 feature 有数据库表映射         | 无 DB 的 feature（如纯流程）       |
| `repo.rs`    | 该 feature 有数据库读写操作       | 纯流程型、无持久化的 feature       |
| `dto.rs`     | 该 feature 有 HTTP 请求/查询/响应 | 纯内部模块、中间件                 |
| `service.rs` | 该 feature 有业务逻辑             | 几乎所有 feature 都应该建           |
| `api.rs`     | 该 feature 注册 HTTP 路由         | 纯后台任务、worker 类型            |

### 2.2 各层职责

| 层                | 职责                                                                                                                                 |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `core/`           | 对接外部能力（DB、配置、JWT、密码、权限、系统信息等）；只做连接与封装，不含业务逻辑，不依赖 features。                               |
| `common/`         | 跨 feature 共用**类型**（error、api 响应、分页、路由扩展、文件工具）。可含 `utils/`，但仅限单职责、边界清晰的子模块，避免泛化堆砌。 |
| `api.rs`          | Handler + 路由注册；只做参数提取与响应组装，不写业务逻辑、不直连 DB。                                                               |
| `dto.rs`          | HTTP 请求/查询/响应结构、serde 规则；可 `From<Model>`；不放 DB 实体与业务逻辑。                                                       |
| `model.rs`        | 与表映射的结构体，字段 snake_case，无业务方法、不依赖 HTTP 类型。                                                                   |
| `repo.rs`         | 所有 SQL，返回 model；入参基础类型或内部 Command/Query 结构，不依赖 dto。可依赖 `common/error`；`get_by_id` 不存在可返回 NotFound。   |
| `service.rs`      | 业务校验、跨 repo 事务协调、业务错误语义化。入参 dto 或基础类型，出参 model 或 dto 中的响应类型（由 api 转响应）。可调其它 feature 的 service，不可直调其 repo。 |
| `common/error.rs`  | 项目级错误枚举（如 ServiceError、AppError），实现 `IntoResponse`；service 返回此类型，api 用 `?` 转 HTTP 响应。                     |
| 应用状态          | 只放全局共享资源（如 PgPool）；feature 状态以子结构挂载。多 feature 时可用聚合的 Services，或直接平铺，全项目统一一种方式。        |

### 2.3 命名规范

#### 2.3.1 结构体命名

| 类型            | 命名示例                                       | 所在文件   |
| --------------- | ---------------------------------------------- | ---------- |
| DB 实体         | `User`, `UserWithRolesEntity`, `LoginCredentialsEntity` | `model.rs` |
| 创建请求体      | `CreateUserDto`, `CreateDeployPayload`         | `dto.rs`   |
| 更新请求体      | `UpdateUserDto`                               | `dto.rs`   |
| 查询参数        | `UserQueryDto`, `ListUserQuery`                | `dto.rs`   |
| 响应体（单条）  | `UserItemVo`, `UserResp`                      | `dto.rs`   |
| 响应体（列表）  | `UserListResp`                                | `dto.rs`   |
| 跨 feature 传递 | `CurrentUser`, `AuthContext`                   | `core/` 或 `common/` |

请求体可用 `*Dto` 或 `*Payload`，响应可用 `*Vo` 或 `*Resp`，项目内保持一致即可。

#### 2.3.2 函数命名（repo 层）

| 函数名模式          | 语义                       | 返回类型                                              |
| ------------------- | -------------------------- | ----------------------------------------------------- |
| `find_by_id`        | 按主键查单条               | `Result<Option<Model>>`                               |
| `find_by_xxx`       | 按条件查单条               | `Result<Option<Model>>`                               |
| `get_by_id`         | 按主键查单条，不存在即错误 | `Result<Model>`（repo 直接返回 `AppError::NotFound`） |
| `find_all` / `list` | 查列表                     | `Result<Vec<Model>>`                                  |
| `insert`            | 插入一条                   | `Result<Model>` 或 `Result<i64>` 等                   |
| `update_by_id`      | 按主键更新                 | `Result<Model>` 或 `Result<()>`                       |
| `delete_by_id`      | 按主键删除                 | `Result<()>`                                           |
| `exists_by_xxx`     | 存在性检查                 | `Result<bool>`                                         |

- `find_by_id`：调用方需自行判断 `Option` 是否存在。
- `get_by_id`：适合「不存在即错误」的场景，repo 直接返回 `AppError::NotFound`，service 可直接透传。

#### 2.3.3 函数命名（service 层）

| 函数名              | 语义                                           |
| ------------------- | ---------------------------------------------- |
| `create`            | 创建，包含业务校验                             |
| `get` / `get_by_id` | 查询单条，NotFound 时返回 `AppError::NotFound`  |
| `list`              | 查询列表，可带分页                             |
| `update`            | 更新，包含权限和业务校验                       |
| `delete` / `remove` | 删除                                           |

---

## 三、最小骨架示例（PostgreSQL + 静态方法透传 pool）

本项目当前采用「Repo/Service 静态方法 + pool 透传」的方式；也可采用「结构体持有 pool / Arc<Repo>」的方式，按项目统一即可。

```rust
// src/features/user/repo.rs（示例：PostgreSQL）
use sqlx::PgPool;

use crate::common::error::ServiceError;
use super::model::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<User>, ServiceError> {
        let user = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(user)
    }

    pub async fn get_by_id(pool: &PgPool, id: i64) -> Result<User, ServiceError> {
        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("user".to_string()))
    }

    pub async fn insert(pool: &PgPool, name: &str) -> Result<User, ServiceError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users(name) VALUES ($1) RETURNING id, name",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(user)
    }
}
```

```rust
// src/features/user/service.rs
use crate::common::error::ServiceError;
use super::{dto::CreateUserDto, model::User, repo::UserRepository};

use sqlx::PgPool;

pub struct UserService;

impl UserService {
    pub async fn get(pool: &PgPool, id: i64) -> Result<User, ServiceError> {
        UserRepository::get_by_id(pool, id).await
    }

    pub async fn create(pool: &PgPool, dto: CreateUserDto) -> Result<User, ServiceError> {
        UserRepository::insert(pool, &dto.name).await
    }
}
```

```rust
// src/features/user/api.rs
use axum::{routing::{get, post}, Router};
use sqlx::PgPool;

use crate::common::api::{ApiResponse, AppResult};

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/users", post(create_handler))
        .route("/users/:id", get(get_handler))
}
```

---

## 四、禁止事项

| 禁止                                      | 原因                                                                                                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| 在 `model.rs` 放 HTTP 请求/响应结构体      | 边界混淆，model 只表示 DB 实体                                                                                                             |
| 在 `api.rs` 写 SQL 或业务校验逻辑         | 职责不单一，测试困难                                                                                                                       |
| 在 `repo.rs` 调用其他 feature 的 repo     | 产生跨领域隐式耦合                                                                                                                         |
| 用 `utils/`、`common/`、`misc/` 无边界堆砌 | 语义模糊，易沦为代码垃圾桶；若使用 `common/utils`，仅允许单职责、命名清晰的子模块                                                         |
| 为对称性强行建空文件                      | 增加认知噪音，无实际价值                                                                                                                   |
| 在持锁期间 `.await`（`std::sync::Mutex`） | 跨 await 点持锁，引发死锁或编译错误                                                                                                        |
| 在根应用状态直接放 feature 业务字段       | 应通过子状态挂载，如 `video: Arc<VideoState>`，否则应用状态膨胀、边界模糊。                                                               |

---

_本规范随项目演进持续更新 · 以实际可维护性为最终判断标准_
