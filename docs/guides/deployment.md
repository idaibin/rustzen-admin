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
├── versions/
└── web/
    └── dist/
```

## Rules

- Use one deploy root for the whole app.
- Production runs with `WorkingDirectory=/opt/rustzen-admin`.
- Production sets `RUSTZEN_RUNTIME_ROOT=.`.
- Runtime config comes from `RUSTZEN_SQLITE_PATH` and `RUSTZEN_*`.
- Production must provide `RUSTZEN_SQLITE_PATH` and `RUSTZEN_JWT_SECRET`.
- `config/app.env` is only an environment-variable carrier.
- Backend static files are served from `<runtime_root>/web/dist`.
- Uploads live under `<runtime_root>/data/uploads`.
- Avatars live under `<runtime_root>/data/avatars`.
- Logs live under `<runtime_root>/logs`.
- Uploaded server versions live under `<runtime_root>/versions`.
- Build and deploy targets are defined in the root `justfile`.
- `just build` writes versioned deployment outputs under `target/rustzen-admin/`.
- Build outputs are flat files: `rustzen-admin-<version>` and `dist-<version>.zip`.
- Frontend release builds use pnpm with `apps/web/pnpm-lock.yaml`.
- The sqlite-first phase uses SQLite by default and does not require PostgreSQL for local startup.
- Deploy version management accepts only `server` and `web` components.
- `server` uploads are executable binary files with a `RUSTZEN_ADMIN_MARKER` marker and matching `x86_64` or `aarch64` arch.
- `web` uploads are zip files containing `dist/index.html`, `dist/assets/*.js` or `*.css`, and `dist/__rustzen_admin_marker__.json`.
- Deploying `server` switches `<runtime_root>/bin/rustzen-admin` to the uploaded version and triggers `rustzen-admin.service` restart. The service template lives at `deploy/rustzen-admin.service`.
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

## Build Output Checks

- `target/rustzen-admin/rustzen-admin-<version>` exists and is executable.
- `target/rustzen-admin/dist-<version>.zip` exists.
- The web zip contains `dist/index.html`.
- The web zip contains at least one `dist/assets/*.js` or `dist/assets/*.css` file.
- The web zip contains `dist/__rustzen_admin_marker__.json`.

## Deploy Checks

- Uploaded deploy file SHA-256 matches the stored database record before deployment.
