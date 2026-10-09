# Vercel Container Images 适用性研究

核对日期：2026-10-09。状态：**研究完成；未部署、未构建 Vercel 镜像、未运行 Vercel 预览。**本文件是候选方案，不改变当前签名包与 systemd 发布契约（见 [部署指南](../guides/deployment.md)）。

## 平台事实

- [Container Images](https://vercel.com/docs/functions/container-images) 仍为 Beta，面向所有计划。项目根目录的 `Dockerfile.vercel` 或 `Containerfile.vercel` 会被自动发现、构建为 OCI 镜像并存入 Vercel Container Registry (VCR)；通过 CLI 或已连接仓库的推送可触发部署。因此研究阶段不添加这些自动发现文件或 `vercel.json`。
- [Services](https://vercel.com/docs/services) 也为 Beta。官方配置支持服务 `runtime: "container"`、`root`、相对于该 root 的 `entrypoint`，以及顶层 `rewrites` 将公开请求送到指定服务。官方 [Rust/Axum 示例](https://vercel.com/kb/guide/deploy-rust-on-vercel-with-docker) 使用 `Dockerfile.vercel` 和这些字段。这里没有确认 `services.image`、`healthcheckPath` 或直接拉取第三方私有 registry 的部署字段。
- 容器仍作为**请求驱动的 Vercel Function**运行，须监听 `0.0.0.0:$PORT`（平台默认端口 80）。无流量时 production 约 5 分钟、preview 约 30 秒后缩容；缩容发出 SIGTERM，宽限 30 秒。容器文件系统与进程内存不能作为持久状态。见 [Container Images](https://vercel.com/docs/functions/container-images) 与 [官方 Axum 示例](https://vercel.com/kb/guide/deploy-rust-on-vercel-with-docker)。
- [Functions 限制](https://vercel.com/docs/functions/limitations) 将响应流计入单次请求时长，并规定请求或响应体上限 4.5 MB。[官方工作进程指南](https://vercel.com/kb/guide/docker-monolith-workers-vercel) 指出容器在 Pro/Enterprise 最高按 800 秒规划；1800 秒扩展 Beta 仅覆盖指定 Node.js、Bun、Python 运行时，不包括容器镜像。不能把镜像视为常驻主机。
- [WebSockets](https://vercel.com/docs/functions/websockets) 在 Functions 中为 Beta，连接到达时长上限会关闭，客户端须重新连接和恢复订阅。本仓库当前通知实时通道使用 SSE，不能把 WebSocket 支持直接当成 SSE 路径通过的证据。

## 与本仓库的对应关系

| 当前实现 | 影响与判断 |
| --- | --- |
| `apps/admin/src/infra/app/runtime.rs` 由 Admin Axum 托管嵌入式 Web，Admin 网关在固定 loopback 端点访问 Monitor、Insights、Reports；`docs/architecture.md` 定义四个独立服务与一个签名回滚边界。 | 单个 Rust Axum HTTP 入口在技术上可以作为容器候选；直接把当前四进程签名包装入一个 Function 会改变故障域、服务间寻址、启动与发布契约，不能从官方最小示例直接推出可用。Services 多服务路由与内部通信需要单独设计和实测。 |
| `crates/config/src/` 与 `crates/storage/src/sqlite.rs` 使用各模块的本地 SQLite 路径；Admin 发布、日志、头像、Reports 输出和自动化产物也写入本地目录。 | Function 的非持久文件系统不适合作为当前四库的主存储或持久文件输出。迁到外部持久存储将是产品与数据层改造，不属于本次清理；不能仅凭换 Dockerfile 宣称全产品可部署。 |
| `apps/reports/src/features/automation/scheduler.rs`、`apps/monitor/src/features/monitoring/background/mod.rs`、各模块通知 relay 等启动后台循环；Monitor Agent 每 30 秒采集主机指标。 | 缩容会中止这些循环，也无法把 Function 当作持续监控主机。官方建议将计划任务、队列消费者、长期进程拆到 Cron/Queues/Workflows 或其他常驻平台；本仓库尚无等价移植。 |
| `apps/admin/src/features/notifications/realtime/` 使用 SSE；Reports 提供自动化运行与实时帧；Admin 发布接口接收完整签名包。 | SSE 虽属流式 HTTP，仍受单次请求时长约束；断线重连、代理缓冲与认证需实测。自动化任务时长和发布上传大小还需逐项对照平台限制，不能假定现有路径可用。 |

## 可执行的下一阶段验证方案

1. 保持当前本地签名包/systemd 主部署不变，先做**隔离的无状态 Axum 样机**：在独立测试项目中用多阶段 `Dockerfile.vercel` 编译最小 HTTP 入口，使其读取 `PORT`、监听 `0.0.0.0` 并在 SIGTERM 内收尾。不要在本仓库加入会自动触发部署的配置。
2. 若要验证 Rustzen Admin 的真实切片，先确定可持久存储、四服务之间的安全通信和后台工作替代方案；分别实现并测试，不复用本地 SQLite 文件作为生产主库。用隔离测试数据测冷启动、并发、SSE 重连、超时、4.5 MB 载荷边界、SIGTERM 与缩容恢复。
3. 再对照当前签名包、RBAC/HMAC 网关及发布回滚契约审查方案。只有这些阻碍解除且环境负责人授权后，才创建项目、连接 Git、部署预览或生产。当前没有这些运行证据。

结论：Vercel Container Images **可用于验证一个无状态 Axum HTTP 切片**；现有 Rustzen Admin 完整四服务产品**不能直接按当前实现迁入**。这一判断结合了上述官方平台约束与本仓库代码；尚未进行 Vercel 构建、部署或端到端运行验证。
