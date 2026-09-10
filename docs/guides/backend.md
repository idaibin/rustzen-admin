# Backend Guide

Rules for Rust backend work under `apps/admin/`, `apps/monitor/`,
`apps/insights/`, and `apps/reports/`.

## Application boundaries

- Admin owns authentication, RBAC, module synchronization and gateway,
  release management, embedded Web assets, and Admin-only features.
- Monitor, Insights, and Reports each own complete feature layers, migrations,
  one SQLite database, and one server process.
- Applications do not call another application's repo or database.
- Shared request contracts and HMAC delegation live in `crates/ipc/`; shared
  auth policy lives in `crates/auth/`.
- Admin has three mutually exclusive source compositions. The default `full`
  feature retains the complete current Admin behavior. The reviewed selected
  compositions are `monitor-distribution` and `analytics-distribution`; each
  retains access, its one selected module gateway and its composition-qualified
  Web artifact. They use `migrations/sqlite-monitor` and
  `migrations/sqlite-analytics` respectively as final fresh-install schemas.
  Validate them with `just verify-monitor-admin` and
  `just verify-analytics-admin`.

## Feature layers

- `mod.rs`: route assembly only.
- `handler.rs`: request and response handling only; no SQL.
- `service.rs`: orchestration, validation, transactions, and feature-local
  coordination.
- `repo.rs`: feature-local SQL and persistence only.
- `types.rs`: row/entity, request, response, query, and command types.

Documented read-only aggregation features may omit `repo.rs` when they own no
persistence; Admin `features/dashboard/` is the current intentional exception.

## Route and Manifest rules

- Admin-native protected routes use capability constants and
  `PermissionsCheck::Require(...)` by default.
- Module routes use `ModuleRouter` helpers. The Rust registration is the only
  declaration of method, path, access mode, handler, and capability.
- `module.toml` contains only fixed module metadata and default menu
  presentation. Do not add route or capability lists to TOML.
- A module Manifest must be generated from the registered Rust router. Do not
  build a second route catalog.
- Preserve public Web API paths and response envelopes when moving ownership.

## Persistence and configuration

- SQL must be explicit; do not use `SELECT *`.
- This initialization template keeps only the final fresh-install schema. Update
  the owning application's initialization migration directly, as required by
  root `AGENTS.md`; do not add upgrade migrations or legacy-data conversion.
  Validate with a newly created database. Existing local databases must not be
  deleted without an explicit user request.
- Use `crates/storage/` for shared SQLite connection and maintenance behavior.
- Use the focused config type from `crates/config/`; a process must not parse
  settings owned only by another process.
- Optional numeric settings remain absent when unset. Explicit zero is a value,
  not an absence sentinel.
- Local development uses safe code defaults. Production secrets and release
  verification settings must be explicit and non-placeholder.
- Scheduled Admin tasks remain bounded by
  `RUSTZEN_TASK_RUN_TIMEOUT_SECONDS`.
- Row-deleting retention work must use the SQLite maintenance plan rather than
  assuming `DELETE` immediately shrinks database or WAL files.

## Gateway hot path

The warm module request path may use only the immutable route registry,
in-memory user capability cache, reused HTTP client, streaming body, and one
request-scoped HMAC delegation. It must not:

- query SQLite for routes, permissions, menus, or module state;
- read TOML;
- fetch or parse a Manifest;
- perform service discovery;
- construct a new HTTP client;
- send a full permission or role set;
- parse and re-serialize a proxied JSON body.

## General rules

- `just check` runs Rust cases with `--test-threads=1`: independent Admin test
  databases currently share the process-global permission cache. Direct full
  workspace tests must use the same flag. Concurrency tests still exercise
  parallel tasks inside each case; serialization is not a concurrency proof.
- Use `snake_case` for Rust and database names.
- Use `camelCase` for frontend-facing JSON, normally through
  `#[serde(rename_all = "camelCase")]`.
- Use the root `justfile` as the command source of truth.
- Keep changes inside the owning application unless the shared contract truly
  has multiple consumers.
- Do not add compatibility fallbacks, registries, plugin systems, launchers, or
  speculative abstraction layers.
