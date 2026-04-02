# Deployment Guidelines

> This document defines production deployment layout, release flow, and runtime configuration sources.

## Scope

- Use one deploy root for the whole app.
- Keep backend binary, frontend bundle, config, data, logs, and versions under the same root.
- Do not add extra compatibility paths or fallback layouts.

## Deploy Layout

```txt
/opt/rustzen-admin/
├── bin/
│   └── rustzen-admin
├── config/
│   └── app.env
├── data/
│   ├── server/
│   └── uploads/
├── logs/
├── versions/
└── web/
    └── dist/
```

## Directory Rules

- `bin/`: executable binaries only.
- `config/`: deployment and runtime config only.
- `data/`: persistent application data.
- `data/server/`: backend persistence data.
- `data/uploads/`: user-uploaded files.
- `logs/`: runtime logs.
- `versions/`: release packages and archived bundles.
- `web/dist/`: frontend build output.

## Runtime Rules

- Run the service with `WorkingDirectory=/opt/rustzen-admin`.
- Backend static files must be served from `web/dist`.
- Uploads must be stored under `data/uploads/`.
- The backend must not depend on build-machine absolute paths.
- Both development and production must provide runtime config through environment variables.
- Database connections must use `DATABASE_URL`.
- Other application runtime config must come from `RUSTZEN_*` environment variables.
- `config/app.env` is only the environment-variable carrier, not a second config model.
- Production deployment must not rely on code defaults for JWT secret or similar runtime settings.

## Runtime Config

### Canonical Files

- `.env`: local development env file
- `config/app.env`: backend process environment variables
- `.env.example`: the single env template for both development and deployment
- `deploy/systemd/rustzen-admin.service`: systemd service template

### `app.env` Suggested Fields

`.env.example` is the canonical field list. `config/app.env` should mirror it for production.

```dotenv
DATABASE_URL=postgres://user:password@127.0.0.1:5432/rustzen_admin

RUSTZEN_APP_HOST=0.0.0.0
RUSTZEN_APP_PORT=8007

RUSTZEN_DB_MAX_CONN=10
RUSTZEN_DB_MIN_CONN=1
RUSTZEN_DB_CONN_TIMEOUT=10
RUSTZEN_DB_IDLE_TIMEOUT=0

RUSTZEN_JWT_SECRET=replace-me
RUSTZEN_JWT_EXPIRATION=3600

RUSTZEN_WEB_DIST=web/dist
RUSTZEN_UPLOAD_DIR=data/uploads
RUSTZEN_AVATAR_DIR=data/uploads/avatars
RUSTZEN_UPLOAD_PUBLIC_PREFIX=/uploads
```

- `DATABASE_URL` is the database connection string used by the backend and local migration tools.
- Production service startup reads `DATABASE_URL` for the database and `RUSTZEN_*` for the other application settings.

### Config Rules

- `DATABASE_URL` and `RUSTZEN_*` env values are the deployment runtime config source.
- Development should use the same `DATABASE_URL` and `RUSTZEN_*` keys through local `.env`.
- `config/app.env` owns application process env values.
- `.env.example` should stay aligned with the fields the runtime actually reads.
- Production deployment should generate `config/app.env` from the same field set as `.env.example`.
- Do not introduce `system.yaml` or another parallel runtime config source.
- Do not keep the same setting in code defaults, yaml, and env at the same time.
- Deployment directories may be relative to `WorkingDirectory`; runtime code must not assume build-machine absolute paths.
- Production database configuration must use a PostgreSQL connection URL in `DATABASE_URL`.

### Operational Constraints

- `working_dir` must match the deploy root.
- `ExecStart` must point to `bin/rustzen-admin`.
- Frontend static assets must be deployed to `web/dist`.
- Upload data must be stored under `data/uploads/`.
- Service values must not be split across multiple config models.
- Deployment examples in this document are production requirements, not a description of every current code default.

## Build Flow

- Build frontend first.
- Build backend release binary.
- Assemble the deployment tree under `rustzen-admin/`.
- Copy the binary to `bin/rustzen-admin`.
- Copy frontend output to `web/dist/`.
- Generate `config/app.env` from the runtime field set.
- Package the directory as `rustzen-admin.zip`.
- Start or restart the system service after extracting the package.

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
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
```

## Checks

- `bin/rustzen-admin` exists.
- `web/dist/index.html` exists.
- `config/app.env` is present in production.
- `config/app.env` contains all required `RUSTZEN_*` values.
- `data/uploads/` exists and is writable.
- `logs/` exists and is writable.
