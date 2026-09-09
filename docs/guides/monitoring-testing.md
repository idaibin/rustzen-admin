# Monitoring Testing

This matrix is the executable acceptance basis for Monitoring. Tests use an in-memory SQLite pool
unless restart durability requires a temporary file database.

## 已落地的行为测试

当前 `cargo test -p rustzen-monitor` 的测试通过真实函数、HTTP
handler 和 SQLite 持久层验证以下边界：

- `protocol::tests::validates_each_wire_field_at_its_declared_boundary` 覆盖身份、主机名、版本、CPU、字节数、序列号和挂载点的包含边界及越界值； malformed RFC3339 和空磁盘列表分别验证。
- `agent::tests::agent_collection_maps_fixed_values_and_skips_invalid_devices` 验证固定 CPU、内存、多挂载点映射，并确认空路径、零容量、非法可用容量、伪文件系统、容器临时挂载、loop 设备和重复本地 bind mount 不会污染上报；真实 `/`、`/data` 与 `/var/lib/docker` 挂载仍保留，同源网络挂载不会被误合并。
- `agent::tests::agent_send_report_observes_request_timeout` 及 `agent_response_statuses_and_failures_are_distinct` 覆盖超时、401、503、accepted、duplicate、stale 和坏响应；`next_sequence` 验证失败/过期不推进序列。独立 Agent 测试入口为 `just verify-monitor-agent`，Controller 的默认测试命令不包含 Agent 测试。
- `agent_loop_keeps_sequence_and_skips_missed_ticks_for_all_send_outcomes` 直接调用生产 `run_agent_loop`，通过可注入 collector/sender/readiness seam 验证 accepted、duplicate 后只发送一次 readiness，401、503、timeout、stale 和 missed tick 不发送 readiness，同时保持既有序列与调度行为。
- `app::tests::agent_report_route_returns_accepted_duplicate_and_stale_envelopes` 验证 Controller 上报 envelope，既有 route 测试验证 token-before-body、401 和 422。
- `historical_duplicate_stale_and_retired_reports_bypass_clock_skew` 与 `historical_fenced_reports_return_200_statuses_but_new_sequence_is_422` 验证 fencing 先于时钟偏差：历史 duplicate/stale/retired 报告仍返回 200 且无副作用，新的 sequence 才返回 422。
- `older_collected_at_is_stale_even_with_an_increasing_sequence`、`report_clock_skew_has_an_inclusive_five_minute_boundary` 验证 collection time 严格递增、五分钟允许偏差和远未来/过去拒绝；服务端 liveness 仍使用 receive time；`concurrent_replay_has_one_accept_and_one_duplicate` 验证 SQLite fencing 竞态不会产生重复样本或 500。
- `threshold_boundary_applies_to_cpu_memory_and_each_disk`、`disabled_offline_alert_does_not_create_incident`、`invalid_alert_settings_are_rejected_without_partial_update`、`metric_buckets_align_by_epoch_and_retain_independent_mount_series`、`file_cleanup_reports_reclaim_maintenance_and_is_idempotent`、`cleanup_retries_maintenance_after_committed_deletion_failure` 和 retention 测试覆盖阈值包含边界、禁用、非法设置、5 分钟桶、挂载点独立序列、文件库 freelist 维护、maintenance 失败结果与重试及保留/清理。
- `sqlite_foreign_key_and_active_incident_uniqueness_are_enforced`、唯一初始化基线和文件重开测试覆盖 FK、active incident 唯一约束、仅创建最终表结构和重启持久化。

当前验收基线：Controller 47 项与 Agent 12 项测试通过；`just check` 和 Admin OpenAPI/client
契约检查通过。`just verify-modules-mvp` 的底层服务验证脚本通过全新的四个服务数据库验证
24 种启动顺序、原生 macOS Agent 经 Admin 网关上报、服务隔离与数据库恢复。
新增 `scripts/verify-monitoring-scenarios.mjs` 验证真实 owner/viewer 权限、四个导航入口、
全局与节点策略、重复/过期隔离、CPU/磁盘告警三次触发与
三次恢复、事件分页/详情、非法输入和 30 天查询边界。该脚本被统一 worker 验证入口调用。

