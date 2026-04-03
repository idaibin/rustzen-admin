# 🤖 AI 使用规范说明

## 📦 依赖安装规范

- 所有 Rust 依赖必须通过 `cargo add xxx@latest` 添加；
- 所有前端依赖必须通过 `pnpm add xxx@latest` 添加；
- 不允许使用未指定版本或默认旧版本；
- 所有依赖应写入 `Cargo.toml` / `package.json`，避免版本不一致。

## 🧱 项目任务管理

- 所有项目开发/构建命令请通过 `just` 调用；
- 启动后端：`just dev-server`
- 启动前端：`just dev-web`
- 构建生产版本：`just build`
- 清理构建产物：`just clean`

## 🧩 项目模块约定

- Rust 后端模块拆分为：`mod.rs`、`handler.rs`、`service.rs`、`repo.rs`、`types.rs`
- React 前端模块集中于 `src/`，使用 `zustand` 管理共享状态，API 逻辑放在 `src/api/`
- 页面接口类型统一声明在 `src/api/<module>/`，不要在页面里重复定义

## 📑 文件/目录约定

- 所有临时生成文件请放在 `.tmp/`，不要提交
- 所有文档应放在 `docs/` 中，推荐使用 Markdown
