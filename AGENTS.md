# Repository Guidelines

## Source of Truth

- `docs/architecture.md`: layout, module shape, naming, and command summary
- `README.md`: developer entry points and common commands
- `web/AGENTS.md`: frontend-specific guidance

## Current Layout

- Backend lives in `server/` for now.
- Migrations live in `server/migrations/`.
- Frontend lives in `web/`.
- Target layout is `server/` + `web/`.
- Treat this as a monorepo in migration.
- Root only keeps workspace metadata, docs, and command entry points.
- Backend target structure is `server/src/features/<feature>/` with `mod.rs + handler.rs + service.rs + repo.rs + types.rs`.
- Do not change the directory structure unless the docs and commands are updated in the same change.

## Working Rules

- Prefer the smallest change that solves the task.
- Keep backend code in `server/src/infra/`, `server/src/common/`, `server/src/middleware/`, and `server/src/features/<feature>`.
- Keep each feature self-contained inside `server/src/features/<feature>/`.
- Keep generated files out of manual edits.
- Do not rename or move backend paths unless docs and commands are updated with the change.
- When API contracts or schema change, update backend, frontend, and docs together.
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
- `just check`: run backend check and frontend lint
- `just build`: build backend and frontend
