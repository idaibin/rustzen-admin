# AI Coding Rules

This is a current guide for AI-assisted contributions.

## Current Truth

- Source of truth for implementation decisions remains:
  - source code
  - `docs/architecture.md`
  - `docs/guides/*` (especially `docs/guides/ai-coding-rules.md`)
- `docs/history/` remains historical input, not active implementation truth.

## Working Rules

- Prefer the smallest meaningful change that matches the current task.
- Prefer existing modules and abstractions over speculative layers.
- Do not add compatibility fallback logic unless there is a user-facing migration requirement.
- Keep `docs/architecture.md` and command surface (`justfile`) aligned with structure changes.
- Keep module ownership boundaries explicit and stable.
- Update code and docs together for API, config, or schema changes.

## Ownership Notes

- `crates/` owns shared Rust capability, auth, and storage helpers.
- `apps/server/` owns backend runtime and features.
- `apps/web/` owns frontend routes, API modules, and static runtime-facing behavior.
- `deploy/` owns packaging and release topology.

## Verification Rule

- Before marking a task as complete, run the task's verification command and then `git diff --check`.

## Phase Scope

- Desktop, plugins, sync, PostgreSQL provider support, and enterprise deployment are outside the sqlite-first phase scope.
