# Documentation Governance Rollout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Roll out the new documentation governance system so `rustzen-admin` has explicit homes for goals, plans, specs, and agent docs while keeping `AGENTS.md` files thin.

**Architecture:** Keep the existing guide documents in place and add the minimum new document surfaces needed for governance. Tighten root and subdirectory `AGENTS.md` files so they act as entrypoints, while stable rules live in guide docs and new repository-state documents live under `docs/goals/`, `docs/plans/`, `docs/specs/`, and `docs/agents/`.

**Tech Stack:** Markdown, repository docs, `AGENTS.md`, `README.md`, `rg`, `sed`, `git diff`

---

## File Structure

- Create: `docs/goals/product-direction.md`
- Create: `docs/agents/operating-rules.md`
- Create: `docs/agents/current-iteration.md`
- Modify: `AGENTS.md`
- Modify: `zen-server/AGENTS.md`
- Modify: `zen-web/AGENTS.md`
- Modify: `README.md`
- Modify: `docs/architecture.md`
- Reference only: `docs/specs/documentation-governance.md`
- Validate with: `find docs -maxdepth 2 -type f | sort`, `git diff --check`, `rg`

### Task 1: Seed The New Documentation Areas

**Files:**
- Create: `docs/goals/product-direction.md`
- Create: `docs/agents/operating-rules.md`
- Create: `docs/agents/current-iteration.md`
- Reference: `docs/specs/documentation-governance.md`

- [ ] **Step 1: Create `docs/goals/product-direction.md` with a stable product-direction baseline**

```md
# Product Direction

> Status: active baseline

## Positioning

`rustzen-admin` is an open-source Rust full-stack admin foundation.

It is not a vertical product and it is not a template dump. The repository exists to provide a clear, maintainable base for real admin systems with explicit backend, frontend, permission, and deployment boundaries.

## Target Outcome

- keep the monorepo small and understandable
- keep auth and permission reusable
- keep backend features self-contained
- keep frontend and backend contracts synchronized
- keep documentation usable as an engineering interface

## Non-Goals

- do not turn the repository into a multi-runtime platform unless the product scope requires it
- do not add compatibility layers or fallback paths to preserve old structure
- do not expand documentation into a heavy process system

## Repository Direction

- strengthen the repository as a clean admin foundation
- improve deployment and packaging clarity when needed
- grow feature depth without losing structural clarity
- keep documentation governance explicit as the codebase evolves
```

- [ ] **Step 2: Create `docs/agents/operating-rules.md` as the stable agent runtime contract**

```md
# Agent Operating Rules

> Stable operating rules for agent work in `rustzen-admin`

## Required Reading Order

1. `README.md`
2. `AGENTS.md`
3. the nearest subdirectory `AGENTS.md`
4. relevant guide docs in `docs/`
5. relevant files in `docs/goals/`, `docs/plans/`, `docs/specs/`, and `docs/agents/`

## Document Placement Rules

- put product direction in `docs/goals/`
- put work sequencing in `docs/plans/`
- put design and structural contracts in `docs/specs/`
- put stable agent rules in `docs/agents/operating-rules.md`
- put current execution state in `docs/agents/current-iteration.md`

## Working Rules

- keep `AGENTS.md` thin
- keep formal rules in `docs/`
- prefer the smallest viable documentation change
- update code, docs, and command references together when structure changes
- do not duplicate stable rules across multiple files
```

- [ ] **Step 3: Create `docs/agents/current-iteration.md` as the short-lived execution-state file**

```md
# Current Iteration

> Current repository documentation iteration state

## Focus

Roll out the documentation governance structure introduced by `docs/specs/documentation-governance.md`.

## In Scope

- tighten root `AGENTS.md`
- tighten `zen-server/AGENTS.md`
- tighten `zen-web/AGENTS.md`
- expose `docs/goals/`, `docs/plans/`, `docs/specs/`, and `docs/agents/`
- seed the first goal and agent documents

## Out Of Scope

- backend feature refactors
- frontend feature refactors
- deployment changes unrelated to documentation governance

## Exit Conditions

- the new docs areas exist
- entry documents point to them
- `AGENTS.md` responsibilities are clear and non-overlapping
```

- [ ] **Step 4: Verify the new seed files exist and are in the right places**

Run:

```bash
find docs -maxdepth 2 -type f | sort
```

Expected: output includes:

```txt
docs/agents/current-iteration.md
docs/agents/operating-rules.md
docs/goals/product-direction.md
docs/plans/documentation-governance-rollout.md
docs/specs/documentation-governance.md
```

- [ ] **Step 5: Commit the seeded documentation areas**

Run:

```bash
git add docs/goals/product-direction.md docs/agents/operating-rules.md docs/agents/current-iteration.md
git commit -m "docs: add documentation governance seed files"
```

