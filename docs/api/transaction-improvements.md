# 用户创建事务改进文档

## 问题描述

在之前的实现中，用户创建和角色绑定是分离的操作：

1. 先创建用户
2. 再设置用户角色

这导致了一个数据一致性问题：如果角色设置失败（例如角色 ID 不存在），用户已经创建成功，造成数据不一致。

## 改进方案

### 1. 新增事务方法

在 `UserRepository` 中添加了 `create_with_roles` 方法：

```rust
pub async fn create_with_roles(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    real_name: Option<&str>,
    status: i16,
    role_ids: &[i64],
) -> Result<UserEntity, sqlx::Error>
```

### 2. 事务处理流程

```rust
let mut tx = pool.begin().await?;

// 1. 创建用户
let user = sqlx::query_as::<_, UserEntity>(...).fetch_one(&mut *tx).await?;

// 2. 验证角色ID有效性
let valid_roles = sqlx::query_as::<_, (i64,)>(
    "SELECT id FROM roles WHERE id = ANY($1) AND deleted_at IS NULL AND status = 1"
).bind(role_ids).fetch_all(&mut *tx).await?;

// 3. 如果有无效角色ID，回滚事务
if valid_roles.len() != role_ids.len() {
    tx.rollback().await?;
    return Err(sqlx::Error::RowNotFound);
}

// 4. 插入用户角色关联
sqlx::query(&query_builder).execute(&mut *tx).await?;

// 5. 提交事务
tx.commit().await?;
```

### 3. 错误处理改进

新增 `InvalidRoleId` 错误类型：

```rust
/// One or more role IDs do not exist or are inactive.
#[error("One or more role IDs are invalid")]
InvalidRoleId,
```

错误映射：

- HTTP 状态码：`400 Bad Request`
- 错误代码：`10203`
- 错误消息：`"One or more role IDs are invalid."`

### 4. 服务层改进

修改 `UserService::create_user` 方法：

```rust
// 在事务中创建用户和设置角色
let user = UserRepository::create_with_roles(
    pool,
    &request.username,
    &request.email,
    &password_hash,
    request.real_name.as_deref(),
    request.status.unwrap_or(1),
    &request.role_ids,
).await.map_err(|e| {
    // 检查角色相关错误
    match &e {
        sqlx::Error::RowNotFound => ServiceError::InvalidRoleId,
        _ => ServiceError::DatabaseQueryFailed,
    }
})?;
```

## 事务保证

### 原子性 (Atomicity)

- 用户创建和角色绑定要么全部成功，要么全部失败
- 如果任何一步失败，整个操作回滚

### 一致性 (Consistency)

- 角色 ID 必须存在且状态为正常
- 不会出现用户存在但角色绑定失败的情况

### 验证逻辑

1. **角色存在性验证**：检查所有角色 ID 在 roles 表中存在
2. **角色状态验证**：确保角色未删除且状态为启用（status=1）
3. **完整性验证**：所有提供的角色 ID 都必须有效

## 测试用例

### 1. 正常创建用户

```json
{
  "username": "normaluser",
  "email": "normal@test.com",
  "password": "test123",
  "realName": "正常用户",
  "status": 1,
  "roleIds": [1, 2] // 有效的角色ID
}
```

### 2. 无效角色 ID 测试

```json
{
  "username": "invalidrole",
  "email": "invalid@test.com",
  "password": "test123",
  "realName": "无效角色用户",
  "status": 1,
  "roleIds": [999, 1000] // 不存在的角色ID
}
```

预期结果：

- 用户不会被创建
- 返回 400 错误：`"One or more role IDs are invalid."`

### 3. 部分无效角色 ID 测试

```json
{
  "username": "partialinvalid",
  "email": "partial@test.com",
  "password": "test123",
  "realName": "部分无效角色用户",
  "status": 1,
  "roleIds": [1, 999] // 一个有效，一个无效
}
```

预期结果：

- 用户不会被创建
- 返回 400 错误：`"One or more role IDs are invalid."`
- 事务回滚确保数据一致性

## 好处

1. **数据一致性**：消除了用户创建成功但角色绑定失败的问题
2. **错误处理**：提供更清晰的错误信息
3. **原子操作**：确保用户创建的完整性
4. **性能优化**：减少数据库往返次数
5. **代码简化**：减少了服务层的复杂性

## API 文档更新

在创建用户接口文档中添加了事务说明：

```markdown
### 3. 创建用户

- **URL**: `POST /api/system/users`
- **描述**: 创建新用户
- **事务处理**: 用户创建和角色绑定在同一个事务中进行，确保数据一致性
```

## 注意事项

1. 确保数据库支持事务（PostgreSQL 支持）
2. 角色表必须有正确的约束和索引
3. 测试时需要确保有有效的角色数据
4. 监控事务性能，避免长时间锁定
