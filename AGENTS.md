# Repository Guidelines

## Source of Truth

- `README.md`: developer entry point and document map
- `AGENTS.md`: repository-level collaboration rules and reading order
- `zen-server/AGENTS.md`: backend quick-entry guidance inside `zen-server/`
- `zen-web/AGENTS.md`: frontend quick-entry guidance inside `zen-web/`
- `docs/architecture.md`: repository layout, document layers, repo boundaries, and command summary
- `docs/README.md`: documentation system map and placement rules
- `docs/backend-guide.md`: Rust backend layering, file roles, naming, and database rules
- `docs/frontend-guide.md`: React routing, state, API organization, page rules, and UI constraints
- `docs/deployment-guide.md`: production deployment rules, packaging, service startup, and runtime config requirements
- `docs/permission-guide.md`: current permission model and usage constraints
- `docs/project-map.md`: file and entrypoint map for fast orientation

## Reading Order

1. Read `README.md`.
2. Read `AGENTS.md`.
3. Read the nearest subdirectory `AGENTS.md`.
4. Read the relevant guide docs in `docs/`.
5. Read additional docs under `docs/` only when the task needs them.

## Repository Boundaries

- Shared auth and permission capability code lives in `zen-core/`.
- Backend lives in `zen-server/`.
- Migrations live in `zen-server/migrations/`.
- Frontend lives in `zen-web/`.
- Deployment assets live in `deploy/`.
- Root keeps workspace metadata, docs, command entry points, and the shared crate.

## Working Rules

- Prefer the smallest viable change.
- Do not add fallback or compatibility logic.
- Keep stable formal rules in `docs/`.
- Keep subdirectory `AGENTS.md` files thin.
- Update code, docs, and commands together when structure changes.
