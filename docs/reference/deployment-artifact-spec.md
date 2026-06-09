# Deployment Artifact Spec

This is a reference appendix for the deployment guide.

## Naming

- `server` means the standalone Linux x86_64 backend executable.
- `web` means the frontend dist zip archive.
- All exported build outputs live under `target/rustzen-admin/`.

## Build Outputs

```txt
target/rustzen-admin/
├── rustzen-admin-<version>
└── dist-<version>.zip
```

## Dockerfiles

- Root `Dockerfile` exports the standalone backend binary used by `just build-server`.
- `just build` writes the versioned server binary and web dist zip.

On Apple Silicon, these builds target `linux/amd64`.

Use the root `justfile` as the command source of truth; inspect the relevant target before running it.

## Deploy Versions

- The deploy UI manages uploaded component versions, not remote SSH targets.
- Supported components are `server` and `web`.
- Each version row records `component`, `version`, `arch`, `filePath`, `fileSize`, `fileHash`, current/deployed/expired state, deploy time, deploy user, and notes.
- `server` files are saved under `<runtime_root>/versions/server-<version>-<arch>`.
- `web` zip files are saved under `<runtime_root>/web/web-<version>.zip`.
- Before deploy, the server re-checks the saved file size, SHA-256, and component-specific validation rules.

## Upload Validation

- `server` uploads must be executable ELF, Mach-O, or shebang-script files.
- `server` uploads must contain `RUSTZEN_ADMIN_MARKER\ncomponent=server\n`.
- `server` uploads with detectable architecture must match the selected `x86_64` or `aarch64` arch.
- `web` uploads must be zip files.
- `web` zips must contain `dist/index.html`.
- `web` zips must contain at least one `dist/assets/*.js` or `dist/assets/*.css` file.
- `web` zips must contain `dist/__rustzen_admin_marker__.json` with `component: "web"` and `build_id: "manual"`.

## Deploy Behavior

- Deploying `server` atomically switches `<runtime_root>/bin/rustzen-admin` to the uploaded version with a symlink, marks the version current, and triggers `systemctl restart rustzen-admin.service`.
- Deploying `web` extracts the saved zip and replaces `<runtime_root>/web/dist`.
- Marking a version expired is blocked for the current version.
- Deleting a version is blocked for the current version and removes the saved file when possible.

## systemd Shape

The service template lives at `deploy/rustzen-admin.service`.

```ini
[Service]
WorkingDirectory=/opt/rustzen-admin
ExecStart=/opt/rustzen-admin/bin/rustzen-admin
EnvironmentFile=/opt/rustzen-admin/config/app.env
Environment=RUSTZEN_RUNTIME_ROOT=.
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
```

## Config

```dotenv
RUSTZEN_SQLITE_PATH=./data/rustzen.db
RUSTZEN_JWT_SECRET=<production-secret>
```

`.env.example` is the canonical complete field list for self-contained deployments.
