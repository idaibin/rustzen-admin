# 数据库迁移文件

本目录包含 rustzen-admin 项目的数据库迁移文件。

## 📁 文件结构

```
backend/migrations/
├── 001_simple_schema.sql  # 极简版表结构
└── README.md             # 迁移说明文档
```

## 🚀 使用方法

### 1. 使用 SQLx 迁移（推荐）

```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features rustls,postgres

# 创建数据库
createdb rustzen_admin

# 运行迁移
sqlx migrate run --database-url "postgresql://username:password@localhost/rustzen_admin"
```

### 2. 手动执行 SQL 文件

```bash
# 执行表结构迁移
psql -U username -d rustzen_admin -f 001_simple_schema.sql
```

## 📋 数据库设计

### 极简版表结构

本项目采用极简设计，包含以下 6 张核心表：

1. **users** - 用户表

   - 基本用户信息：用户名、邮箱、密码、真实姓名等
   - 软删除：使用 `deleted_at` 字段

2. **roles** - 角色表

   - 角色信息：角色名称、描述、状态
   - 软删除：使用 `deleted_at` 字段

3. **user_roles** - 用户角色关联表

   - 多对多关系：用户可以有多个角色

4. **menus** - 菜单表

   - 菜单信息：标题、路径、组件、图标等
   - 支持树形结构：通过 `parent_id` 构建层级
   - 软删除：使用 `deleted_at` 字段

5. **role_menus** - 角色菜单关联表

   - **权限控制核心**：角色可以访问哪些菜单
   - 多对多关系：角色可以访问多个菜单

6. **operation_logs** - 操作日志表
   - 记录用户操作：操作类型、描述、IP 地址等

### 权限模型

采用最简单的 **基于角色的菜单权限控制**：

```
用户(Users) ←→ 角色(Roles) ←→ 菜单(Menus)
```

- **用户** 通过 `user_roles` 关联到 **角色**
- **角色** 通过 `role_menus` 关联到 **菜单**
- **权限控制** = 用户能看到哪些菜单

### 软删除策略

- 使用 `deleted_at` 字段实现软删除
- `deleted_at IS NULL` 表示记录未删除
- `deleted_at IS NOT NULL` 表示记录已删除
- 唯一索引加上 `WHERE deleted_at IS NULL` 条件

## 🔧 数据库配置

### 环境变量

```bash
# PostgreSQL 连接配置
DATABASE_URL=postgresql://username:password@localhost:5432/rustzen_admin
```

### 连接池配置

```toml
[database]
max_connections = 10
min_connections = 5
connect_timeout = 30
```

## 🚨 注意事项

### 生产环境部署

1. **备份数据库**：

   ```bash
   pg_dump rustzen_admin > backup_$(date +%Y%m%d_%H%M%S).sql
   ```

2. **检查权限**：确保数据库用户有足够权限

3. **测试迁移**：在开发环境先测试

### 软删除查询

查询时需要过滤已删除的记录：

```sql
-- 查询未删除的用户
SELECT * FROM users WHERE deleted_at IS NULL;

-- 查询未删除的角色
SELECT * FROM roles WHERE deleted_at IS NULL;

-- 查询未删除的菜单
SELECT * FROM menus WHERE deleted_at IS NULL;
```

## 🔍 故障排查

### 常见问题

1. **权限不足**：检查数据库用户权限
2. **外键约束失败**：检查关联数据是否存在
3. **唯一约束冲突**：检查数据重复（注意软删除条件）

### 调试命令

```bash
# 检查迁移状态
sqlx migrate info --database-url $DATABASE_URL

# 查看表结构
psql -U username -d rustzen_admin -c "\d users"
```

## 📈 扩展建议

如果后续需要更复杂的功能，可以考虑：

1. **细粒度权限**：添加操作权限表
2. **数据权限**：添加数据范围控制
3. **审计日志**：添加 `created_by`、`updated_by` 字段
4. **多租户**：添加租户隔离
5. **缓存优化**：添加 Redis 缓存层

---

**设计原则**：先简单，后复杂。根据实际需求逐步扩展。
