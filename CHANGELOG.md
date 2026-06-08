# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-06-08

### Added

- Added SQLite-first embedded migrations and seed data under `apps/server/migrations/sqlite/`.
- Added shared `crates/config`, `crates/runtime`, and `crates/storage` helpers for runtime configuration, paths, and SQLite startup.
- Added repository rules and guides for the `apps/server`, `apps/web`, and `crates/*` layout.

### Changed

- **Breaking:** SQLite is now the default runtime storage backend. PostgreSQL-first behavior is preserved on the `legacy/pg-admin` branch and the `pg-admin` tag.
- Moved source ownership to the current monorepo layout: `apps/server`, `apps/web`, and shared `crates/*`.
- Moved local development ports to backend `9800` and frontend `9801`.
- Reworked system permission checks around explicit capability boundaries while keeping existing capability codes.
- Updated deployment assets, runtime configuration, and documentation for the SQLite-first baseline.

### Fixed

- Fixed collapsed sidebar menu icons so they remain visible when the navigation is minimized.

### Migration notes

- Existing PostgreSQL-first deployments should stay on `legacy/pg-admin` or the `pg-admin` tag.
- Fresh SQLite-first installs should use `.env.example`, set `RUSTZEN_JWT_SECRET`, and start the backend so embedded migrations apply.
- If a local development database has stale migration checksums, run `just reset-db` before restarting the backend.

## [0.3.0] - 2026-04-05

### Added

- Monorepo layout: backend under `server/`, frontend under `web/`, SQL migrations under `server/migrations/`.
- Embedded migrations applied automatically on backend startup (`run_migrations` in `server/src/infra/db.rs`).
- Centralized runtime layout via `RUSTZEN_RUNTIME_ROOT` (default `.rustzen-admin` in development), deriving:
    - `web/dist` for static frontend assets
    - `data/uploads` and `data/avatars` for uploads
    - `logs` for file logging (with retention controlled by `RUSTZEN_LOG_RETENTION_DAYS`)
- Separate static route for avatars at `{RUSTZEN_FILES_PREFIX}/avatars` (default `/resources/avatars`).
- Split Docker build assets: `deploy/binary.Dockerfile`, `deploy/release.Dockerfile`, `deploy/runtime.Dockerfile`.

### Changed

- Environment variables use the `RUSTZEN_` prefix (see `.env.example`). `DATABASE_URL` remains unprefixed.
- Replaced per-path env keys (`web_dist`, `upload_dir`, `avatar_dir`, `upload_public_prefix`) with `RUSTZEN_RUNTIME_ROOT` and `RUSTZEN_FILES_PREFIX`.
- CI and `just` targets updated for the new layout (`just build-binary`, `just build-release`, `just build-image`). See `justfile` and `docs/guides/deployment.md`.

### Migration notes

- **Fresh installs**: use `.env.example`, run the backend once so embedded migrations apply; no repair script needed.
- **Full upgrade steps**: `docs/guides/deployment.md`.
