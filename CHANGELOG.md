# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Monorepo directories renamed to `zen-core/`, `zen-server/`, and `zen-web/` (Cargo package names unchanged: `rustzen-core`, `server`). Packaged deployments still place static assets under `<runtime_root>/web/dist`, not under the `zen-web/` source folder name.
- Older changelog sections below may still name directories as they were at release time (for example `server/` or `web/`); the current tree uses `zen-server/` and `zen-web/` (plus `zen-core/`).

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
- One-time repair script for legacy databases: `deploy/sql/repair_menu_schema.sql` (adds `menus.parent_code` and `menus.is_manual` where missing).

### Changed

- Environment variables use the `RUSTZEN_` prefix (see `.env.example`). `DATABASE_URL` remains unprefixed.
- Replaced per-path env keys (`web_dist`, `upload_dir`, `avatar_dir`, `upload_public_prefix`) with `RUSTZEN_RUNTIME_ROOT` and `RUSTZEN_FILES_PREFIX`.
- CI and `just` targets updated for the new layout (`just build-binary`, `just build-release`, `just build-image`). See `justfile` and `docs/deployment-guide.md`.

### Migration notes

- **Fresh installs**: use `.env.example`, run the backend once so embedded migrations apply; no repair script needed.
- **Existing databases** that predate `parent_code` / `is_manual` on `menus`: run `psql "$DATABASE_URL" -f deploy/sql/repair_menu_schema.sql` once before startup. Details: `docs/deployment-guide.md`, `docs/permission-guide.md`.
- **Full upgrade steps**: `docs/deployment-guide.md`.
