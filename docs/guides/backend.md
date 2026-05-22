# Backend Guide

Rules for Rust backend work under `zen-server/`.

## Layers

- `mod.rs`: route assembly only.
- `handler.rs`: request and response handling only; no SQL.
- `service.rs`: orchestration, validation, transactions, and cross-repo coordination.
- `repo.rs`: SQL and persistence only; do not cross feature boundaries.
- `types.rs`: row/entity types, then request, response, query, and command types.

## Rules

- New features use `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs`.
- Reuse auth and permission code from `zen-core/`; do not re-implement it in `zen-server/`.
- Use `PermissionsCheck::Require(...)` by default.
- Use `snake_case` for Rust and database names.
- Use `camelCase` for JSON and frontend-facing fields.
- Prefer `#[serde(rename_all = "camelCase")]` on HTTP request/response structs.
- SQL must be explicit; do not use `SELECT *`.
- Schema changes require migrations.
- Runtime config uses `DATABASE_URL` and `RUSTZEN_*`.
- PostgreSQL is the only runtime database backend.
- Use root `justfile` as the command source of truth.

## Prohibited

- SQL in handlers.
- Services bypassing repos.
- Repos calling across features.
- Generated-file edits.
- Compatibility fallbacks.
- Speculative abstraction.
