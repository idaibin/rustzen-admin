⸻

# 📦 Rust 项目数据库迁移规范（rustzen-admin）

> 本文档为 rustzen-admin 项目设计的数据库迁移规范，遵循“模块分组 + 安全顺序 + Zen 风格”，确保迁移清晰有序、结构可扩展、依赖不出错，适用于中大型 Rust Web 项目。

---

## ✳️ 命名规则概览

迁移文件统一格式如下：

```text
{模块分类编号}{模块编号}{步骤编号}_{动作}_{对象}.sql

命名示例：

文件名	含义
100100_create_user.sql	用户模块创建表
100200_create_role.sql	角色模块创建表
107000_view_user.sql	用户相关视图（含关联查询）
108000_func_user.sql	用户模块相关函数
109000_seed_user.sql	用户模块初始数据


⸻

📦 目录结构示例（Zen 规范 + 历史归档）

migrations/
├── 100100_create_user.sql
├── 100200_create_role.sql
├── 100300_create_menu.sql
├── 100400_create_log.sql
├── 100500_create_user_role.sql
├── 100600_create_role_menu.sql
├── 100700_create_dict.sql
├── 100800_create_foreign_keys.sql
├── 100900_create_triggers.sql
├── 107000_view_user_with_roles.sql
├── 107001_view_user_permissions.sql
├── 107002_view_user_menu_info.sql
├── 107003_view_user_info_summary.sql
├── 107004_view_user_info_stats.sql
├── 107005_view_recent_operations.sql
├── 107006_view_user_activity_summary.sql
├── 108000_func_user_role.sql
├── 108001_func_user_menu_data.sql
├── 108002_func_user_basic_info.sql
├── 108003_func_user_permissions.sql
├── 108004_func_user_has_permission.sql
├── 108005_func_login_credentials.sql
├── 108006_func_log_operation.sql
├── 108007_func_log_operations_bulk.sql
├── 108008_func_create_log_partition.sql
├── 108009_func_manage_log_partitions.sql
├── 108010_func_get_log_partition_info.sql
├── 108011_func_analyze_user_query_performance.sql
├── 109000_seed_user.sql
├── 109001_seed_menu.sql
├── 109002_seed_role.sql
├── 109003_seed_dict.sql
├── 109004_seed_log.sql
├── legacy/
│   ├── 001_system_schema.sql
│   ├── 002_system_seed.sql
│   ├── 003_log_system.sql
│   └── 004_user_info_optimization.sql

⸻

📐 模块编号规范

每个功能模块编号区间预留 100 个范围，便于扩展：

模块类别	起始编号	示例文件
系统设置	100000	100000_create_setting.sql
用户管理	100100	100100_create_user.sql
角色权限	100200	100200_create_role.sql
菜单结构	100300	100300_create_menu.sql
日志审计	100400	100400_create_log.sql
数据字典	100500	100500_create_dict.sql
通用枚举	100600	100600_create_enum.sql

🚨 若跨模块使用（如 user view 依赖 role），请将 View、Func 等延后统一集中在 1097xx~1099xx。

⸻

🔄 文件类型与执行顺序

顺序	类型	编号范围	文件命名示例	说明
1️⃣	表结构定义	100000+	100100_create_user.sql	所有表结构，必须先执行
2️⃣	索引优化	合并至表结构	同上	索引建议与建表一起写入，避免碎片化
3️⃣	视图定义	107000+	107999_view_user.sql	需等待依赖的表存在后执行
4️⃣	函数定义	108000+	108000_func_user.sql	依赖视图/表，必须延后执行
5️⃣	数据初始化	109000+	109999_seed_user.sql	插入基础数据，结构建立完毕后执行


⸻

🧘 Zen 风格设计建议
	•	✅ 每个模块保持清晰编号，解耦执行顺序
	•	✅ 所有视图和函数集中编号段维护，避免循环依赖
	•	✅ 保持命名语义明确：create_、view_、func_、seed_
	•	✅ 所有 SQL 文件可阅读、可维护、可复用

⸻

🧪 示例：用户模块 SQL 文件

migrations/
├── 100100_create_user.sql          # 建表 + 索引
├── 107000_view_user.sql           # 联表 user + roles
├── 108000_func_user.sql           # 登录验证、权限检查
├── 109000_seed_user.sql           # 插入 admin 账号


⸻

🧱 函数依赖注意事项

PostgreSQL 中函数在“定义时”验证依赖，不是执行时！

因此：
	•	❗ 函数使用的 表、视图、函数 都必须在它之前存在；
	•	❗ 所以函数文件务必在所有依赖对象 之后执行（建议在 108000+ 范围）；

⸻

🎯 命名风格推荐（以 user 为例）

类型	命名示例	说明
表结构	100100_create_user.sql	user 表定义，含字段、索引、约束等
视图	107000_view_user.sql	user + roles 联表查询封装
函数	108000_func_user.sql	get_user_basic、check_permission
初始化数据	109000_seed_user.sql	初始超级管理员账号


⸻

📌 常见问题解答（FAQ）

Q: 视图是否必须跨表才定义？

A: 不必须。如果单表字段映射复杂或多次复用，也建议定义 View 简化代码。

Q: 函数可以只用 SQL 实现吗？

A: 是的。封装查询逻辑、权限判断、聚合统计等，都适合用 SQL 函数实现，提高性能和复用性。

Q: 文件过多怎么办？

A: 模块统一命名、集中编号设计，即使几十个文件也不会混乱。

⸻

🧩 示例推荐编号

类型	编号范围	示例文件
表结构	100100	100100_create_user.sql
视图	107000	107000_view_user.sql
函数	108000	108000_func_user.sql
初始化	109000	109000_seed_user.sql


⸻

🧭 结语

该规范已在 rustzen-admin 项目中实践并验证，适用于多模块、强依赖、长生命周期项目。如果你想让你的 Rust Web 项目具备良好可维护性、强健的数据层结构，建议全面采用。

欢迎在你的项目或博客中引用此规范，如需协助整合项目结构，可联系作者继续优化。

```
