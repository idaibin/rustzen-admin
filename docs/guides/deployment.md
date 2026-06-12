# Deployment Guide

Current deployment rules.

## Runtime Layout

```txt
<runtime_root>/
├── bin/
│   ├── rustzen-admin -> rustzen-admin-<version>-<arch>
│   └── rustzen-admin-<version>-<arch>
├── config/
├── data/
│   ├── avatars/
│   ├── db/
│   └── uploads/
├── logs/
├── systemd/
├── versions/
└── web/
    └── dist/
```

## Rules

- Use one deploy root for the whole app.
- Production runs with `WorkingDirectory=/opt/rustzen-admin`.
- Production sets `RUSTZEN_RUNTIME_ROOT=.`.
- Runtime config comes from `RUSTZEN_SQLITE_PATH` and `RUSTZEN_*`.
- The deployment backend API port is `9880`.
- Production must provide `RUSTZEN_SQLITE_PATH` and `RUSTZEN_JWT_SECRET`.
- `config/app.env` is only an environment-variable carrier.
- `RUSTZEN_TIMEZONE` controls process-local timezone behavior such as local log dates and scheduled task cron evaluation; the default is `UTC`.
- Backend static files are served from `<runtime_root>/web/dist`.
- SQLite database files live under `<runtime_root>/data/db`.
- Uploads live under `<runtime_root>/data/uploads`.
- Avatars live under `<runtime_root>/data/avatars`.
- Logs live under `<runtime_root>/logs`.
- Installed build artifacts initialize `bin/rustzen-admin` as a symlink to the packaged server version, for example `bin/rustzen-admin-0.1.1-x86_64`.
- Uploaded server versions live under `<runtime_root>/versions/server-<version>-<arch>`.
- Build and deploy targets are defined in the root `justfile`.
- `just build` writes versioned deployment outputs under `target/rustzen-admin/`.
- Build outputs include `rustzen-admin-<version>`, `dist-<version>.zip`, `config/app.env`, `systemd/rustzen-admin.service`, and `setup-layout.sh`.
- The build `config/app.env` is copied from `.env.example` with `RUSTZEN_RUNTIME_ROOT=.`, `RUSTZEN_APP_PORT=9880`, and `RUSTZEN_SQLITE_PATH=./data/db/rustzen.db` for the deploy layout.
- The build `config/app.env` keeps the release JWT placeholder; production must replace it with a real secret.
- `setup-layout.sh` creates the deploy directory structure from the build outputs and does not overwrite an existing `config/app.env`.
- `setup-layout.sh` enables the systemd service but skips startup while `RUSTZEN_JWT_SECRET` is still a release placeholder.
- `setup-layout.sh` prints the JWT update command and follow-up `systemctl` commands after setup.
- Production must replace the JWT secret before startup.
- Frontend release builds use pnpm with `apps/web/pnpm-lock.yaml`.
- The sqlite-first phase uses SQLite by default and does not require PostgreSQL for local startup.
- Deploy version management accepts only `server` and `web` components.
- `server` uploads are executable binary files with a `RUSTZEN_ADMIN_MARKER` marker and matching `x86_64` or `aarch64` arch.
- `web` uploads are zip files containing `dist/index.html`, `dist/assets/*.js` or `*.css`, and `dist/__rustzen_admin_marker__.json`.
- Deploying `server` switches `<runtime_root>/bin/rustzen-admin` to the uploaded version file in `<runtime_root>/versions/` and triggers `rustzen-admin.service` restart. The service template lives at `deploy/rustzen-admin.service`.
- Server deploy records are marked current only after the restart trigger is accepted.
- Deploying `web` replaces `<runtime_root>/web/dist` from the uploaded zip.
- Web deploy restores the previous `web/dist` when the database current-version update fails.
- Deleting or cleaning deploy versions removes the uploaded file before marking the database record deleted; file deletion failure leaves the database record visible.

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

## Existing SQLite Path Move

For deployments that already created SQLite files directly under `data/`, move
the database files into `data/db/` while the service is stopped:

```bash
systemctl stop rustzen-admin
mkdir -p /opt/rustzen-admin/data/db
if ls /opt/rustzen-admin/data/rustzen.db* >/dev/null 2>&1; then
  mv /opt/rustzen-admin/data/rustzen.db* /opt/rustzen-admin/data/db/
fi
grep -q '^RUSTZEN_SQLITE_PATH=./data/db/rustzen.db$' /opt/rustzen-admin/config/app.env || \
  sed -i 's#^RUSTZEN_SQLITE_PATH=.*#RUSTZEN_SQLITE_PATH=./data/db/rustzen.db#' /opt/rustzen-admin/config/app.env
systemctl start rustzen-admin
```

## Build Output Checks

- `target/rustzen-admin/rustzen-admin-<version>` exists and is executable.
- `target/rustzen-admin/dist-<version>.zip` exists.
- `target/rustzen-admin/config/app.env` exists and sets `RUSTZEN_RUNTIME_ROOT=.`, `RUSTZEN_APP_PORT=9880`, and `RUSTZEN_SQLITE_PATH=./data/db/rustzen.db`.
- `target/rustzen-admin/systemd/rustzen-admin.service` exists.
- `target/rustzen-admin/setup-layout.sh` exists and is executable.
- The web zip contains `dist/index.html`.
- The web zip contains at least one `dist/assets/*.js` or `dist/assets/*.css` file.
- The web zip contains `dist/__rustzen_admin_marker__.json`.

## Installed Layout Checks

- `bin/rustzen-admin` exists.
- `web/dist/index.html` exists.
- `config/app.env` exists.
- Runtime data and log directories are writable.
- `data/db` exists for SQLite database files.
- `systemd/rustzen-admin.service` exists.
- `systemd/rustzen-admin.service` does not set `TZ`; backend startup sets process `TZ` from `RUSTZEN_TIMEZONE`.
- `config/app.env` carries `RUSTZEN_TIMEZONE=UTC` for process-local timezone behavior.
- Uploaded deploy file SHA-256 matches the stored database record before deployment.
