# Repository Rules

## Source of Truth

- Current facts: source code, then [docs/architecture.md](./docs/architecture.md), then [docs/guides/](./docs/guides/).
- Command source: root `justfile`; inspect the target before running it.

## Reading Order

1. Read `README.md`.
2. Read `AGENTS.md`.
3. Read the nearest subdirectory `AGENTS.md`.
4. Read only the relevant guide in `docs/guides/`.
5. Use `docs/reference/` only for deeper context.

## Boundaries

- Shared auth and permission capability code lives in `zen-core/`.
- Backend lives in `zen-server/`.
- Migrations live in `zen-server/migrations/`.
- Frontend lives in `zen-web/`.
- Deployment assets live in `deploy/`.
- Root keeps workspace metadata, docs, command entry points, and the shared crate.

## Working Rules

- Prefer the smallest viable change.
- Do not add fallback or compatibility logic.
- Keep stable rules in `docs/architecture.md` and `docs/guides/`.
- Keep subdirectory `AGENTS.md` files thin.
- Do not use `docs/reference/` or `docs/history/` as default implementation truth.
- SQLite is the default V2 storage backend.
- PostgreSQL-first behavior is archived under `legacy/pg-admin`.
- Update code, docs, and commands together when structure changes.
