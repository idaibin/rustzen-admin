# Backend Guide

## Scope

- Applies to `server/`.
- Keep only quick local rules for work inside `server/`; do not duplicate the full specification documents.

## Quick Entry

- See `docs/backend-guide.md` for full backend boundaries.
- See `docs/architecture.md` for repository-wide rules.
- See `docs/permission-guide.md` for the permission model.
- See `docs/project-map.md` for directory and entrypoint lookup.

## Directory Highlights

- `server/src/features/`: business features
- `server/src/infra/`: infrastructure such as config, database, JWT, and permissions
- `server/src/common/`: shared capabilities across features
- `server/src/middleware/`: middleware
- `server/migrations/`: database migrations

## Rules

- For a new feature, create `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs` first.
- Handlers only deal with request and response handling; do not write SQL there.
- Services handle orchestration, validation, and transactions.
- Repos only handle persistence and must not cross feature boundaries.
- Use `Require(...)` as the default permission check.
- Do not add compatibility fallbacks or extra abstraction layers.

## Commands

- `cargo check --manifest-path server/Cargo.toml`
- `just dev-server`
- `just check` (backend `cargo check` + frontend `vp lint`)

## Maintenance Rules

- Command descriptions in `server/AGENTS.md`, `docs/*`, and `justfile` must stay aligned. Update them together when command entrypoints change.
