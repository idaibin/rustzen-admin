# Architecture

> This is the single repository-wide specification. It defines repository layout, document layers, monorepo boundaries, change sync rules, and command entrypoints.

## Repository Layout

- The root keeps workspace metadata, shared command entrypoints, repository-level documents, and the shared `zen-core/` crate.
- Shared auth and permission capabilities live in `zen-core/`.
- The backend lives in `zen-server/`.
- The frontend lives in `zen-web/`.
- Deployment assets live in `deploy/`.
- Database migrations live in `zen-server/migrations/`.
- `docs/` contains the repository's formal documentation system, including guides, goals, plans, specs, and agent-facing documents.

Source vs runtime: frontend **source** lives under `zen-web/`. After packaging, the backend serves static files from `<runtime_root>/web/dist` (see `docs/deployment-guide.md`); that `web/dist` path is the deploy layout, not the monorepo folder name.

Current layout:

```txt
.
├── Cargo.toml
├── Cargo.lock
├── justfile
├── zen-core/
│   ├── Cargo.toml
│   └── src/
│       ├── auth/
│       ├── permission/
│       ├── error.rs
│       └── lib.rs
├── zen-server/
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
├── zen-web/
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
├── deploy/
│   ├── sql/
│   │   └── repair_menu_schema.sql
│   ├── binary.Dockerfile
│   ├── release.Dockerfile
│   ├── runtime.Dockerfile
│   └── rustzen-admin.service
└── docs/
```

## Document Layers

- Root documents: `README.md` and `AGENTS.md`
- Subdirectory entry documents: `zen-server/AGENTS.md` and `zen-web/AGENTS.md`
- `docs/` formal documents:
    - `docs/README.md`: documentation system map and placement rules
    - `docs/goals/`: product direction and long-lived repository goals
    - `docs/plans/`: active and upcoming work planning
    - `docs/specs/`: formal design and structure specifications
    - `docs/agents/`: stable operating rules and current agent iteration state
    - `docs/architecture.md`: repository-wide rules
    - `docs/repository-comparison.md`: cross-repository baseline for future optimization decisions
    - `docs/backend-guide.md`: backend rules
    - `docs/frontend-guide.md`: frontend rules
    - `docs/deployment-guide.md`: production layout, build artifacts, and runtime config
    - `docs/permission-guide.md`: permission model and usage constraints
    - `docs/project-map.md`: entrypoint and high-frequency path index

## Directory Responsibilities

- `zen-core/src/auth/`: shared JWT, auth context, extractor, and auth middleware
- `zen-core/src/permission/`: shared permission checks, registry, and route helpers
- `zen-server/src/features/`: backend business features
- `zen-server/src/infra/`: infrastructure such as config, database, password, auth runtime wiring, and menu sync
- `zen-server/src/common/`: shared cross-feature capabilities
- `zen-server/src/middleware/`: Axum middleware
- `zen-web/src/routes/`: pages and route entrypoints
- `zen-web/src/api/`: frontend barrel exports, request wrappers, API constants, option lists, and API types
- `zen-web/src/components/`: shared frontend components; each uses a `base-<name>/` subdirectory (for example `base-auth/`, `base-button/`, `base-layout/`, `base-user/`)
- `zen-web/src/components/base-layout/`: frontend admin shell
- `zen-web/src/store/`: shared frontend state
- `deploy/`: deployment assets, the binary/release/runtime Dockerfiles, `deploy/sql/` repair scripts, and the systemd service template

## Repository Boundaries

- Do not add parallel app directories or move the existing primary directories without an explicit repository change.
- Backend business code belongs in `zen-server/src/features/<feature>/`.
- Frontend pages belong in `zen-web/src/routes/`, request wrappers belong in `zen-web/src/api/request.ts`, and API barrel exports belong in `zen-web/src/api/index.ts` and `zen-web/src/api/system/index.ts`.
- Generated files must stay in generated paths and out of manually maintained paths.

## Change Sync

- When an API contract changes, update the backend implementation, frontend calls, frontend types, and docs together.
- When the database schema changes, update migrations, backend queries, and related API docs together.
- When the directory structure changes, update `README.md`, `AGENTS.md`, `docs/architecture.md`, `docs/project-map.md`, and `zen-server/AGENTS.md` / `zen-web/AGENTS.md` together.
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
just build-binary # export the Ubuntu x86_64 backend binary from Docker
just build-release # export the Ubuntu x86_64 release tree and zip from Docker
just build-image # build the Ubuntu x86_64 runtime image from Docker
```
