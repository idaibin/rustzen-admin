# Project Map

> This document is only an index. Use it to quickly locate entrypoints, directories, and high-frequency change points.

## Repository Entrypoints

- Backend startup entrypoint: `server/src/main.rs`
- Frontend startup entrypoint: `web/src/main.tsx`
- Root route entrypoint: `web/src/routes/__root.tsx`
- Root layout entrypoint: `web/src/components/base-layout/index.tsx`
- Shared command entrypoint: `justfile`

## Deployment Index

- Standalone Linux x86_64 backend binary build: `deploy/binary.Dockerfile`
- Release tree and zip build: `deploy/release.Dockerfile`
- Runtime image build: `deploy/runtime.Dockerfile`
- Systemd service template: `deploy/rustzen-admin.service`
- One-time repair SQL: `deploy/sql/repair_menu_schema.sql`

## Backend Index

- Feature registry: `server/src/features/mod.rs`
- Auth: `server/src/features/auth/`
- Dashboard: `server/src/features/dashboard/`
- System management aggregator: `server/src/features/system/mod.rs`
- Users: `server/src/features/system/user/`
- Roles: `server/src/features/system/role/`
- Menus: `server/src/features/system/menu/`
- Dictionaries: `server/src/features/system/dict/`
- Logs: `server/src/features/system/log/`
- Infrastructure: `server/src/infra/`
- Runtime config entrypoint: `server/src/infra/config.rs`
- Service assembly and static paths: `server/src/infra/app.rs`
- System info helpers: `server/src/infra/system_info.rs`
- Shared utilities: `server/src/common/`
- Upload path handling: `server/src/common/files.rs`
- Middleware: `server/src/middleware/`
- Database migrations: `server/migrations/`

## Frontend Index

- App bootstrap: `web/src/main.tsx`
- Root route guard and devtools: `web/src/routes/__root.tsx`
- Route pages: `web/src/routes/`
- Shared layout component: `web/src/components/base-layout/`
- API barrel: `web/src/api/index.ts`
- System API barrel: `web/src/api/system/index.ts`
- Shared API runtime refs: `web/src/api/runtime.ts`
- Shared request layer: `web/src/api/request.ts`
- API wrappers and modules: `web/src/api/index.ts`, `web/src/api/system/index.ts`, `web/src/api/runtime.ts`, `web/src/api/request.ts`, `web/src/api/<module>/api.ts`
- API module layout, naming, `apiRequest` default vs `raw`, and `list` return conventions: see **API Module Rules** in `docs/frontend-guide.md`
- Shared components: `web/src/components/` (`base-auth/`, `base-button/`, `base-layout/`, `base-user/`, …)
- Shared state: `web/src/store/`
- Utilities: `web/src/util/`
- Global styles: `web/src/style.css`

## High-Frequency Change Points

- Add a backend endpoint: `server/src/features/<feature>/`
- Change permission checks: `server/src/common/router_ext.rs`
- Change permission model: `server/src/infra/permission.rs`
- Add a frontend page: `web/src/routes/`
- Change frontend requests: `web/src/api/request.ts` and `web/src/api/<module>/api.ts`
- Change frontend bootstrap and providers: `web/src/main.tsx`
- Change root route guard or devtools: `web/src/routes/__root.tsx`
- Change frontend layout: `web/src/components/base-layout/index.tsx`
- Change auth state: `web/src/store/useAuthStore.ts`
- Change deployment packaging or service files: `deploy/`

## Common Commands

- `just dev-server`
- `just dev-web`
- `just check`
- `just build`
- `just build-binary`
- `just build-release`
- `just build-image`
