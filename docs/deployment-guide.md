# Deployment Guidelines

> This document defines production deployment layout, release flow, and runtime configuration sources.

## Scope

- Use one deploy root for the whole app.
- Keep backend binary, frontend bundle, config, data, logs, and versions under the same root.
- Do not add extra compatibility paths or fallback layouts.
- Keep repository deployment assets flat under `deploy/` except `deploy/sql/` for one-off SQL scripts; do not add nested `docker/` or `systemd/` directories unless the deployment surface actually becomes complex.

## Naming Convention

- `binary` means the standalone Linux x86_64 backend executable.
- `release` means the deployable directory tree plus its zip archive.
- `runtime` means the Docker image that runs the full application.
- All build outputs live under `target/dist/`.
- The release tree root is `target/dist/rustzen-admin/`.
- The runtime image tag is `rustzen-admin:runtime`.

## Deploy Layout

```txt
/opt/rustzen-admin/
├── bin/
│   └── rustzen-admin
├── config/
│   └── app.env
├── data/
│   ├── uploads/
│   └── avatars/
├── logs/
├── systemd/
│   └── rustzen-admin.service
└── web/
    └── dist/
```

## Directory Rules

- `bin/`: executable binaries only.
- `config/`: deployment and runtime config only.
- `data/`: persistent application data.
- `data/uploads/`: user-uploaded files.
- `data/avatars/`: avatar files.
- `logs/`: runtime logs.
- `systemd/`: packaged service templates.
- `web/dist/`: frontend build output.

## Runtime Rules

- Run the service with `WorkingDirectory=/opt/rustzen-admin`.
- Set `RUSTZEN_RUNTIME_ROOT=.` in production so runtime paths resolve inside the deploy root.
- Backend static files must be served from `<runtime_root>/web/dist`.
- Uploads must be stored under `<runtime_root>/data/uploads/`.
- The backend must not depend on build-machine absolute paths.
- Both development and production must provide runtime config through environment variables.
- Do not configure a dedicated `/healthz` liveness probe; the service is supervised by systemd restart policy.
- Database connections must use `DATABASE_URL`.
- Other application runtime config must come from `RUSTZEN_*` environment variables.
- `config/app.env` is only the environment-variable carrier, not a second config model.
- Production deployment must not rely on code defaults for JWT secret or similar runtime settings.

## Production Minimum

- Production deployments must provide `DATABASE_URL` and `RUSTZEN_JWT_SECRET`.
- `RUSTZEN_RUNTIME_ROOT=.` is fixed by `deploy/rustzen-admin.service` for systemd deployments and by `deploy/runtime.Dockerfile` for the runtime image.
- Other `RUSTZEN_*` keys have code defaults, but keeping them explicit in `config/app.env` is still preferred.

## Runtime Config

### Canonical Files

- `.env`: local development env file
- `config/app.env`: backend process environment variables
- `.env.example`: the single env template for both development and deployment
- `deploy/binary.Dockerfile`: Docker multi-stage build for standalone backend binary export
- `deploy/release.Dockerfile`: Docker multi-stage build for release tree and zip export
- `deploy/runtime.Dockerfile`: Docker multi-stage build for runtime image export
- `deploy/rustzen-admin.service`: systemd service template
- `deploy/sql/repair_menu_schema.sql`: one-time repair SQL for older deployments

### `config/app.env` Templates

`config/app.env` can be kept minimal in production or expanded into a self-contained file. The release package build currently generates the complete production template by copying `.env.example` and rewriting `RUSTZEN_RUNTIME_ROOT=.`.

#### Minimum Production Template

```dotenv
DATABASE_URL=postgres://user:password@127.0.0.1:5432/rustzen_admin
RUSTZEN_JWT_SECRET=replace-me
```

- This is the smallest valid production `config/app.env`.
- `RUSTZEN_RUNTIME_ROOT=.` is fixed by `deploy/rustzen-admin.service` for systemd deployments and by `deploy/runtime.Dockerfile` for the runtime image.
- `RUST_LOG=info` and `RUST_BACKTRACE=1` are fixed outside `config/app.env`.

#### Complete Production Template

`.env.example` is the canonical full field list. Use this version when you want `config/app.env` to be self-contained.

```dotenv
DATABASE_URL=postgres://user:password@127.0.0.1:5432/rustzen_admin

RUSTZEN_APP_HOST=0.0.0.0
RUSTZEN_APP_PORT=8007

RUSTZEN_DB_MAX_CONN=4
RUSTZEN_DB_MIN_CONN=1
RUSTZEN_DB_CONN_TIMEOUT=10
RUSTZEN_DB_IDLE_TIMEOUT=600

RUSTZEN_JWT_SECRET=replace-me
RUSTZEN_JWT_EXPIRATION=3600

RUSTZEN_RUNTIME_ROOT=.
RUSTZEN_FILES_PREFIX=/resources

RUSTZEN_LOG_FILE_PREFIX=server
RUSTZEN_LOG_RETENTION_DAYS=7

RUST_LOG=info
```

- `DATABASE_URL` is the database connection string used by the backend and local migration tools.
- Production service startup reads `DATABASE_URL` for the database and `RUSTZEN_*` for the other application settings.
- `RUSTZEN_DB_IDLE_TIMEOUT` is measured in seconds. Set `0` only when you explicitly want to disable idle connection reaping.
- `RUSTZEN_RUNTIME_ROOT` is the single runtime directory root; the app derives `web/dist`, `data/`, and `logs/` from it.
- Runtime logs are written into `<runtime_root>/logs` with daily rolling files named as `RUSTZEN_LOG_FILE_PREFIX-YYYY-MM-DD.log`.
- Expired log files are deleted by the app itself based on `RUSTZEN_LOG_RETENTION_DAYS`.
- Local development may omit the standard `RUSTZEN_*` runtime keys because the backend provides code defaults for host, port, DB pool, JWT expiration, runtime root, file prefix, and log settings.
- Local-development runtime root defaults to `.rustzen-admin`; production should override it explicitly.
- `RUSTZEN_JWT_SECRET` has no code default and must be set explicitly in every environment.
- The frontend is developed and started separately in local development; only the packaged release is expected to contain `web/dist`.

