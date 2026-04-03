# Project Map

> This document is only an index. Use it to quickly locate entrypoints, directories, and high-frequency change points.

## Repository Entrypoints

- Backend startup entrypoint: `server/src/main.rs`
- Frontend startup entrypoint: `web/src/main.tsx`
- Root route entrypoint: `web/src/routes/__root.tsx`
- Root layout entrypoint: `web/src/components/base-layout/index.tsx`
- Shared command entrypoint: `justfile`

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
- API wrappers: `web/src/api/`
- Shared components: `web/src/components/`
- Shared state: `web/src/stores/`
- Constants: `web/src/constant/`
- Utilities: `web/src/util/`
- Global styles: `web/src/style.css`

## High-Frequency Change Points

- Add a backend endpoint: `server/src/features/<feature>/`
- Change permission checks: `server/src/common/router_ext.rs`
- Change permission model: `server/src/infra/permission.rs`
- Add a frontend page: `web/src/routes/`
- Change frontend requests: `web/src/api/<module>/`
- Change frontend bootstrap and providers: `web/src/main.tsx`
- Change root route guard or devtools: `web/src/routes/__root.tsx`
- Change frontend layout: `web/src/components/base-layout/index.tsx`
- Change auth state: `web/src/stores/useAuthStore.ts`

## Common Commands

- `just dev-server`
- `just dev-web`
- `just check`
- `just build`
