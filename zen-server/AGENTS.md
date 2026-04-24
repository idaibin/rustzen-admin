# Backend Guide

## Scope

- Applies to `zen-server/`.
- Keep only quick local rules for work inside `zen-server/`.

## Quick Entry

- `docs/backend-guide.md`
- `docs/architecture.md`
- `docs/permission-guide.md`
- `docs/project-map.md`

## Directory Highlights

- `zen-core/`: shared auth and permission capability crate used by `zen-server/`
- `zen-server/src/features/`: business features
- `zen-server/src/infra/`: infrastructure such as config, database, auth runtime wiring, and menu sync
- `zen-server/src/common/`: shared capabilities across features
- `zen-server/src/middleware/`: middleware
- `zen-server/migrations/`: database migrations

## Local Rules

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
