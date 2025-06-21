# 🤖 AI 使用规范说明

## 📦 依赖安装规范

- 所有 Rust 依赖必须通过 `cargo add xxx@latest` 添加；
- 所有前端依赖必须通过 `pnpm add xxx@latest` 添加；
- 不允许使用未指定版本或默认旧版本；
- 所有依赖应写入 `Cargo.toml` / `package.json`，避免版本不一致。

## 🧱 项目任务管理

- 所有项目开发/构建命令请通过 `just` 调用；
- 启动开发模式：`just dev`
- 构建生产版本：`just build`
- 清理构建产物：`just clean`

## 🧩 项目模块约定

- Rust 后端模块拆分为：`routes/`、`service/`、`repo/`、`model/`
- React 前端模块集中于 `src/`，使用 `hooks` 管理业务逻辑，`zustand` 管理状态，`swr` 请求数据
- 页面接口类型统一声明在 `types/`，使用 `declare module` 按模块划分

## 📑 文件/目录约定

- 所有临时生成文件请放在 `.tmp/`，不要提交
- 所有文档应放在 `docs/` 中，推荐使用 Markdown
