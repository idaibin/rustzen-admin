# Documentation Governance Spec

> Status: proposed baseline for repository documentation governance
>
> Scope: `rustzen-admin` root docs, root `AGENTS.md`, `zen-server/AGENTS.md`, and `zen-web/AGENTS.md`

## Goal

Create a medium-weight documentation system for `rustzen-admin` that keeps:

- repository entry rules clear
- product and repository goals explicit
- plans separate from specs
- agent operating rules separate from current execution state
- subdirectory `AGENTS.md` files small and stable

The result should support ongoing repository cleanup, feature growth, and future optimization without turning the repo into a process-heavy documentation system.

## Non-Goals

- Do not introduce a heavy evidence/archive workflow like `zen-clear`.
- Do not duplicate the same rules across root docs, subdirectory docs, and `AGENTS.md`.
- Do not move business implementation guidance out of the existing domain guides unless needed.
- Do not create a second documentation system outside `docs/`.

## Current Problems

### Root problems

- `docs/` currently focuses on technical guide documents only.
- There is no stable home for product goals, roadmap-style plans, or agent execution state.
- The repository has no explicit contract for where future planning documents should live.

### `AGENTS.md` problems

- Root `AGENTS.md` carries both repository rules and document-map responsibilities.
- Subdirectory `AGENTS.md` files already try to stay thin, but there is no repository-wide rule that defines exactly what belongs there versus in `docs/`.
- Future growth risks duplicating rules between root and subdirectory `AGENTS.md` files.

### Lifecycle problems

- Specs, plans, goals, and agent state are not yet separated by purpose.
- The repo has no clear rule for when a document belongs in `docs/specs/` versus a repository guide such as `docs/backend-guide.md`.

## Design Principles

- Keep `AGENTS.md` thin.
- Keep `docs/` as the single formal documentation system.
- Separate stable rules from changeable execution state.
- Separate goals from plans and plans from specs.
- Prefer adding explicit homes for future documents over rewriting every existing doc.
- Preserve the current repo strength: clarity, not bureaucracy.

## Proposed Documentation Layout

The repository keeps the current guide documents and adds four bounded document areas:

```txt
docs/
├── goals/
├── plans/
├── specs/
├── agents/
├── architecture.md
├── backend-guide.md
├── deployment-guide.md
├── frontend-guide.md
├── permission-guide.md
├── project-map.md
└── repository-comparison.md
```

## Responsibilities By Area

### `docs/goals/`

Purpose:

- product positioning
- repository direction
- medium- to long-lived goals
- scope boundaries

Typical documents:

- product vision
- repository evolution goals
- target audience and non-goals

Rules:

- goals describe desired direction, not implementation detail
- goals must not become execution logs or feature checklists

### `docs/plans/`

Purpose:

- active and upcoming work planning
- phased repository cleanup plans
- scoped implementation plans derived from approved specs

Typical documents:

- documentation migration plan
- repository cleanup plan
- feature implementation plan

Rules:

- plans describe work sequencing and delivery slices
- plans may reference specs, but must not replace specs
- plans must not become the source of stable architecture rules

### `docs/specs/`

Purpose:

- formal design decisions
- structural contracts
- implementation-level specifications for bounded changes

Typical documents:

- documentation governance spec
- auth model refactor spec
- deployment packaging spec

Rules:

- specs define what the system should look like
- specs can include migration strategy, but not daily execution state
- specs should be the basis for later plan documents

### `docs/agents/`

Purpose:

- agent execution rules
- current agent-facing repository state

This directory is intentionally split into two document types:

- stable operating rules
- current iteration state

Recommended files:

- `docs/agents/operating-rules.md`
- `docs/agents/current-iteration.md`

Rules:

- `operating-rules.md` is stable and procedural
- `current-iteration.md` is short-lived and current-state only
- `current-iteration.md` must not turn into a historical log
- agent state does not replace plans or specs

## Role Of Repository Guide Documents

The existing top-level guide documents remain the long-lived formal guides:

- `docs/architecture.md`
- `docs/backend-guide.md`
- `docs/frontend-guide.md`
- `docs/deployment-guide.md`
- `docs/permission-guide.md`
- `docs/project-map.md`

These files continue to define stable rules and indexes. They should not become dumping grounds for plans, active tasks, or temporary iteration notes.

## `AGENTS.md` Contract

### Root `AGENTS.md`

Root `AGENTS.md` should contain only:

- source-of-truth map
- repository-wide working rules
- repository layout summary
- required reading order
- boundaries for what belongs in subdirectory `AGENTS.md`

Root `AGENTS.md` should not contain:

- detailed backend implementation rules already covered by `docs/backend-guide.md`
- detailed frontend implementation rules already covered by `docs/frontend-guide.md`
- active task tracking
- repository plans

### Subdirectory `AGENTS.md`

`zen-server/AGENTS.md` and `zen-web/AGENTS.md` should contain only:

- scope statement
- quick entry links to the relevant formal docs
- local directory highlights
- the smallest set of local rules needed before editing
- command entrypoints for that subproject

Subdirectory `AGENTS.md` should not contain:

- full copies of repository rules
- plans or execution state
- large design decisions that belong in `docs/specs/`

## Reading Order Contract

Expected reading order for an agent working in this repo:

1. root `README.md`
2. root `AGENTS.md`
3. relevant subdirectory `AGENTS.md`
4. relevant guide documents in `docs/`
5. relevant file in `docs/goals/`, `docs/plans/`, `docs/specs/`, or `docs/agents/` when the task requires it

## Naming Rules

### Goals

- concise durable names
- examples:
  - `docs/goals/product-direction.md`
  - `docs/goals/repository-evolution.md`

### Plans

- scope- and content-based names
- examples:
  - `docs/plans/documentation-governance-rollout.md`
  - `docs/plans/admin-foundation-phase-1-rollout.md`

### Specs

- capability- and content-based names
- examples:
  - `docs/specs/documentation-governance.md`
  - `docs/specs/auth-permission-simplification.md`

### Agents

- stable fixed names where possible
- examples:
  - `docs/agents/operating-rules.md`
  - `docs/agents/current-iteration.md`

## Ownership Rules

- Stable repository rules belong in guide docs or root `AGENTS.md`.
- Product direction belongs in `docs/goals/`.
- Work sequencing belongs in `docs/plans/`.
- Design intent and structural contracts belong in `docs/specs/`.
- Agent operating rules and current execution state belong in `docs/agents/`.

If the same information appears in multiple places, one location must be declared the source of truth and the others must become links or summaries.

## Initial Rollout

The first rollout should be small and ordered:

1. define the documentation contract
2. add missing directories under `docs/`
3. tighten root `AGENTS.md`
4. tighten `zen-server/AGENTS.md` and `zen-web/AGENTS.md`
5. add initial goal, plan, and agent docs
6. update `README.md` and `docs/architecture.md` to expose the new structure

## Success Criteria

- A contributor can tell where to place a new document without guessing.
- Root `AGENTS.md` stays short and repository-wide.
- Subdirectory `AGENTS.md` files stay local and thin.
- Plans, specs, goals, and agent state no longer mix in the same file.
- Existing guide documents remain stable and single-purpose.

## Open Choices Resolved By This Spec

- Repository docs remain in English.
- Communication with maintainers can still happen in Chinese.
- `AGENTS.md` remains a thin entry layer, not a full manual.
- `docs/agents/` contains both stable rules and current iteration state, but in separate files.

## Follow-Up

The next artifact after this spec should be a rollout plan in `docs/plans/` that turns this structure into concrete file changes.
