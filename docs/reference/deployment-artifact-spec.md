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
- `systemd/rustzen-admin.service` is the packaged service template.

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
RUSTZEN_JWT_SECRET=replace-me
```

`.env.example` is the canonical complete field list for self-contained deployments.
