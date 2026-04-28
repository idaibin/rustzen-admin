# Project Map

> This document is only an index. Use it to quickly locate entrypoints, directories, and high-frequency change points.

## Repository Entrypoints

- Shared auth and permission crate entrypoint: `zen-core/src/lib.rs`
- Backend startup entrypoint: `zen-server/src/main.rs`
- Frontend startup entrypoint: `zen-web/src/main.tsx`
- Root route entrypoint: `zen-web/src/routes/__root.tsx`
- Root layout entrypoint: `zen-web/src/components/base-layout/index.tsx`
- Shared command entrypoint: `justfile`

## Documentation Index

- Documentation system entrypoint: `docs/README.md`
- Repository rules and reading order: `AGENTS.md`
- Backend entry rules: `zen-server/AGENTS.md`
- Frontend entry rules: `zen-web/AGENTS.md`
- Product direction: `docs/goals/product-direction.md`
- Repository evolution goals: `docs/goals/repository-evolution.md`
- Current rollout plan: `docs/plans/documentation-governance-rollout.md`
- Current Phase 1 foundation rollout plan: `docs/plans/admin-foundation-phase-1-rollout.md`
- Current documentation governance spec: `docs/specs/documentation-governance.md`
- Current Phase 1 foundation spec: `docs/specs/admin-foundation-phase-1.md`
- Current audit baseline spec: `docs/specs/audit-baseline.md`
- Current identity baseline spec: `docs/specs/identity-baseline.md`
- Current access baseline spec: `docs/specs/access-baseline.md`
- Current system baseline spec: `docs/specs/system-baseline.md`
- Current runtime baseline spec: `docs/specs/runtime-baseline.md`
- Stable agent operating rules: `docs/agents/operating-rules.md`
- Current agent-facing iteration state: `docs/agents/current-iteration.md`
- Repository-wide structure rules: `docs/architecture.md`
- Cross-repository comparison baseline: `docs/repository-comparison.md`
- Backend implementation rules: `docs/backend-guide.md`
- Frontend implementation rules: `docs/frontend-guide.md`
- Deployment rules: `docs/deployment-guide.md`
- Permission model rules: `docs/permission-guide.md`

## Deployment Index

- Standalone Linux x86_64 backend binary build: `deploy/binary.Dockerfile`
- Release tree and zip build: `deploy/release.Dockerfile`
- Runtime image build: `deploy/runtime.Dockerfile`
- Systemd service template: `deploy/rustzen-admin.service`
- One-time repair SQL: `deploy/sql/repair_menu_schema.sql`

## Backend Index

- Shared auth and permission core: `zen-core/src/`
- Auth context, JWT codec, and middleware: `zen-core/src/auth/`
- Permission rules, registry, and route helper: `zen-core/src/permission/`
- Feature registry: `zen-server/src/features/mod.rs`
- Auth: `zen-server/src/features/auth/`
- Dashboard: `zen-server/src/features/dashboard/`
- System management aggregator: `zen-server/src/features/system/mod.rs`
- Users: `zen-server/src/features/system/user/`
- Roles: `zen-server/src/features/system/role/`
- Menus: `zen-server/src/features/system/menu/`
- Dictionaries: `zen-server/src/features/system/dict/`
- Logs: `zen-server/src/features/system/log/`
- Infrastructure: `zen-server/src/infra/`
- Runtime config entrypoint: `zen-server/src/infra/config.rs`
- Service assembly and static paths: `zen-server/src/infra/app.rs`
- System info helpers: `zen-server/src/infra/system_info.rs`
- Shared utilities: `zen-server/src/common/`
- Upload path handling: `zen-server/src/common/files.rs`
- Middleware: `zen-server/src/middleware/`
- Database migrations: `zen-server/migrations/`

## Phase 1 Capability Map

- `identity`: currently starts from `zen-server/src/features/auth/`
- `access`: currently starts from `zen-server/src/features/system/menu/`, `zen-server/src/features/system/role/`, and access-facing parts of `zen-server/src/features/system/user/`
- `audit`: currently starts from `zen-server/src/features/system/log/`
- `system`: currently starts from `zen-server/src/features/system/dict/` and future config ownership
- `runtime`: currently has no dedicated top-level feature and will be introduced as a new group

Current-to-target backend ownership:

- `auth` -> `identity`
- `system/menu` + `system/role` + access-facing parts of `system/user` -> `access`
- `system/log` -> `audit`
- `system/dict` + future config -> `system`
- new file/resource capability -> `runtime`

## Frontend Index

- App bootstrap: `zen-web/src/main.tsx`
- Root route guard and devtools: `zen-web/src/routes/__root.tsx`
- Root layout shell and navigation: `zen-web/src/components/base-layout/index.tsx`
- Route pages: `zen-web/src/routes/`
- API barrel: `zen-web/src/api/index.ts`
- Shared request layer: `zen-web/src/api/request.ts`
- API modules: `zen-web/src/api/<domain>/api.ts` and `zen-web/src/api/system/<resource>/api.ts`
- Frontend rules: `docs/frontend-guide.md`
- Shared components: `zen-web/src/components/`
- Shared state: `zen-web/src/store/`
- Utilities: `zen-web/src/util/`
- Global styles: `zen-web/src/style.css`

## High-Frequency Change Points

- Add a backend endpoint: `zen-server/src/features/<feature>/`
- Change route permission wiring: `zen-core/src/permission/route.rs`
- Change auth context or JWT behavior: `zen-core/src/auth/`
- Change permission cache or menu sync: `zen-server/src/infra/permission.rs`
- Add a frontend page: `zen-web/src/routes/`
- Change frontend requests: `zen-web/src/api/request.ts` and `zen-web/src/api/<module>/api.ts`
- Change frontend bootstrap and providers: `zen-web/src/main.tsx`
- Change root route guard or devtools: `zen-web/src/routes/__root.tsx`
- Change frontend layout: `zen-web/src/components/base-layout/index.tsx`
- Change auth state: `zen-web/src/store/useAuthStore.ts`
- Change deployment packaging or service files: `deploy/`
- Change product direction or repository intent: `docs/goals/`
- Change active rollout sequencing: `docs/plans/`
- Change structural design contracts: `docs/specs/`
- Change agent execution rules or current iteration state: `docs/agents/`

## Common Commands

- `just dev-server`
- `just dev-web`
- `just check`
- `just build`
- `just build-binary`
- `just build-release`
- `just build-image`
