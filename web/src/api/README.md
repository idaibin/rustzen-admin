# API Service 规范

## 🏗️ 目录结构

```
api/
├── request.ts                    # 核心请求工具
├── auth/
│   └── index.ts                  # 认证相关API
├── system/
│   ├── index.ts                  # 系统模块统一导出
│   ├── user.ts                   # 用户管理API
│   ├── role.ts                   # 角色管理API
│   ├── menu.ts                   # 菜单管理API
│   ├── dict.ts                   # 字典管理API
│   └── log.ts                    # 日志管理API
└── index.ts                      # 全局统一导出

types/
├── api.d.ts                      # 核心API类型
├── auth.d.ts                     # 认证模块类型
├── system.d.ts                   # 系统管理类型
└── ...                           # 其他领域类型
```

## 🎯 命名与导出规范

-   **每个领域（如 system、auth）有独立目录和 index.ts 统一导出。**
-   **全局统一导出在 `api/index.ts`，便于一站式引入。**
-   **所有 CRUD 方法命名统一：**
    -   获取表格数据：`getTableData`
    -   新增：`create`
    -   更新：`update`
    -   删除：`delete`
    -   获取选项：`getOptions`
    -   其他特殊结构（如树）：`getTreeData`、`getMenuTree` 等

## 📦 类型与参数规范

-   **所有 API 类型集中在 `types/` 目录下，按领域划分。**
-   **API 参数、返回值类型均需显式声明，便于类型推断和 IDE 智能提示。**

## 💡 推荐用法

-   **统一从 `@/api` 导入所需 API：**
    ```ts
    import { userAPI, roleAPI, menuAPI, dictAPI, logAPI, authAPI } from "@/api";
    ```
-   **如需领域内细分，也可从子目录导入：**
    ```ts
    import { userAPI } from "@/api/system";
    import { authAPI } from "@/api/auth";
    ```

## 🚀 扩展指引

1. 在 `types/` 下新建领域类型声明文件。
2. 在 `api/` 下新建对应 service 文件。
3. 在领域 `index.ts` 统一导出。
4. 在全局 `api/index.ts` 统一导出。

## 🌟 设计优势

-   结构清晰，易于维护和扩展
-   命名统一，便于团队协作
-   类型安全，IDE 友好

---

**如有特殊结构（如菜单树），建议在 api 层处理好，组件层直接用即可。**

**本规范适用于当前 API 目录结构和命名，后续如有调整请同步更新。**
