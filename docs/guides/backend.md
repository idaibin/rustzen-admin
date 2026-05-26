# Backend Guide

Rules for Rust backend work under `apps/server/`.

## Layers

- `mod.rs`: route assembly only.
- `handler.rs`: request and response handling only; no SQL.
- `service.rs`: orchestration, validation, transactions, and cross-repo coordination.
- `repo.rs`: SQL and persistence only; do not cross feature boundaries.
- `types.rs`: row/entity types, then request, response, query, and command types.

## Rules

- New features use `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs`.
- Reuse auth and permission code from `crates/auth/`; do not re-implement it in `apps/server/`.
- Use storage helpers from `crates/storage/` for SQLite connection and migration calls.
- Use `PermissionsCheck::Require(...)` by default.
- Use `snake_case` for Rust and database names.
- Use `camelCase` for JSON and frontend-facing fields.
- Prefer `#[serde(rename_all = "camelCase")]` on HTTP request/response structs.
- SQL must be explicit; do not use `SELECT *`.
- Schema changes require migrations.
- Runtime config uses `RUSTZEN_STORAGE`, `RUSTZEN_SQLITE_PATH`, and `RUSTZEN_*`.
- SQLite is the default runtime storage backend.
- PostgreSQL compatibility is not part of this V2 first-phase implementation.
- Use root `justfile` as the command source of truth.

## Prohibited

- SQL in handlers.
- Services bypassing repos.
- Repos calling across features.
- Generated-file edits.
- Compatibility fallbacks.
- Speculative abstraction.
