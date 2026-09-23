# rustzen-admin

简体中文 | [English](./README-en.md)

`rustzen-admin` 在一个源码仓库和一个签名发布包中提供 Rustzen Admin、Monitor、Insights 和 Reports 运行时。

这是一个面向小型自托管环境的轻量运维与管理产品，同时也是结构化的 Rust 全栈参考实现。

> `rustzen-admin` 将 Axum 后端、React 前端、共享 crate、部署资产和仓库级文档组织在同一个代码库中，强调清晰边界、可维护性，以及对 AI 协作友好的工程结构。

## 概览

`rustzen-admin` 首先是一个可直接部署的开源管理产品，并保留清晰、可复用的工程结构供二次开发参考，而不只是孤立的 UI 演示。

仓库采用 monorepo 组织方式：

- `crates/auth/` 包含 Rust 服务共享的认证与权限能力
- `crates/ipc/` 包含共享模块响应与分页、健康响应、Manifest、路由和 HMAC 委托契约
- `crates/storage/` 包含共享 SQLite 连接池和维护能力
- `apps/admin/` 包含 Admin API、网关、RBAC、发布管理和 Web 资源托管
- `apps/monitor/` 提供监控能力和可选的受管节点 Agent
- `apps/insights/` 提供产品分析和公共追踪脚本
- `apps/reports/` 提供报表模板、填报执行和实时运行视图
- `apps/cli/` 提供不常驻的统一运维命令 `rz`
- `apps/web/` 包含 React 前端应用
- `deploy/` 包含部署资产和发布支持文件
- `docs/` 包含仓库级架构与开发指南
- 仓库根目录保存共享命令、工作区元数据和协作入口文档

该结构明确了后端、前端和仓库规则，便于理解、审查和持续演进。

## 页面预览

以下为当前完整服务候选在 Linux 中运行时，以 1920×1080 浏览器视口截取的
代表性页面。19 个登录后业务路由均有候选截图；完整清单见
[发布截图索引](./docs/assets/screenshots/README.md)，验收边界见
[本地验证记录](./docs/guides/local-verification.md)。

| 仪表盘 | 监控节点 |
| --- | --- |
| ![仪表盘](./docs/assets/screenshots/dashboard.png) | ![监控节点](./docs/assets/screenshots/monitoring-nodes.png) |

| 分析明细 | 自动任务 |
| --- | --- |
| ![分析明细](./docs/assets/screenshots/analytics-details.png) | ![自动任务](./docs/assets/screenshots/management-scheduled-tasks.png) |

## 仓库结构

→ 架构概览：[docs/architecture.md](./docs/architecture.md)

## 产品方向

→ 产品定位、发展方向与模块边界：[docs/product/product.md](./docs/product/product.md)

## 文档

→ 完整文档索引：[docs/README.md](./docs/README.md)

## 命令入口

根目录 `justfile` 是命令的事实来源；执行前请先查看对应目标。
本地开发与测试构建保留可用回溯信息，但不生成完整变量级调试数据，避免大型
workspace 的 `target/` 被非必要调试信息持续放大。

```bash
cargo run -p rustzen-admin -- serve
cargo run -p rustzen-monitor -- controller
cargo run -p rustzen-insights -- serve
cargo run -p rustzen-reports -- serve
cargo run -p rustzen-cli -- --json doctor
```

部署包会把 `rz` 与四个服务二进制安装到同一
`/opt/rz/current/bin/` 目录，并创建 `/usr/local/bin/rz` 命令入口。`rz` 不加入 `rz-full.service`，不拥有数据库，
也不会合并四个服务的进程或故障域。整体服务命令为：

```bash
rz --help
rz start
rz stop
rz restart
rz status
```

直接查看 systemd 时统一使用 `systemctl status rz-full`；systemd 内部对应的
正式单元名是 `rz-full.service`。

正式发布只有一条安装路径：`just build` 生成包含 Web 与四个服务的签名完整包，
将包和构建生成的安装脚本复制到服务器，执行安装脚本后运行 `rz start`。
安装不要求输入管理员密码、签名密钥、验证公钥或数据库命令；首次启动时各服务
自行创建并校验其 SQLite 数据库。

完整构建产物统一生成到 `target/rz/`：

```text
target/rz/rz-<version>-<arch>.tar
target/rz/rz-install
```

前者是包含 Web、Admin、Monitor、Insights、Reports 和 `rz` 的签名完整包，
后者是服务器安装脚本。安装时只需把这两个文件复制到服务器：

```bash
sudo ./rz-install ./rz-<version>-<arch>.tar
sudo rz start
rz status
```

JSON 成功响应固定包含 `schema_version`、`ok`、`command` 和 `data`；
CLI 参数或执行失败固定包含 `schema_version`、`ok`、`command` 和
`error.code` / `error.message`。`doctor` 只按端点白名单读取配置，
凭据键不会进入 CLI 状态，也不会输出凭据值。

本地启动默认使用 SQLite，不需要 PostgreSQL。
SQLite 连接能力、角色策略、运行时目录和日志均由本仓库维护，不依赖 `rustzen-core` 运行时。
本地开发不需要 `.env`：数据库路径、端口、连接池限制、运行目录、日志、时区、JWT 有效期，以及仅用于开发的 JWT/IPC 密钥均有内置默认值。环境变量只用于覆盖这些默认值。

如果启动出现 `VersionMismatch`，说明本地数据库结构与当前初始化 SQL 校验和不一致。请执行：

```bash
just reset-db
cargo run -p rustzen-admin -- serve
```

再次启动后会自动重建数据库。

## 演示环境

- 本地演示地址：[https://admin.rustzen.dev](https://admin.rustzen.dev)
- 用户名：`owner`
- 演示密码：`rustzen@123`

该公开凭据仅用于上述演示环境。正式安装仍会随机生成初始密码，由服务器 root
用户从 `/opt/rz/data/initial-owner-password` 读取；用户修改密码后，以修改后的密码为准。

## 说明

- `README.md` 和 `AGENTS.md` 只保留轻量入口信息。
- `docs/history/` 保存历史执行记录，不是当前实现事实来源。

## 许可证与商标

源代码采用 [Apache License 2.0](./LICENSE.md) 许可，可在该许可证约束下进行商业使用、修改和分发。
Rustzen 名称、Logo、域名、官方包命名空间和官方分发渠道不包含在软件许可证授权范围内。详见 [NOTICE.md](./NOTICE.md) 和 [TRADEMARKS.md](./TRADEMARKS.md)。
