# Project Map

This is a practical path index for task orientation. It maps where to look first and when a path matters.

## Root

| Path | Value | Inspect when |
| --- | --- | --- |
| `README.md` | Human entrypoint and project summary. | You need a quick repository overview. |
| `AGENTS.md` | Repository-wide AI constraints. | You start any task. |
| `Cargo.toml` | Rust workspace definition. | You need crate or dependency context. |
| `justfile` | Command source of truth. | You need to run, check, build, or package. |
| `.env.example` | Environment-variable template. | You touch runtime config or deployment. |

## Documentation

| Path | Value | Inspect when |
| --- | --- | --- |
| `docs/README.md` | Documentation index with file roles and values. | You need to choose which docs to read. |
| `docs/architecture.md` | Current repository facts. | You need boundaries, topology, or data flow. |
| `docs/project-map.md` | Path orientation. | You need to find the right files. |
| `docs/guides/` | Current development rules. | You are about to edit backend, frontend, deployment, or permission behavior. |
| `docs/reference/` | Optional deep context. | Current facts and guides are not enough. |
| `docs/history/` | Non-current records. | You need completed designs or historical context. |

## Backend

| Path | Value | Inspect when |
| --- | --- | --- |
| `crates/auth/` | Shared auth and permission crate. | You touch auth context, JWT, extractors, or permission checks. |
| `crates/storage/` | Shared SQLite storage helpers and migration invocations. | You touch DB bootstrap, connection helpers, or migration wiring. |
| `apps/server/AGENTS.md` | Backend-specific AI rules. | You work under `apps/server/`. |
| `apps/server/src/infra/` | Config, app assembly, database, auth runtime, permission cache, and menu sync. | You touch startup, runtime paths, DB wiring, static serving, or permission sync. |
| `apps/server/src/common/` | Cross-feature backend helpers. | You touch shared file or utility behavior. |
| `apps/server/src/middleware/` | Axum middleware. | You touch request middleware behavior. |
| `apps/server/migrations/` | SQL migrations. | You change schema. |

## Backend Features

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/server/src/features/auth/` | Login, logout, and current-session bootstrap. | You touch session, token, login info, or logout behavior. |
| `apps/server/src/features/account/` | Current-account profile, avatar, and password flows. | You touch self-service account behavior. |
| `apps/server/src/features/dashboard/` | Dashboard summary APIs. | You touch dashboard cards or summary stats. |
| `apps/server/src/features/system/dict/` | Dictionary management. | You touch dictionary data or option sources. |
| `apps/server/src/features/system/log/` | System log management and current audit carrier. | You touch operation or login logs. |
| `apps/server/src/features/system/menu/` | Menu and permission menu management. | You touch menu trees or permission-code menu rows. |
| `apps/server/src/features/system/role/` | Role management. | You touch roles or role-menu assignment. |
| `apps/server/src/features/system/user/` | User management and access-facing user-role behavior. | You touch admin user CRUD, status, password reset, or user-role assignment. |

## Frontend

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/web/AGENTS.md` | Frontend-specific AI rules. | You work under `apps/web/`. |
| `apps/web/package.json` | Frontend package and pnpm scripts. | You touch frontend dependencies or scripts. |
| `apps/web/pnpm-lock.yaml` | Frontend pnpm lockfile. | You change frontend dependencies or package-manager behavior. |
| `apps/web/pnpm-workspace.yaml` | Frontend pnpm build-script approvals. | You change pnpm install behavior or packages with install scripts. |
| `apps/web/src/routes/` | File-based route pages and root guard. | You add or change pages, redirects, auth gates, or error routes. |
| `apps/web/src/api/` | Request wrapper, API modules, and frontend API types. | You change backend contracts or page data access. |
| `apps/web/src/components/base-layout/` | Admin shell, navigation, and layout concerns. | You touch menus, header, shell, or logout UI. |
| `apps/web/src/components/base-user/` | Current-user UI such as avatar behavior. | You touch profile/avatar visible behavior. |
| `apps/web/src/store/` | Shared frontend state. | You touch auth state or persisted cross-page state. |
| `apps/web/src/constant/` | Frontend constants and option lists. | You touch static frontend options. |
| `apps/web/src/util/` | Frontend utilities. | You touch shared frontend helpers. |

## Deployment

| Path | Value | Inspect when |
| --- | --- | --- |
| `deploy/` | Dockerfiles and service template. | You package or deploy the app. |
| `deploy/sql/` | One-off SQL repair scripts. | You repair an older deployment. |
