# Tauri 桌面端模块

此目录用于存放 `rustzen-admin` 的可选 Tauri 桌面客户端。

## 目录结构

Tauri 采用混合架构，其标准目录结构如下：

```
desktop/
├── src/                      # 前端 UI 代码 (通常会直接引用根目录的 frontend)
├── package.json              # 前端依赖
└── src-tauri/                # 独立的 Rust 后端项目
    ├── Cargo.toml
    ├── tauri.conf.json       # Tauri 应用核心配置
    └── src/
        └── main.rs
```

`src-tauri` 目录是 Tauri 框架的硬性规定，用于存放所有与原生窗口、系统 API 交互的 Rust 代码。

## 快速开始

当需要开发桌面端时，可以在 `desktop` 目录下，通过以下命令初始化一个完整的 Tauri 项目：

```bash
# 假设前端部分已准备好
pnpm create tauri-app
# 或者使用 cargo-binstall 等工具安装 tauri-cli 后
# cargo tauri init
```

这将自动生成所有必需的文件，包括 `src-tauri` 内部的结构。