Expected: commit succeeds and only the new seed files are included.

### Task 2: Tighten The Root `AGENTS.md`

**Files:**
- Modify: `AGENTS.md`
- Reference: `docs/specs/documentation-governance.md`
- Reference: `docs/goals/product-direction.md`
- Reference: `docs/agents/operating-rules.md`

- [ ] **Step 1: Replace the current root `AGENTS.md` shape with a thin repository entry contract**

Target structure:

```md
# Repository Guidelines

## Source of Truth

- `README.md`: developer entry point and document map
- `AGENTS.md`: repository-level collaboration rules and reading order
- `zen-server/AGENTS.md`: backend quick-entry guidance inside `zen-server/`
- `zen-web/AGENTS.md`: frontend quick-entry guidance inside `zen-web/`
- `docs/architecture.md`: repository layout, document layers, repo boundaries, and command summary
- `docs/backend-guide.md`: Rust backend layering, file roles, naming, and database rules
- `docs/frontend-guide.md`: React routing, state, API organization, page rules, and UI constraints
- `docs/deployment-guide.md`: production deployment rules, packaging, service startup, and runtime config requirements
- `docs/permission-guide.md`: current permission model and usage constraints
- `docs/project-map.md`: file and entrypoint map for fast orientation
- `docs/goals/product-direction.md`: product direction and repository intent
- `docs/plans/`: active planning documents
- `docs/specs/`: formal design and structure specs
- `docs/agents/operating-rules.md`: stable agent operating contract
- `docs/agents/current-iteration.md`: current documentation iteration state

## Reading Order

1. Read `README.md`.
2. Read `AGENTS.md`.
3. Read the nearest subdirectory `AGENTS.md`.
4. Read the relevant guide docs in `docs/`.
5. Read the relevant goal, plan, spec, or agent-state file only when the task needs it.

## Repository Boundaries

- Shared auth and permission capability code lives in `zen-core/`.
- Backend lives in `zen-server/`.
- Migrations live in `zen-server/migrations/`.
- Frontend lives in `zen-web/`.
- Deployment assets live in `deploy/`.
- Root keeps workspace metadata, docs, command entry points, and the shared crate.

## Working Rules

- prefer the smallest viable change
- do not add fallback or compatibility logic
- keep stable formal rules in `docs/`
- keep subdirectory `AGENTS.md` files thin
- update code, docs, and commands together when structure changes
```

- [ ] **Step 2: Remove rule duplication from root `AGENTS.md` where guide docs already own the detail**

Run:

```bash
rg -n "Require|sqlx::FromRow|list_users|types.rs|apiRequest" AGENTS.md
```

Expected: no matches, because detailed backend/frontend implementation rules should live in guide docs, not root `AGENTS.md`.

- [ ] **Step 3: Verify the new root `AGENTS.md` points to all four new documentation areas**

Run:

```bash
rg -n "docs/goals|docs/plans|docs/specs|docs/agents" AGENTS.md
```

Expected: all four areas are referenced.

- [ ] **Step 4: Commit the root `AGENTS.md` cleanup**

Run:

```bash
git add AGENTS.md
git commit -m "docs: tighten root agents contract"
```

Expected: commit succeeds and includes only the root `AGENTS.md` change.

### Task 3: Tighten The Subdirectory `AGENTS.md` Files

**Files:**
- Modify: `zen-server/AGENTS.md`
- Modify: `zen-web/AGENTS.md`
- Reference: `docs/backend-guide.md`
- Reference: `docs/frontend-guide.md`
- Reference: `docs/agents/operating-rules.md`

- [ ] **Step 1: Reduce `zen-server/AGENTS.md` to scope, quick entry, local highlights, local rules, and commands**

Target shape:

```md
# Backend Guide

## Scope

- Applies to `zen-server/`.
- Keep only quick local rules for work inside `zen-server/`.

## Quick Entry

- `docs/backend-guide.md`
- `docs/architecture.md`
- `docs/permission-guide.md`
- `docs/project-map.md`
- `docs/agents/operating-rules.md`

## Directory Highlights

- `zen-core/`
- `zen-server/src/features/`
- `zen-server/src/infra/`
- `zen-server/src/common/`
- `zen-server/src/middleware/`
- `zen-server/migrations/`

## Local Rules

- create `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, and `types.rs` first for new features
- keep SQL out of handlers
- keep repos inside feature boundaries
- use `Require(...)` as the default permission check

## Commands

- `cargo check -p server`
- `just dev-server`
- `just check`
- `just build-binary`
- `just build-release`
- `just build-image`
```

- [ ] **Step 2: Reduce `zen-web/AGENTS.md` to the same thin-entry pattern**

Target shape:

```md
# Frontend Guide

## Scope

