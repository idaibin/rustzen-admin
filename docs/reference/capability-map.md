# Capability Map

This is a short orientation map for current capability ownership. Source code, `docs/architecture.md`, and `docs/guides/` take precedence.

## Current Capabilities

| Capability | Backend owner | Frontend owner |
| --- | --- | --- |
| Auth | `apps/admin/src/features/auth/` | `apps/web/src/api/auth/`, `apps/web/src/routes/login.tsx`, `apps/web/src/routes/__root.tsx` |
| Account | `apps/admin/src/features/account/` | `apps/web/src/api/account/`, `apps/web/src/routes/profile.tsx`, `apps/web/src/components/user/` |
| Dashboard account totals and module health | `apps/admin/src/features/dashboard/`, `apps/admin/src/features/modules/` | `apps/web/src/api/dashboard/`, `apps/web/src/routes/index.tsx` |
| Module registry and gateway | `apps/admin/src/features/modules/`, `crates/ipc/` | `apps/web/src/api/system/module/`, `apps/web/src/routes/system/module.tsx` |
| Monitoring | `apps/monitor/src/features/`, `apps/monitor/src/app.rs` | `apps/web/src/api/monitor/`, `apps/web/src/routes/monitoring/` |
| Analytics | `apps/insights/src/features/`, `apps/insights/src/app.rs` | `apps/web/src/api/insights/`, `apps/web/src/routes/analytics/` |
| Reports, including current Automation behavior | `apps/reports/src/features/automation/`, `apps/reports/src/app.rs` | `apps/web/src/api/reports/`, `apps/web/src/routes/reports/` |
| RBAC carriers | `apps/admin/src/features/system/menu/`, `system/role/`, access-facing `system/user/` | `apps/web/src/api/system/menu/`, `system/role/`, `system/user/`; `apps/web/src/routes/system/` |
| Audit carrier | `apps/admin/src/features/manage/log/` | `apps/web/src/api/manage/log/`, `apps/web/src/routes/manage/log.tsx` |
| Scheduled tasks | `apps/admin/src/features/manage/task/` | `apps/web/src/api/manage/task/`, `apps/web/src/routes/manage/task.tsx` |
| Deploy versions | `apps/admin/src/features/manage/deploy/` | `apps/web/src/api/manage/deploy/`, `apps/web/src/routes/manage/deploy.tsx` |
| Runtime files | `apps/admin/src/common/files.rs`, `apps/admin/src/infra/app.rs`, `crates/config/`, `crates/runtime/` | avatar upload UI under `apps/web/src/components/user/` |

## Rules

- Declare a module route and its permission once through that module's Rust
  `ModuleRouter`; keep `module.toml` limited to metadata and default menus.
- Do not create `rbac`, `audit`, `runtime`, or other new top-level feature folders just to match capability names.
- Keep current carriers in place until a real implementation need requires a new owner.
- When backend contracts move, update matching frontend API modules and types in the same task.
- Automation is not a separately shipped module. Product-boundary changes require
  an approved feature specification grounded in the current repository.
