# Capability Map

This is a short orientation map for current capability ownership. Source code, `docs/architecture.md`, and `docs/guides/` take precedence.

## Current Capabilities

| Capability | Backend owner | Frontend owner |
| --- | --- | --- |
| Auth | `apps/server/src/features/auth/` | `apps/web/src/api/auth/`, `apps/web/src/routes/login.tsx`, `apps/web/src/routes/__root.tsx` |
| Account | `apps/server/src/features/account/` | `apps/web/src/api/account/`, `apps/web/src/routes/profile.tsx`, `apps/web/src/components/base-user/` |
| Dashboard | `apps/server/src/features/dashboard/` | `apps/web/src/api/dashboard/`, `apps/web/src/routes/index.tsx` |
| RBAC carriers | `apps/server/src/features/system/menu/`, `system/role/`, access-facing `system/user/` | `apps/web/src/api/system/menu/`, `system/role/`, `system/user/`; `apps/web/src/routes/system/` |
| Audit carrier | `apps/server/src/features/manage/log/` | `apps/web/src/api/manage/log/`, `apps/web/src/routes/manage/log.tsx` |
| Dictionary | `apps/server/src/features/manage/dict/` | `apps/web/src/api/manage/dict/`, `apps/web/src/routes/manage/dict.tsx` |
| Scheduled tasks | `apps/server/src/features/manage/task/` | `apps/web/src/api/manage/task/`, `apps/web/src/routes/manage/task.tsx` |
| Deploy versions | `apps/server/src/features/manage/deploy/` | `apps/web/src/api/manage/deploy/`, `apps/web/src/routes/manage/deploy.tsx` |
| Runtime files | `apps/server/src/common/files.rs`, `apps/server/src/infra/config.rs`, `apps/server/src/infra/app.rs` | avatar upload UI under `apps/web/src/components/base-user/` |

## Rules

- Do not create `rbac`, `audit`, `runtime`, or other new top-level feature folders just to match capability names.
- Keep current carriers in place until a real implementation need requires a new owner.
- When backend contracts move, update matching frontend API modules and types in the same task.
