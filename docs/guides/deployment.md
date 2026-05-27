# Deployment Guide

Current deployment rules.

## Runtime Layout

```txt
<runtime_root>/
├── bin/
├── config/
├── data/
│   ├── uploads/
│   └── avatars/
├── logs/
├── systemd/
└── web/
    └── dist/
```

## Rules

- Use one deploy root for the whole app.
- Production runs with `WorkingDirectory=/opt/rustzen-admin`.
- Production sets `RUSTZEN_RUNTIME_ROOT=.`.
- Runtime config comes from `RUSTZEN_STORAGE`, `RUSTZEN_SQLITE_PATH`, and `RUSTZEN_*`.
- Production must provide `RUSTZEN_STORAGE`, `RUSTZEN_SQLITE_PATH`, and `RUSTZEN_JWT_SECRET`.
- `config/app.env` is only an environment-variable carrier.
- Backend static files are served from `<runtime_root>/web/dist`.
- Uploads live under `<runtime_root>/data/uploads`.
- Avatars live under `<runtime_root>/data/avatars`.
- Logs live under `<runtime_root>/logs`.
- Build and deploy targets are defined in the root `justfile`.
- Frontend release builds use pnpm with `apps/web/pnpm-lock.yaml`.
- `deploy/sql/` contains one-off SQL repair scripts for older deployments; these are not migration files and should not be used for schema evolution.

The sqlite-first phase uses SQLite by default and does not require PostgreSQL for local startup.

## Prohibited

- Parallel runtime config files such as `system.yaml`.
- Fallback deploy layouts.
- Build-machine absolute paths in runtime behavior.
- Nested deployment asset directories unless the deployment surface grows.
- npm or Bun lockfiles for the `apps/web` release build.

## Local startup

```bash
cargo run -p server
```

## Checks

- `bin/rustzen-admin` exists.
- `web/dist/index.html` exists.
- `config/app.env` exists.
- Runtime data and log directories are writable.
- `systemd/rustzen-admin.service` exists in release packages.