浏览器已覆盖四个页面、节点多挂载点详情、自定义策略保存与重置、事件详情、设置保存及
日报空状态。完整分页/筛选组合、日报非 30 秒采样覆盖率、Linux/Windows 真实采集、部署
环境定时任务及完整视觉/异常状态矩阵仍需单独验收。Overview/Nodes 当前没有分页契约，
分页验收适用于 Incidents/Summaries；上述证据不能替代生产部署验证。

## P0 report and persistence

- Reject missing or wrong Agent token.
- Reject empty/invalid node ID, invalid timestamp, non-finite/out-of-range CPU, zero totals,
  `used > total`, empty mount point, and duplicate mount points.
- Prove every rejected report leaves node time, samples, counters, and Incidents unchanged.
- Accept the first `bootId` report only at sequence one.
- Accept strictly increasing sequence for the current boot.
- Return `duplicate` for an exact replay without side effects.
- Return `stale` for a lower sequence, retired boot, or non-one new-boot takeover.
- Prove a new boot at sequence one takes ownership and retires the prior boot.
- Inject a failure after sample insertion and prove the complete report transaction rolls back.
- Prove CPU, memory, and every disk mount are stored independently.

## P0 resource alerts

Run the same behavior table for CPU, memory, `/`, and `/data`:

| Samples | Expected result |
| --- | --- |
| high | abnormal count 1, no Incident |
| high, high | abnormal count 2, no Incident |
| high, high, high | one active Incident |
| fourth high | still one active Incident, latest evidence updated |
| high, normal, high | abnormal count reset; no false trigger |
| active + normal | active, normal count 1 |
| active + normal, normal | active, normal count 2 |
| active + normal, normal, normal | resolved |

Also prove that `/` and `/data` do not share counters or Incidents and that one target cannot have
two active Incident rows.

## P0 offline alerts

- At exactly the configured boundary the node remains online.
- After the boundary one offline scan creates one active Incident.
- Repeated scans retain one active Incident.
- The next accepted report resolves it immediately.
- A duplicate, stale, or retired-boot report cannot resolve it or refresh liveness.
- Disabling offline alerts resolves active offline Incidents and prevents new ones.

## P0 restart durability

Using a file-backed SQLite database:

- accept reports until an abnormal or normal counter reaches two;
- close and reopen Controller storage;
- prove the next matching report reaches three and performs the expected transition;
- prove nodes, latest state, settings, samples, and active/resolved Incidents survive restart.

## P1 settings

- Defaults are CPU 90%, memory 90%, disk 90%, and offline 90 seconds.
- Reject thresholds outside 1-100 and offline duration outside 30-3600 seconds.
- A node without an override always reads the current global defaults.
- Saving node settings creates one complete custom override and marks its source as custom.
- A global edit resets only affected unfinished counters for inheriting nodes; custom nodes keep
  their effective settings and counters.
- Resetting a custom node removes its override, returns the current global defaults, and resets only
  counters whose effective setting changed.
- A threshold edit resets only affected unfinished counters.
- An active Incident is evaluated against the new threshold from the next accepted report.
- Disabling CPU, memory, disk, or offline resolves only that type with `setting disabled`.
- Disabled settings do not create new Incidents.

## P1 retention and summaries

- Delete samples, resolved Incidents, and daily summaries strictly older than the 30-day cutoff.
- Keep rows exactly at the cutoff.
- Keep latest node state, active Incidents, and unfinished counters regardless of age.
- Run cleanup twice and prove idempotence.
- Generate one daily summary per node/date and prove rerunning with unchanged input is idempotent.
- Prove CPU/memory and every mount summary use only accepted samples for that day.
- Prove the API rejects query windows longer than 30 days.

## P1 Agent behavior

