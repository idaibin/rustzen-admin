# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2026-09-22

### Added

- Added `docs/product/product.md` as the current product foundation and fact
  source for positioning, direction, module purposes, and product non-goals.
- Added independently consumable product specifications for role definition
  management and user role-assignment readiness.
- Added one signed full-server bundle containing Web, Admin, Monitor, Insights,
  Reports, and the `rz` lifecycle CLI.
- Added the canonical `rz-full.service` target and update/recovery units for the
  four independent services.
- Added release screenshots for all 19 authenticated business routes.

### Changed

- Preserve every module navigation page when multiple pages share one capability;
  presentation overrides now address navigation inventory IDs through
  `PUT /api/system/menus/inventory/{id}` without changing permission grants.
- Isolate Reports browser profiles per execution and close browsers on setup
  failure as well as normal completion; remove owned profiles after shutdown.
- Extend local verification for Monitoring gateway/RBAC/alert-policy journeys,
  Reports daily/weekly schedule lifecycle, and concurrent browser execution.
- Repositioned `rustzen-admin` as a self-hosted product first and a structured
  Rust full-stack reference implementation second.
- Enforced non-empty user-role and custom-role permission assignments at the
  backend boundary, and limited role options to roles the current operator may
  assign.
- **Breaking:** Reduced Dashboard to account totals and module health, removing
  the duplicated Dashboard health, metrics, and trends endpoints and leaving
  detailed resource and trend views with their owning System and product modules.
- Reused shared module response, pagination, timezone, logging, date display,
  form-field, and query-option mechanisms where wire and UI semantics match.
- Reused one date-time formatter across management and system tables that share
  the same empty-value and locale behavior.
- Centralized the four service health responses and the deployment health-gate
  deserialization contract in `crates/ipc`.
- Documented the current product boundary, shared-capability gate, verified
  reuse index, and live former-product comparison for future module work.
- Standardized fresh deployment on `/opt/rz`, one generated installer, `rz start`,
  `rz stop`, `rz restart`, and `rz status`.

### Fixed

- Remove user-role assignments atomically when deleting a user, and return the
  shared JSON error envelope for invalid Insights collection-policy fields.
- Validate uploaded avatars by decoding PNG/JPEG content and derive their stored
  extension from the detected format.
- Restore search-result clicks, Dashboard locale updates, custom navigation
  titles, and fresh task history after execution.
- Drain Reports executions on SIGTERM/SIGINT, close owned browsers, and persist
  interrupted runs before exiting.
- Emit portable release archives without macOS extended attributes; repair the
  installer signature fallback when `xxd` is unavailable, including UTF-8 locales.
- Restored lint and TypeScript checks for the Vite configuration while
  preserving the TanStack Router, React, and Tailwind plugin order.
- Retired stale core route permissions during startup sync while preserving
  manual menus and module-owned capabilities.
- Restored operation-log coverage for authenticated module enable and disable
  requests, and moved the module-health response DTO to the Modules owner.
- Fixed the four-service verification fixture to write module logs into each
  service's owned log directory.

### Removed

- **Breaking:** Removed the unconsumed Dictionary HTTP/UI surface, permission,
  and navigation. The historical SQLite table remains dormant for upgrade
  compatibility.
- Removed dead module capability constants, an orphan Monitor settings handler,
  unused frontend Alert source, demo navigation entries, and Admin placeholder
  endpoints.

## [0.5.0] - 2026-07-02

### Added

- Added the new default role baseline for fresh deployments: `owner`, `admin`,
  and `viewer`.
- Added direct initialization of `superadmin` with the `owner` role.

### Changed

- Changed fresh SQLite seed data to initialize the final built-in role model
  directly.
- Changed `admin` to keep deploy view-only access while retaining other
  concrete leaf capabilities.
- Changed `viewer` to receive read-only capabilities, including deploy list
  access.
- Adopted the shared Rustzen core runtime layout helper.

### Removed

- Removed the `SYSTEM_ADMIN` seed and owner-role compatibility path.
- Removed startup compatibility migrations for previous SQLite and deploy file
  layouts.
- Removed PostgreSQL migration files from the current deployment baseline.

## [0.4.2] - 2026-06-24

### Changed

- Slimmed the server release binary by replacing the scheduled-task runtime with
  lightweight cron calculation and fixed-offset timezone handling.

### Fixed

- Added the legacy `build_id: "manual"` field to signed web deploy markers so
  new web release zips can still be uploaded through existing deployed servers.
- Fixed signed web deploy package hashing so uploaded release zips validate
  against the backend signature policy.
- Changed server deployment to return before triggering the service restart, so
  nginx does not report the deploy request as a bad gateway during restart.

## [0.4.1] - 2026-06-24

### Added

- Added signed deploy artifacts for server binaries and web release zips.
- Added the system status page and API for runtime storage and resource usage.

### Changed

- Updated the release workflow to read `RUSTZEN_DEPLOY_SIGN_KEY` from GitHub Actions secrets.
- Updated production build config to require deploy signature verification.

## [0.4.0] - 2026-06-08

### Added

- Added SQLite-first embedded migrations and seed data under `apps/server/migrations/sqlite/`.
- Added shared `crates/config`, `crates/runtime`, and `crates/storage` helpers for runtime configuration, paths, and SQLite startup.
- Added repository rules and guides for the `apps/server`, `apps/web`, and `crates/*` layout.

### Changed

- **Breaking:** SQLite is now the default runtime storage backend.
- Moved source ownership to the current monorepo layout: `apps/server`, `apps/web`, and shared `crates/*`.
- Moved local development ports to backend `9801` and frontend `9800`.
- Reworked system permission checks around explicit capability boundaries while keeping existing capability codes.
- Updated deployment assets, runtime configuration, and documentation for the SQLite-first baseline.

### Fixed

- Fixed collapsed sidebar menu icons so they remain visible when the navigation is minimized.

### Migration notes

- Existing PostgreSQL-first deployments need a separate migration review before adopting the SQLite-first runtime.
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
