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
| `zen-core/` | Shared auth and permission crate. | You touch auth context, JWT, extractors, or permission checks. |
| `zen-server/AGENTS.md` | Backend-specific AI rules. | You work under `zen-server/`. |
| `zen-server/src/infra/` | Config, app assembly, database, auth runtime, permission cache, and menu sync. | You touch startup, runtime paths, DB wiring, static serving, or permission sync. |
| `zen-server/src/common/` | Cross-feature backend helpers. | You touch shared file or utility behavior. |
| `zen-server/src/middleware/` | Axum middleware. | You touch request middleware behavior. |
| `zen-server/migrations/` | SQL migrations. | You change schema. |

## Backend Features

| Path | Value | Inspect when |
| --- | --- | --- |
| `zen-server/src/features/auth/` | Login, logout, and current-session bootstrap. | You touch session, token, login info, or logout behavior. |
| `zen-server/src/features/account/` | Current-account profile, avatar, and password flows. | You touch self-service account behavior. |
| `zen-server/src/features/dashboard/` | Dashboard summary APIs. | You touch dashboard cards or summary stats. |
| `zen-server/src/features/system/dict/` | Dictionary management. | You touch dictionary data or option sources. |
| `zen-server/src/features/system/log/` | System log management and current audit carrier. | You touch operation or login logs. |
| `zen-server/src/features/system/menu/` | Menu and permission menu management. | You touch menu trees or permission-code menu rows. |
| `zen-server/src/features/system/role/` | Role management. | You touch roles or role-menu assignment. |
| `zen-server/src/features/system/user/` | User management and access-facing user-role behavior. | You touch admin user CRUD, status, password reset, or user-role assignment. |

## Frontend

| Path | Value | Inspect when |
| --- | --- | --- |
| `zen-web/AGENTS.md` | Frontend-specific AI rules. | You work under `zen-web/`. |
| `zen-web/package.json` | Frontend package and scripts. | You touch frontend dependencies or scripts. |
| `zen-web/src/routes/` | File-based route pages and root guard. | You add or change pages, redirects, auth gates, or error routes. |
| `zen-web/src/api/` | Request wrapper, API modules, and frontend API types. | You change backend contracts or page data access. |
| `zen-web/src/components/base-layout/` | Admin shell, navigation, and layout concerns. | You touch menus, header, shell, or logout UI. |
| `zen-web/src/components/base-user/` | Current-user UI such as avatar behavior. | You touch profile/avatar visible behavior. |
| `zen-web/src/store/` | Shared frontend state. | You touch auth state or persisted cross-page state. |
| `zen-web/src/constant/` | Frontend constants and option lists. | You touch static frontend options. |
| `zen-web/src/util/` | Frontend utilities. | You touch shared frontend helpers. |

## Deployment

| Path | Value | Inspect when |
| --- | --- | --- |
| `deploy/` | Dockerfiles and service template. | You package or deploy the app. |
| `deploy/sql/` | One-off SQL repair scripts. | You repair an older deployment. |
