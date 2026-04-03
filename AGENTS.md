# Repository Guidelines

## Source of Truth

- `README.md`: developer entry point and document map
- `AGENTS.md`: repository-level collaboration rules
- `server/AGENTS.md`: backend quick-entry guidance inside `server/`
- `web/AGENTS.md`: frontend quick-entry guidance inside `web/`
- `docs/architecture.md`: repository layout, document layers, repo boundaries, and command summary
- `docs/deployment-guide.md`: production deployment rules, packaging, service startup, and runtime config requirements
- `docs/permission-guide.md`: current permission model and usage constraints
- `docs/backend-guide.md`: Rust backend layering, file roles, naming, and database rules
- `docs/frontend-guide.md`: React routing, state, API organization, page rules, and UI constraints
- `docs/project-map.md`: file and entrypoint map for fast orientation

## Current Layout

- Backend lives in `server/`.
- Migrations live in `server/migrations/`.
- Frontend lives in `web/`.
- Root only keeps workspace metadata, docs, and command entry points.
- Root-level docs define the outer contract; subdirectory `AGENTS.md` files only provide local quick-entry rules.
- When working in a specific subproject, always read the nearest `AGENTS.md` in that directory before modifying files.
- Backend structure is `server/src/features/<feature>/` with `mod.rs + handler.rs + service.rs + repo.rs + types.rs`.
- Do not change the directory structure unless the docs and commands are updated in the same change.

## Working Rules

- Prefer the smallest change that solves the task.
- Prefer the smallest viable implementation (MVP) that solves the task.
- Do not stack extra abstractions, fallback branches, or compatibility code.
- Reuse existing code before introducing new code paths.
- Keep backend code in `server/src/infra/`, `server/src/common/`, `server/src/middleware/`, and `server/src/features/<feature>`.
- Keep each feature self-contained inside `server/src/features/<feature>/`.
- Keep generated files out of manual edits.
- Do not rename or move backend paths unless docs and commands are updated with the change.
- When API contracts or schema change, update backend, frontend, and docs together.
- Documentation must stay single-responsibility; do not reintroduce cross-linked domain docs inside `docs/`.
- Permission checks currently use `Require(...)` as the default mode.
- `Any(...)` and `All(...)` are reserved in code for future permission combinations and should not be used unless the feature explicitly needs them.
- For feature route handlers, use `list_users`, `get_user`, `create_user`, `update_user`, `delete_user` style names.
- When adding a new feature, create `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs` first, then wire routes from `mod.rs`.
- Keep row/entity definitions at the top of `types.rs`.
- Use `sqlx::FromRow` on response types when the database shape already matches the API shape closely enough; do not add a separate `model.rs` just for basic queries.
- Keep request/response/query types in `types.rs`.

## Commands

- Start backend and frontend separately; do not add a combined `just dev` target back.
- `just dev-server`: start backend
- `just dev-web`: start frontend
- `just check`: run backend check and frontend `vp lint`
- `just build`: build backend and frontend
