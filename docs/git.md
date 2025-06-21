# ✍️ Git 提交规范 (Git Commit Convention)

本规范为 `rustzen-admin` 项目的 Git 提交信息约定，旨在提升日志可读性、自动化生成 CHANGELOG，并为 AI 辅助工具提供上下文。

规范基于 [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)，并针对本项目进行了模块化定制。

---

## 格式

```
<type>(<scope>): <subject>
```

---

## 提交类型 (Type)

| 类型 (Type) | 说明 (Description)                                          |
| :---------- | :---------------------------------------------------------- |
| `feat`      | 新功能 (Features)                                           |
| `fix`       | 修复 Bug (Bug Fixes)                                        |
| `docs`      | 文档更新 (Documentation only)                               |
| `style`     | 样式调整 (Formatting, spacing, etc.)                        |
| `refactor`  | 代码重构 (No new features or bug fixes)                     |
| `test`      | 添加或修改测试 (Adding or modifying tests)                  |
| `chore`     | 构建、工具或依赖更新 (Build process, tooling, dependencies) |
| `perf`      | 性能优化 (Performance improvements)                         |
| `ci`        | CI/CD 配置与脚本                                            |
| `build`     | 构建系统或外部依赖的变更                                    |
| `revert`    | 回滚之前的提交                                              |

---

## 功能范围 (Scope)

`scope` 用于描述本次提交影响的范围，例如功能模块、分层等。

| Scope   | 对应模块/目录 (Corresponding Module/Directory)     |
| :------ | :------------------------------------------------- |
| `api`   | 后端 API 相关                                      |
| `user`  | 用户管理模块                                       |
| `role`  | 角色管理模块                                       |
| `auth`  | 登录鉴权                                           |
| `ui`    | 前端通用 UI 变更                                   |
| `types` | 类型定义变更                                       |
| `deps`  | 依赖更新 (e.g., `deps(frontend)`, `deps(backend)`) |
| `infra` | 构建、部署、CI/CD 工具 (Infrastructure)            |
| `docs`  | 文档内容更新                                       |

---

## 提交主题 (Subject)

`subject` 是对提交的简短描述，遵循以下原则：

- **使用祈使句**：例如使用 `add` 而不是 `added` 或 `adds`。
- **小写开头**：句首单词无需大写。
- **无结尾句号**：结尾不加 `.`。
- **简明扼要**：建议不超过 50 个字符。

---

## ✅ 提交示例

- **新功能**: `feat(user): add user role assignment logic`
- **修复 Bug**: `fix(api): correct pagination query in user list`
- **文档**: `docs(readme): update development startup instructions`
- **样式**: `style(ui): adjust table spacing and button size`
- **重构**: `refactor(auth): simplify jwt middleware injection`
- **依赖**: `chore(deps): bump sqlx to 0.7.1`

---

## 🛡️ 规范守护 (Linting)

为了保证提交规范的严格执行，可以引入以下工具：

| 工具 (Tool)            | 说明 (Description)               | 是否依赖 Node.js |
| :--------------------- | :------------------------------- | :--------------- |
| `commitlint` + `husky` | 前端项目中最常见的组合           | ✅ 是            |
| `lefthook`             | Rust 友好的跨语言 Git Hooks 工具 | ❌ 否            |
| CI 校验                | 在 GitHub Actions 中增加校验步骤 | ❌ 否            |

这些工具可以自动检查每次的 `git commit` 信息是否符合规范，从而在源头保证日志质量。