### Config Rules

- `DATABASE_URL` and `RUSTZEN_*` env values are the deployment runtime config source.
- Development should use the same `DATABASE_URL` and `RUSTZEN_*` keys through local `.env`.
- `config/app.env` owns application process env values.
- `.env.example` should stay aligned with the fields the runtime actually reads.
- Production deployment should generate `config/app.env` from the complete field set shown above when it wants a self-contained file.
- Do not introduce `system.yaml` or another parallel runtime config source.
- Do not keep the same setting in code defaults, yaml, and env at the same time.
- Production runtime config must explicitly provide `DATABASE_URL` and `RUSTZEN_JWT_SECRET`.
- Local-development code defaults are available for `RUSTZEN_APP_HOST`, `RUSTZEN_APP_PORT`, `RUSTZEN_DB_MAX_CONN`, `RUSTZEN_DB_MIN_CONN`, `RUSTZEN_DB_CONN_TIMEOUT`, `RUSTZEN_DB_IDLE_TIMEOUT`, `RUSTZEN_JWT_EXPIRATION`, `RUSTZEN_RUNTIME_ROOT`, `RUSTZEN_FILES_PREFIX`, `RUSTZEN_LOG_FILE_PREFIX`, and `RUSTZEN_LOG_RETENTION_DAYS`.
- `RUSTZEN_JWT_SECRET` has no code default and must be set explicitly.
- Deployment directories may be relative to `WorkingDirectory`; runtime code must not assume build-machine absolute paths.
- Production database configuration must use a PostgreSQL connection URL in `DATABASE_URL`.

### Operational Constraints

- `working_dir` must match the deploy root.
- `ExecStart` must point to `bin/rustzen-admin`.
- Frontend static assets must be deployed to `<runtime_root>/web/dist`.
- Upload data must be stored under `<runtime_root>/data/uploads/`.
- Avatar data must be stored under `<runtime_root>/data/avatars/` and served from the shared file prefix.
- Public file access must use the shared route prefix from `RUSTZEN_FILES_PREFIX`.
- Service values must not be split across multiple config models.
- Deployment examples in this document are production requirements, not a description of every current code default.

## Build Flow

- Local development keeps frontend and backend separate; do not assume the backend will serve `web/dist` unless you are using a packaged deploy tree.
- `deploy/binary.Dockerfile` exports the standalone backend binary.
- `deploy/release.Dockerfile` assembles the release tree and writes `rustzen-admin.zip`.
- `deploy/runtime.Dockerfile` bakes the release layout into a deployable image.
- `just build-binary` exports `target/dist/bin/rustzen-admin`.
- `just build-release` exports `target/dist/rustzen-admin/` and `target/dist/rustzen-admin.zip`.
- `just build-image` builds `rustzen-admin:runtime` as a local Docker image for direct deployment or manual testing.
- The release tree contains `bin/`, `config/`, `data/uploads/`, `data/avatars/`, `logs/`, `systemd/`, and `web/dist/`.
- `config/app.env` is generated from `.env.example` and rewritten with `RUSTZEN_RUNTIME_ROOT=.`
- The backend runs embedded SQLx migrations on startup, so new deploys do not need a manual migration step.

## Docker Build

- Use `deploy/binary.Dockerfile` for standalone backend binary builds, `deploy/release.Dockerfile` for release tree and zip builds, and `deploy/runtime.Dockerfile` for runtime image builds.
- On Apple Silicon, build with `--platform linux/amd64` to produce Ubuntu x86_64 artifacts.
- The binary Dockerfile uses only the Rust backend stage and exports the final executable.
- The release Dockerfile uses frontend and backend build stages, then exports the release tree and zip.
- The runtime Dockerfile uses frontend and backend build stages, then exports the runtime image.
- Build the `binary` target to `target/dist/bin/`.
- Build the `release` target to `target/dist/rustzen-admin/`.
- Build the `runtime` target with `--load` when you want the image in the local Docker daemon.
- The `just` entrypoints for this flow are `just build-binary`, `just build-release`, and `just build-image`.

## systemd Example

```ini
[Unit]
Description=rustzen-admin
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/rustzen-admin
ExecStart=/opt/rustzen-admin/bin/rustzen-admin
Restart=always
RestartSec=5
EnvironmentFile=/opt/rustzen-admin/config/app.env
Environment=RUSTZEN_RUNTIME_ROOT=.
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
```

## Checks

- `bin/rustzen-admin` exists.
- `<runtime_root>/web/dist/index.html` exists.
- `config/app.env` is present in production.
- `config/app.env` contains all required `RUSTZEN_*` values.
- `<runtime_root>/data/uploads/` exists and is writable.
- `<runtime_root>/data/avatars/` exists or can be created by the app.
- `<runtime_root>/logs/` exists and is writable.
- `systemd/rustzen-admin.service` exists in the release package.

## One-time Repair SQL

- Use `deploy/sql/repair_menu_schema.sql` only for existing deployments that already ran older migrations.
- Run it manually before the service starts if `menus.parent_code` or `menus.is_manual` is missing.
- New databases should not need this script because the base migrations already include the final schema.

Example:

```bash
psql "$DATABASE_URL" -f deploy/sql/repair_menu_schema.sql
```