- One collection contains CPU, memory, and each eligible real disk mount once per local block
  device; pseudo filesystems and ephemeral container mounts are absent.
- Successful, rejected, and network-failed submissions all wait for the next scheduled interval.
- The installed Agent sends its system-service readiness signal exactly once only after `accepted`
  or `duplicate`; `401`, `stale`, malformed/other HTTP responses, TLS failures, and network failures
  leave it unready.
- No path persists, retries, or backfills an old sample.
- Test scheduling with an injected interval/clock; do not sleep for real 30-second periods.

## P1 API and Manifest

- Runtime Manifest is derived from registered Rust routes.
- `agent-reports` is public but still enforces Agent token.
- Every protected route has the documented capability.
- There is no `heartbeat`, checks, check-result, or acknowledged-Incident route.
- Backend response fields match `apps/web/src/api/monitor` consumers.
- Error tests cover `401`, `404`, and `422` plus `accepted`, `duplicate`, and `stale` report results.

## P2 Web

- Navigation contains Overview, Nodes, Incidents, and Daily summaries only; Global settings opens from Nodes.
- Overview distinguishes online/offline nodes and active Incidents.
- Node details display independent disk-mount series and a maximum 30-day query.
- Incidents display only active/resolved states.
- Users without `monitor:manage` cannot mutate settings.
- Loading, empty, error, disabled-alert, stale-node, and partial-day states are distinguishable.

## Implementation order and gates

1. Migration and storage constraints.
2. Agent report DTO, validation, fencing, and atomic persistence.
3. Resource alert counters and Incident transitions.
4. Offline scan.
5. Settings mutation.
6. Daily summaries and retention.
7. ModuleRouter, capability, and Web contract.
8. UI after its separate specification is Ready.

After each slice run its named focused tests, then `cargo fmt --all -- --check`,
`cargo check -p rustzen-monitor`, `cargo clippy -p rustzen-monitor --all-targets -- -D warnings`,
`cargo test -p rustzen-monitor`, and `git diff --check`. Cross-module completion additionally runs
`just verify-modules-mvp`; browser and deployed multi-node behavior remain separate gates.

## Linux dual-Agent runtime gate

`just verify-monitor-agent-multi-node-linux` is a bounded, native-architecture
Colima/Docker check. It creates a fresh central Admin and Monitor SQLite state plus
two independent unprivileged Agent service identities. Each Agent has its own node ID,
runtime root, log directory, and Unix readiness socket. Before central services start,
neither readiness socket may receive `READY=1`. Once Admin and Monitor are healthy,
the gate requires both Agent reports to be accepted, verifies the two nodes through the
Admin gateway, checks distinct current boot IDs and raw metric samples, restarts the
central processes, and requires both existing nodes to continue reporting without a
third registration.

Evidence is atomically published below
`target/rz/monitor-agent-multi-node/current/manifest.json`. The manifest contains the
source digest, platform, staged binary digests, service UID evidence, readiness events,
Admin query outcomes, and recovery result; it never contains the Agent token. This is
real Linux process and service-account evidence, not a systemd PID 1, remote-host,
production TLS, or independent-kernel test.

## Linux dual-Agent Nodes Chromium gate

`just verify-monitor-agent-multi-node-ui-linux` reuses the two real Agent runtime
topology and adds the Admin Web shell plus Reports-driven Chromium. It binds the
current source identity and SHA-256 values for Admin, Monitor, Reports, and Agent
binaries. The gate obtains the two node ID/current boot ID pairs from Admin, opens the
Nodes list and each detail Drawer, checks the same boot ID and the 5-minute history
surface, and publishes one ordered browser-step receipt plus two 1440x900 detail PNGs
with SHA-256 and byte-size evidence. The manifest also binds the exact submitted
step list and the Reports API's persisted flow definition before it accepts the
receipt's contiguous successful step sequence.

Its evidence is local, disposable Linux-container evidence only. It does not replace
the native dual-Agent gate and does not certify systemd PID 1, independent hosts,
production TLS, or the complete Monitoring UI state matrix.
