# Backend Guide

## Scope

- Applies to all Rust backend implementation under `server/`.
- Defines backend implementation rules only. Do not repeat repository-wide or frontend rules here.

## Layout

- See `docs/architecture.md` for the repository tree layout.
- Shared auth and permission capabilities live in `core/`; `server/` only wires them to runtime config, cache, and database-backed menu sync.
- For a new feature, create these five files first.
- In `types.rs`, put row and entity types first, then request, response, and query types.
- `mod.rs` exports and wires feature routes only; it must not carry business implementation.

## Layering

- `mod.rs`: route assembly only.
- `handler.rs`: request and response handling only; no SQL.
- `service.rs`: business orchestration, validation, and transactions.
- `repo.rs`: SQL and persistence only.
- `types.rs`: all local types live here.

## Naming

- Use `snake_case` for Rust and database names.
- Use `camelCase` for JSON and frontend-facing fields.
- Handler names should follow endpoint intent: `list_users`, `get_user`, `create_user`, `update_user`, `delete_user`.
- Do not add redundant `_handler` suffixes; handler functions already live in `handler.rs`.
- Repo names should follow data intent: `find_by_id`, `insert`, `update`, `delete_by_id`.
- Prefer `#[serde(rename_all = "camelCase")]` on response types to keep output format consistent.

## Constraints

- Use `Require(...)` as the default permission check.
- Use `Any(...)` or `All(...)` only when a feature explicitly needs it.
- Do not re-implement JWT, auth extractor, or route permission helpers inside `server/`; reuse `core/`.
- Build the smallest implementation that solves the current requirement.
- Handlers must not touch the database.
- Services must not bypass repos.
- Repos must not cross feature boundaries.
- When database shape already matches the API response closely, prefer a `sqlx::FromRow` type in `types.rs` instead of splitting out `model.rs`.

## Configuration

- Application runtime config uses `RUSTZEN_*` environment variables.
- Database connections use `DATABASE_URL`.
- Both development and production read the same runtime keys from environment variables.
- Use a single `RUSTZEN_RUNTIME_ROOT` to derive runtime directories such as `web/dist`, `data/`, and `logs/`.
- `config/app.env` is only the environment-variable carrier, not a second config system.
- Do not maintain a parallel yaml primary config for runtime paths, database connections, JWT, or other application runtime settings.
- Only complex structured rules that are not process-level config may live in separate json or yaml files.
- The backend provides safe defaults for local development runtime keys, including bind address, port, DB pool settings, JWT expiration, runtime root, file prefix, and log settings.
- Production deployments must provide `DATABASE_URL` and `RUSTZEN_JWT_SECRET`; other `RUSTZEN_*` keys have defaults but may still be set explicitly in `config/app.env`.
- `RUSTZEN_JWT_SECRET` has no code default and must be set explicitly.
- Production runtime config must remain explicit for deployment-specific values and secrets.
- PostgreSQL is the only supported runtime database backend for this project.
- Deployment-specific path values and environment file layout belong in `docs/deployment-guide.md`, not here.

## Database

- SQL must be explicit. Do not use `SELECT *`.
- Every schema change must come with a migration.
- Migration files must use a clear prefix such as `0101_...sql`.
- Default audit fields:
    - Main tables: `created_at / updated_at / created_by / updated_by`
    - Soft delete: add `deleted_at / deleted_by` only when needed
    - Join tables: keep only `created_at` by default
    - Log tables: keep only `created_at` or `occurred_at` by default
- `deleted_at / deleted_by` only represent soft deletion.

## Errors

- Errors must be explicit; do not swallow them.
- Do not add speculative fallback branches.
- Do not treat "return a default when not found" as a normal branch.

## Prohibited

- Do not write SQL in handlers.
- Do not call across features in repos.
- Do not bypass repos from services.
- Do not edit generated outputs manually.
- Do not add compatibility fallbacks.
- Do not stack extra abstraction.

## Checks

- Run `cargo check -p server` after backend changes.
