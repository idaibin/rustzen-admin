# Capability Map

This is a short orientation map for current capability ownership. Source code, `docs/architecture.md`, and `docs/guides/` take precedence.

## Current Capabilities

| Capability | Backend owner | Frontend owner |
| --- | --- | --- |
| Auth | `zen-server/src/features/auth/` | `zen-web/src/api/auth/`, `zen-web/src/routes/login.tsx`, `zen-web/src/routes/__root.tsx` |
| Account | `zen-server/src/features/account/` | `zen-web/src/api/account/`, `zen-web/src/routes/profile.tsx`, `zen-web/src/components/base-user/` |
| Dashboard | `zen-server/src/features/dashboard/` | `zen-web/src/api/dashboard/`, `zen-web/src/routes/index.tsx` |
| RBAC carriers | `zen-server/src/features/system/menu/`, `system/role/`, access-facing `system/user/` | `zen-web/src/api/system/menu/`, `system/role/`, `system/user/`; `zen-web/src/routes/system/` |
| Audit carrier | `zen-server/src/features/system/log/` | `zen-web/src/api/system/log/`, `zen-web/src/routes/system/log.tsx` |
| System dictionary | `zen-server/src/features/system/dict/` | `zen-web/src/api/system/dict/`, `zen-web/src/routes/system/dict.tsx` |
| Runtime files | `zen-server/src/common/files.rs`, `zen-server/src/infra/config.rs`, `zen-server/src/infra/app.rs` | avatar upload UI under `zen-web/src/components/base-user/` |

## Rules

- Do not create `rbac`, `audit`, `runtime`, or other new top-level feature folders just to match capability names.
- Keep current carriers in place until a real implementation need requires a new owner.
- When backend contracts move, update matching frontend API modules and types in the same task.
