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

1. Read `README.md`.
2. Read `AGENTS.md`.
3. For product boundary, positioning, direction, or module-purpose decisions,
   read `docs/product/product.md`.
4. Read the nearest subdirectory `AGENTS.md`.
5. Read only the relevant guide in `docs/guides/`.
6. Use `docs/reference/` only for deeper context.

## Boundaries

- RustZen classification: Web/Rust A-class reference layout.
- Shared auth and permission capability code lives in `crates/auth/`.
- Shared Manifest, route, and delegation contracts live in `crates/ipc/`.
- Backends live in `apps/admin/`, `apps/monitor/`, `apps/insights/`, and
  `apps/reports/`.
- Each backend owns its migrations under its application directory.
- Frontend lives in `apps/web/`.
- Deployment assets live in `deploy/`.
- Root keeps workspace metadata, docs, command entry points, and shared crates.
- Deployment contract uses one signed `target/rz/rz-<version>-<arch>.tar`
  bundle, `/opt/rz`, `deploy/rz.target`, `deploy/rz-recovery.service`, four
  server units, and `deploy/setup-layout.sh`.
- Do not apply Peripheral Vercel, Tauri client, or legacy `zen-server` /
  `zen-web` layout rules to this repository.
- Do not add systemd `User`/`Group`, hardening, or install-path permission
  changes without reviewing `deploy/setup-layout.sh` and the `/opt` runtime
  directory ownership model together.

## Working Rules

- Prefer the smallest viable change.
- Use `.codex/` as the local task workspace. Keep unfinished cross-session handoffs
  under `.codex/handoffs/<task-id>.md` and raw review packages, responses, ledgers,
  and attachments under `.codex/reviews/<review-id>/`; both directories are ignored.
  Put only explicitly requested, sanitized, durable artifacts under `docs/`.
- Do not add fallback or compatibility logic.
- Keep stable product decisions in `docs/product/product.md`; keep stable
  implementation facts and rules in `docs/architecture.md` and `docs/guides/`.
- Keep subdirectory `AGENTS.md` files thin.
- Do not use `docs/reference/` or `docs/history/` as default implementation truth.
- SQLite is the default storage backend.
- Update code, docs, and commands together when structure changes.
- Keep task completion tied to the task's verification commands before updating status.
