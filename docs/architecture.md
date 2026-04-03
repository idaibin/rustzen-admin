# Architecture

> This is the single repository-wide specification. It defines repository layout, document layers, monorepo boundaries, change sync rules, and command entrypoints.

## Repository Layout

- The root only keeps workspace metadata, shared command entrypoints, and repository-level documents.
- The backend lives in `server/`.
- The frontend lives in `web/`.
- Database migrations live in `server/migrations/`.
- `docs/` only contains single-responsibility specification documents.

Current layout:

```txt
.
├── Cargo.toml
├── Cargo.lock
├── pnpm-workspace.yaml
├── justfile
├── server/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
│       ├── common/
│       ├── features/
│       │   └── <feature>/
│       │       ├── mod.rs
│       │       ├── handler.rs
│       │       ├── service.rs
│       │       ├── repo.rs
│       │       └── types.rs
│       ├── infra/
│       └── middleware/
├── web/
│   └── src/
│       ├── api/
│       │   ├── index.ts
│       │   ├── runtime.ts
│       │   ├── request.ts
│       │   ├── api.d.ts
│       │   ├── system/
│       │   │   ├── index.ts
│       │   │   └── <module>/
│       │   │       ├── api.ts
│       │   │       └── types.d.ts
│       │   └── <module>/
│       │       ├── api.ts
│       │       └── types.d.ts
│       ├── components/
│       │   ├── base-auth/
│       │   ├── base-button/
│       │   ├── base-layout/
│       │   └── base-user/
│       ├── routes/
│       ├── store/
│       ├── util/
│       └── style.css
└── docs/
```

## Document Layers

- Root documents: `README.md` and `AGENTS.md`
- Subdirectory entry documents: `server/AGENTS.md` and `web/AGENTS.md`
- `docs/` specification documents:
    - `docs/architecture.md`: repository-wide rules
    - `docs/backend-guide.md`: backend rules
    - `docs/frontend-guide.md`: frontend rules
    - `docs/deployment-guide.md`: production layout, build artifacts, and runtime config
    - `docs/permission-guide.md`: permission model and usage constraints
    - `docs/project-map.md`: entrypoint and high-frequency path index

## Directory Responsibilities

- `server/src/features/`: backend business features
- `server/src/infra/`: infrastructure such as config, database, JWT, password, and permissions
- `server/src/common/`: shared cross-feature capabilities
- `server/src/middleware/`: Axum middleware
- `web/src/routes/`: pages and route entrypoints
- `web/src/api/`: frontend barrel exports, request wrappers, API constants, option lists, and API types
- `web/src/components/`: shared frontend components; each uses a `base-<name>/` subdirectory (for example `base-auth/`, `base-button/`, `base-layout/`, `base-user/`)
- `web/src/components/base-layout/`: frontend admin shell
- `web/src/store/`: shared frontend state

## Repository Boundaries

- Do not add parallel app directories or move the existing primary directories without an explicit repository change.
- Backend business code belongs in `server/src/features/<feature>/`.
- Frontend pages belong in `web/src/routes/`, request wrappers belong in `web/src/api/request.ts`, and API barrel exports belong in `web/src/api/index.ts` and `web/src/api/system/index.ts`.
- Generated files must stay in generated paths and out of manually maintained paths.

## Change Sync

- When an API contract changes, update the backend implementation, frontend calls, frontend types, and docs together.
- When the database schema changes, update migrations, backend queries, and related API docs together.
- When the directory structure changes, update `README.md`, `AGENTS.md`, `docs/architecture.md`, `docs/project-map.md`, and relevant subdirectory entry docs together.
- When adding a feature, create the expected layer files together and avoid temporary scattered implementations.

## Working Style

- Prefer the smallest possible change.
- Reuse existing implementation before introducing new code paths.
- Do not add fallback branches for old paths, old parameters, or old structures.
- Do not add abstraction in advance for hypothetical future use.
- Change source definitions first, then callers, then run unified verification.

## Prohibited

- Do not edit generated files manually.
- Do not put business code in the repository root.
- Do not scatter SQL, HTTP requests, or page styles into the wrong layer.
- Do not keep multiple parallel implementations alive long term.

## Command Entrypoints

```bash
just dev-server # start the backend only
just dev-web    # start the frontend only
just check      # backend check + frontend vp lint
just build      # build backend and frontend
```