- Applies to `zen-web/`.
- Keep only quick local rules for work inside `zen-web/`.

## Quick Entry

- `docs/frontend-guide.md`
- `docs/architecture.md`
- `docs/project-map.md`
- `docs/agents/operating-rules.md`

## Directory Highlights

- `zen-web/src/routes/`
- `zen-web/src/api/`
- `zen-web/src/components/`
- `zen-web/src/store/`
- `zen-web/src/main.tsx`
- `zen-web/src/routes/__root.tsx`

## Local Rules

- keep request logic in `zen-web/src/api/`
- keep pages thin
- do not edit generated files manually
- update API wrappers before page usage when contracts change

## Commands

- `cd zen-web && pnpm dev`
- `cd zen-web && pnpm build`
- `cd zen-web && vp lint`
- `cd zen-web && vp check --fix`
```

- [ ] **Step 3: Verify both subdirectory `AGENTS.md` files reference formal docs instead of duplicating them**

Run:

```bash
rg -n "docs/(backend-guide|frontend-guide|architecture|project-map|permission-guide|agents/operating-rules)" zen-server/AGENTS.md zen-web/AGENTS.md
```

Expected: both files contain quick-entry references to the formal docs they depend on.

- [ ] **Step 4: Verify both subdirectory `AGENTS.md` files do not contain active-iteration content**

Run:

```bash
rg -n "Current Iteration|In Scope|Out Of Scope|Exit Conditions|plan|roadmap" zen-server/AGENTS.md zen-web/AGENTS.md
```

Expected: no matches, because execution state belongs in `docs/agents/current-iteration.md` or `docs/plans/`.

- [ ] **Step 5: Commit the subdirectory `AGENTS.md` cleanup**

Run:

```bash
git add zen-server/AGENTS.md zen-web/AGENTS.md
git commit -m "docs: tighten subproject agent entry docs"
```

Expected: commit succeeds and includes only the two subdirectory `AGENTS.md` files.

### Task 4: Expose The New Documentation Structure In Repository Entry Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture.md`
- Reference: `docs/goals/product-direction.md`
- Reference: `docs/plans/documentation-governance-rollout.md`
- Reference: `docs/specs/documentation-governance.md`
- Reference: `docs/agents/operating-rules.md`
- Reference: `docs/agents/current-iteration.md`

- [ ] **Step 1: Update `README.md` so the documentation entry map includes the new docs areas**

Add entries in the documentation section for:

```md
- [docs/goals/product-direction.md](./docs/goals/product-direction.md): product direction and repository intent
- [docs/plans/documentation-governance-rollout.md](./docs/plans/documentation-governance-rollout.md): rollout plan for documentation governance
- [docs/specs/documentation-governance.md](./docs/specs/documentation-governance.md): formal documentation governance spec
- [docs/agents/operating-rules.md](./docs/agents/operating-rules.md): stable agent operating rules
- [docs/agents/current-iteration.md](./docs/agents/current-iteration.md): current documentation iteration state
```

- [ ] **Step 2: Update `docs/architecture.md` document layers so the new docs areas are part of the formal repository layout**

Add the new document areas to the document-layer section:

```md
- `docs/goals/`: product direction and long-lived repository goals
- `docs/plans/`: active and upcoming work planning
- `docs/specs/`: formal design and structure specifications
- `docs/agents/`: stable operating rules and current agent iteration state
```

Also update the top layout statement so `docs/` is no longer described only as technical specification documents.

- [ ] **Step 3: Verify repository entry docs expose the new structure**

Run:

```bash
rg -n "docs/goals|docs/plans|docs/specs|docs/agents" README.md docs/architecture.md
```

Expected: both files reference the new documentation areas.

- [ ] **Step 4: Run final documentation validation**

Run:

```bash
git diff --check -- AGENTS.md zen-server/AGENTS.md zen-web/AGENTS.md README.md docs/architecture.md docs/goals/product-direction.md docs/agents/operating-rules.md docs/agents/current-iteration.md
```

Expected: no whitespace or patch-format errors.

Run:

```bash
find docs -maxdepth 2 -type f | sort
```

Expected: the new directories and seed files appear in stable locations.

- [ ] **Step 5: Commit the documentation entrypoint updates**

Run:

```bash
git add README.md docs/architecture.md
git commit -m "docs: expose documentation governance structure"
```

Expected: commit succeeds and includes only the repository entry docs.

## Self-Review Checklist

- This plan covers all items required by `docs/specs/documentation-governance.md`.
- The plan does not rely on unspecified placeholder files.
- The plan keeps `AGENTS.md` thin and routes stable detail into `docs/`.
- The rollout seeds all four new document areas:
  - `docs/goals/`
  - `docs/plans/`
  - `docs/specs/`
  - `docs/agents/`
