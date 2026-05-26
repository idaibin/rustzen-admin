# Crate Rules

## Read

- `docs/guides/backend.md`
- `docs/guides/deployment.md`
- `docs/guides/ai-coding-rules.md`

## Boundaries

- SQLite connection helpers and migration entry points for backend runtime.
- No cross-database abstraction layer for first-phase implementation.

## Rules

- Keep connection APIs concrete (`SqlitePool` family).
- Keep migration invocation explicit and deterministic.
- Keep APIs minimal and avoid speculative wrappers.
