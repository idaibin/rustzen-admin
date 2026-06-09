# Deployment Artifact Spec

This is a reference appendix for the deployment guide.

## Naming

- `binary` means the standalone Linux x86_64 backend executable.
- `release` means the deployable directory tree plus zip archive.
- `runtime` means the Docker image that runs the full application.
- All exported build outputs live under `target/dist/`.

## Build Outputs

```txt
target/dist/
├── bin/
│   └── rustzen-admin
├── rustzen-admin/
│   ├── bin/
│   ├── config/
│   ├── data/
│   │   ├── uploads/
│   │   └── avatars/
│   ├── logs/
│   ├── install.sh
│   ├── systemd/
│   └── web/
│       └── dist/
└── rustzen-admin.zip
```

## Dockerfiles

- `deploy/binary.Dockerfile` exports the standalone backend binary.
- `deploy/release.Dockerfile` builds frontend and backend artifacts, assembles the release tree, and writes `rustzen-admin.zip`.
- `deploy/runtime.Dockerfile` builds the runtime image.

On Apple Silicon, these builds target `linux/amd64`.

Use the root `justfile` as the command source of truth; inspect the relevant target before running it.

## Release Tree Rules

- `bin/rustzen-admin` is the backend executable.
- `config/app.env` is generated from `.env.example` and rewritten with `RUSTZEN_RUNTIME_ROOT=.`.
- `web/dist/` contains the frontend production bundle.
- `data/uploads/`, `data/avatars/`, and `logs/` are runtime directories.
- `install.sh` is the packaged installer used when installing the release tree.
- `systemd/rustzen-admin.service` is the packaged service template.

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

```ini
[Service]
WorkingDirectory=/opt/rustzen-admin
ExecStart=/opt/rustzen-admin/bin/rustzen-admin
EnvironmentFile=/opt/rustzen-admin/config/app.env
Environment=RUSTZEN_RUNTIME_ROOT=.
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
```

## Config Template

Minimum production `config/app.env`:

```dotenv
RUSTZEN_SQLITE_PATH=./data/rustzen.db
RUSTZEN_JWT_SECRET=rustzen-admin-release-{version}
```

`.env.example` is the canonical complete field list for self-contained deployments.
