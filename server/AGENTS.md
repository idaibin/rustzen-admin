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

- `core/`: shared auth and permission capability crate used by `server/`
- `server/src/features/`: business features
- `server/src/infra/`: infrastructure such as config, database, auth runtime wiring, and menu sync
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

- `cargo check -p server`
- `just dev-server`
- `just check` (backend `cargo check` + frontend `vp lint`)
- `just build-binary` (Docker standalone backend binary export)
- `just build-release` (Docker release tree and zip export)
- `just build-image` (Docker runtime image build)

## Maintenance Rules

- Command descriptions in `server/AGENTS.md`, `docs/*`, and `justfile` must stay aligned. Update them together when command entrypoints change.
