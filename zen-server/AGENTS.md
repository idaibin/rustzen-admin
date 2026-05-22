# Backend Rules

## Read

- `docs/guides/backend.md`
- `docs/guides/permission.md`

## Boundaries

- Business features live in `zen-server/src/features/`.
- Shared auth and permission code stays in `zen-core/`.
- Migrations live in `zen-server/migrations/`.

## Rules

- New features use `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs`.
- Handlers only handle requests and responses; do not write SQL there.
- Services handle orchestration, validation, and transactions.
- Repos only handle persistence and must not cross feature boundaries.
- Use `Require(...)` as the default permission check.
- Do not add compatibility fallbacks or extra abstraction layers.

## Command Source

- Use root `justfile` as the command source of truth.
