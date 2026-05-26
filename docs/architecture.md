# Architecture

`rustzen-admin` is a Rust full-stack admin foundation. The repository is a monorepo with one shared Rust capability crate, one backend service, one React frontend, and deployment assets.

## Module Boundaries

- `zen-core/` owns shared auth and permission capability code.
- `zen-server/` owns the Axum backend runtime and business features.
- `zen-server/migrations/` owns SQL migrations.
- `zen-web/` owns the React frontend.

SQLite is the default V2 storage backend. PostgreSQL-first behavior is archived under `legacy/pg-admin`.
- `deploy/` owns deployment assets.
- `docs/` owns repository documentation.

Backend feature code lives under `zen-server/src/features/<feature>/` with `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs` unless the feature is intentionally smaller.

Frontend pages live under `zen-web/src/routes/`. Frontend API modules live under `zen-web/src/api/`.

## Runtime Topology

Local development runs backend and frontend separately:

- backend and frontend development targets are defined in the root `justfile`.
- frontend dev traffic calls backend APIs through the frontend dev server configuration.

Packaged deployment runs the backend as the serving process:

- backend binary: `<runtime_root>/bin/rustzen-admin`
- frontend static files: `<runtime_root>/web/dist`
- uploads: `<runtime_root>/data/uploads`
- avatars: `<runtime_root>/data/avatars`
- logs: `<runtime_root>/logs`
- process env: `<runtime_root>/config/app.env`

`RUSTZEN_RUNTIME_ROOT` is the single runtime root. Production uses `.` from the deploy root. Local development defaults to `.rustzen-admin`.

## Data Flow

- Browser pages call typed frontend API modules from `zen-web/src/api/`.
- Frontend API modules call backend HTTP endpoints through `zen-web/src/api/request.ts`.
- Backend handlers parse requests and return responses.
- Backend services coordinate validation, transactions, permission-aware behavior, and repo calls.
- Backend repos run SQL against SQLite by default.
- Static files and uploaded resources are served from paths derived from `RUSTZEN_RUNTIME_ROOT`.

## Permissions

- JWT, auth context extraction, permission checks, and route permission registration live in `zen-core/`.
- Backend permission cache and menu synchronization live in `zen-server/src/infra/`.
- New protected backend routes use `PermissionsCheck::Require(...)` by default.
- `*` is the only full-authorization grant.

## Change Sync

- API contract changes update backend types, frontend API modules, frontend types, and relevant docs together.
- Schema changes update migrations, SQL queries, API types, and relevant docs together.
- Structure changes update `README.md`, `AGENTS.md`, `docs/README.md`, this file, and `docs/project-map.md` together.

## Command Source

- Use root `justfile` as the command source of truth; inspect the relevant target before running it.
