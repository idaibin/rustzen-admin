# Workspace Root Implementation

This reference explains runtime-root behavior. The source of truth is `zen-server/src/infra/config.rs`.

## Current Behavior

- `RUSTZEN_RUNTIME_ROOT` is the single runtime root.
- Local development defaults to `.rustzen-admin`.
- Production deployment sets `RUSTZEN_RUNTIME_ROOT=.` from the deploy root.
- `RUSTZEN_FILES_PREFIX` defaults to `/resources`.

## Derived Paths

From `RUSTZEN_RUNTIME_ROOT`, the backend derives:

- `web/dist`
- `data/`
- `data/uploads`
- `data/avatars`
- `logs`

Avatar public URLs use:

- `RUSTZEN_FILES_PREFIX` for uploads
- `<RUSTZEN_FILES_PREFIX>/avatars` for avatars

## Serving

`zen-server/src/infra/app.rs` mounts:

- upload files at `RUSTZEN_FILES_PREFIX`
- avatar files at `<RUSTZEN_FILES_PREFIX>/avatars`
- frontend static files from `web/dist`

The backend must not depend on build-machine absolute paths.

## Tests

`zen-server/src/infra/config.rs` includes unit tests for:

- local default runtime root
- derived runtime paths
- avatar public prefix
