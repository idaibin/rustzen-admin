# Workspace Root Implementation

This reference explains runtime-root behavior. The source of truth is
`crates/runtime/src/lib.rs`, the focused files under `crates/config/src/`, and
the Admin mounting code in `apps/admin/src/infra/app.rs`.

## Current Behavior

- `RUSTZEN_RUNTIME_ROOT` is the single runtime root.
- Local development defaults to `.rustzen-admin`.
- Production deployment sets `RUSTZEN_RUNTIME_ROOT=.` from the deploy root.
- The public file prefix is fixed in code as `/resources`.
- Other runtime paths, ports, pool limits, logging, timezone, retention, and
  task timeout also have built-in defaults and do not belong in the minimal
  production environment file.

## Derived Paths

From `RUSTZEN_RUNTIME_ROOT`, the backend derives:

- `web/dist`
- `data/`
- `data/db/admin.db`
- `data/db/monitor.db`
- `data/db/insights.db`
- `data/reports/db/reports.db`
- `data/reports`
- `data/uploads`
- `data/avatars`
- `logs`

Avatar public URLs use:

- `/resources` for uploads
- `/resources/avatars` for avatars

## Serving

`apps/admin/src/infra/app.rs` mounts:

- upload files at the fixed `/resources` prefix
- avatar files at `/resources/avatars`
- frontend static files from `web/dist`

Each application resolves its own default database path through its focused
configuration type. The Admin update worker also derives all four database
paths from the same runtime root for one release backup and rollback boundary.
No runtime process may depend on build-machine absolute paths.

## Tests

`crates/config/src/*.rs` and `crates/runtime/src/lib.rs` include unit tests for:

- local default runtime root
- derived runtime paths
- all four default database paths
- avatar public prefix
- absent optional numeric values and explicit zero values
