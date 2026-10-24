# Repository Rules

## Source of Truth

- Product positioning, direction, and module-purpose facts:
  [docs/product/product.md](./docs/product/product.md).
- Current implementation facts: source code, then
  [docs/architecture.md](./docs/architecture.md), then
  [docs/guides/](./docs/guides/).
- AI contribution constraints: [docs/guides/ai-coding-rules.md](./docs/guides/ai-coding-rules.md).
- Command source: root `justfile`; inspect the target before running it.

## Reading Order

Unless explicitly relative (such as `../../DESIGN.md`), `docs/` and command paths
in this repository's nested guidance are relative to the repository root.

1. Read `README.md`.
2. Read `AGENTS.md`.
3. For product boundary, positioning, direction, or module-purpose decisions,
   read `docs/product/product.md`.
4. Read the nearest subdirectory `AGENTS.md`.
5. Read only the relevant guide in `docs/guides/`.
6. Use `docs/reference/` only for deeper context.

## Boundaries

- Rustzen classification: Web/Rust A-class reference layout.
- Shared auth and permission capability code lives in `crates/auth/`.
- Shared Manifest, route, and delegation contracts live in `crates/ipc/`.
- Backends live in `apps/admin/`, `apps/monitor/`, `apps/insights/`, and
  `apps/reports/`.
- Each backend owns its migrations under its application directory.
- Frontend lives in `apps/web/`.
- Deployment assets live in `deploy/`.
- Root keeps workspace metadata, docs, command entry points, and shared crates.
- Full remains the default signed `target/rz/rz-<version>-<arch>.tar` bundle;
  the explicit core build writes `target/rz/core/rz-<version>-<arch>.tar` with
  only `rz` and `rz-admin` plus the core units. Both install under `/opt/rz` and
  use `deploy/rz-recovery.service` and `deploy/setup-layout.sh`; full starts four
  server units while core starts Admin only.
- Do not apply Peripheral, Tauri client, or legacy `zen-server` /
  `zen-web` layout rules to this repository.
- Do not add systemd `User`/`Group`, hardening, or install-path permission
  changes without reviewing `deploy/setup-layout.sh` and the `/opt` runtime
  directory ownership model together.

## Working Rules

- Versioning belongs to `rustzen-admin` independently of `ops-suite` and former
  standalone products. Follow this repository's `CHANGELOG.md` and explicitly
  approved release plan; keep unreleased work under `Unreleased` without an implicit
  version bump. Synchronize the workspace, Web package, lockfile, and current
  examples only when a version change is authorized.
- This repository is an initialization template. Maintain only the final fresh-install baseline;
  general historical compatibility is out of scope. The narrowly approved exception
  is the immediately preceding supported full release and its data, owned by
  [the composition contract](docs/product/features/composable-distribution/spec.md#compatibility-and-trust-boundary)
  and OE-A09–12. Preserve its exact signed identity, data and recovery boundary;
  that exception does not authorize arbitrary migrations or existing-data resets.
- When a schema changes, update that application's existing initialization migration directly. Do
  not add sequential upgrade migrations, legacy data conversion, dual-read or dual-write paths,
  fallback branches, rollback compatibility, or old-database support.
- Acceptance starts from a newly initialized database. Do not delete an existing database during
  implementation unless the user explicitly requests that destructive action.
- Keep stable product decisions in `docs/product/product.md`; keep stable
  implementation facts and rules in `docs/architecture.md` and `docs/guides/`.
- Do not use `docs/reference/` or `docs/history/` as default implementation truth.
- SQLite is the default storage backend.
- Update code, docs, and commands together when structure changes.
