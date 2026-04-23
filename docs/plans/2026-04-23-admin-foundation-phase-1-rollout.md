# Admin Foundation Phase 1 Rollout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Phase 1 foundation spec into a staged implementation program that reshapes the current admin shell into a reusable five-group foundation: `identity`, `access`, `audit`, `system`, and `runtime`.

**Architecture:** Keep the existing repository shape and migrate capability ownership in layers. Start with documentation and boundary cleanup, then split backend ownership, then align frontend APIs and routes, and finally close the first reusable baseline with minimal feature slices in priority order.

**Tech Stack:** Markdown, Rust backend features, React frontend routes and API modules, `AGENTS.md`, repository guide docs, `rg`, `find`, `git diff`

---

## File Structure

- Create: `docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md`
- Modify: `docs/README.md`
- Modify: `README.md`
- Modify: `docs/agents/current-iteration.md`
- Reference: `docs/specs/2026-04-23-admin-foundation-phase-1.md`
- Reference: `docs/specs/2026-04-22-documentation-governance.md`
- Reference: `docs/project-map.md`
- Reference: `zen-server/src/features/`
- Reference: `zen-web/src/routes/`
- Reference: `zen-web/src/api/`

## Current-State Mapping

Current backend feature ownership:

- `auth/`
- `dashboard/`
- `system/`

Current frontend route ownership:

- `login`
- `index`
- `system/dict`
- `system/log`
- `system/menu`
- `system/role`
- `system/user`

Current frontend API ownership:

- `auth/`
- `dashboard/`
- `system/`

Phase 1 target ownership:

- `identity`
- `access`
- `audit`
- `system`
- `runtime`

This means the rollout must explicitly address:

- how `auth` maps into `identity`
- how current `system/*` subfeatures split between `access`, `audit`, and `system`
- how current frontend `system/*` pages and APIs are regrouped without adding speculative capability layers

## Current Migration Findings

### Backend Findings

- `zen-server/src/features/auth/` already contains the Phase 1 `identity` baseline for login, logout, current user, and avatar update.
- `AuthService::get_login_info()` currently returns permission-bearing session data, which means the current `auth` area still exposes `access` output even though the capability owner should become `identity`.
- `zen-server/src/features/auth/handler.rs` records login events through `features::system::log::LogService`, so login flow currently crosses from `identity` into `audit`.
- `zen-server/src/features/system/mod.rs` is a mixed ownership aggregator and should not remain the long-term owner for `menu`, `role`, `log`, and access-facing user management.
- `zen-server/src/features/system/user/` is a boundary hotspot: it owns admin-side user CRUD, role assignment, status changes, and password reset. These are not self-service identity capabilities and should not move into the `identity` group.
- There is no dedicated self-profile update or self-password change endpoint yet beyond avatar upload, so the `identity` baseline is only partially complete.

### Frontend Findings

- `zen-web/src/api/auth/api.ts` only covers login, logout, and current-user fetch, which aligns with the minimum identity session surface.
- `zen-web/src/routes/login.tsx` is the existing `identity` entrypoint and can remain the starting point for the frontend identity slice.
- There is no dedicated profile route group yet under `zen-web/src/routes/profile/`.
- `zen-web/src/routes/system/user.tsx` is an administrative user-management page with role assignment, password reset, status changes, and delete actions. It is not a self-service identity page and should stay out of the first `identity` slice.
- `zen-web/src/api/system/user/api.ts` is also admin-facing and should not be reused as the Phase 1 `identity` API surface.

## Status Snapshot

- done: expose the Phase 1 foundation spec and rollout plan in repository entry docs
- done: add backend and frontend capability alignment to `docs/project-map.md`
- done: confirm the current codebase still maps cleanly into the five approved capability groups
- done: write the dedicated `identity` child spec
- done: write the dedicated `access` child spec
- done: turn the identity findings into an implementation checklist before code refactors begin
- done: define the `access` ownership contract for administrative user management
- next: derive concrete `system/user` split checklists for backend and frontend

### Task 1: Expose The Phase 1 Plan In Documentation Entry Points

**Files:**
- Modify: `docs/README.md`
- Modify: `README.md`
- Modify: `docs/agents/current-iteration.md`
- Create: `docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md`

- [ ] **Step 1: Add the new rollout plan to `docs/README.md`**

Update the `Plans` section so it includes:

```md
### Plans

Sequencing and delivery planning:

- `plans/2026-04-22-documentation-governance-rollout.md`
- `plans/2026-04-23-admin-foundation-phase-1-rollout.md`
```

- [ ] **Step 2: Add the new Phase 1 spec and plan to the root `README.md` documentation entry list**

Add entries such as:

```md
- [docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md](./docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md): rollout plan for the first admin foundation capability phase
- [docs/specs/2026-04-23-admin-foundation-phase-1.md](./docs/specs/2026-04-23-admin-foundation-phase-1.md): Phase 1 product-capability spec for the admin foundation
```

- [ ] **Step 3: Update `docs/agents/current-iteration.md` so the current documentation focus includes Phase 1 foundation planning**

Add or adjust content so it clearly states:

```md
## In Scope

- phase-1 product-capability planning for `identity`, `access`, `audit`, `system`, and `runtime`
- rollout planning for turning the current admin shell into a reusable foundation
```

- [ ] **Step 4: Verify the new plan is discoverable from repository entry docs**

Run:

```bash
rg -n "2026-04-23-admin-foundation-phase-1" README.md docs/README.md docs/agents/current-iteration.md
```

Expected: matches exist in the root `README.md`, `docs/README.md`, and the current-iteration doc.

- [ ] **Step 5: Commit the documentation-entry updates**

Run:

```bash
git add README.md docs/README.md docs/agents/current-iteration.md docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
git commit -m "docs: add phase 1 foundation rollout plan"
```

Expected: commit succeeds and only Phase 1 planning entry docs are included.

### Task 2: Produce The Backend Capability Migration Map

**Files:**
- Modify: `docs/project-map.md`
- Reference: `docs/specs/2026-04-23-admin-foundation-phase-1.md`
- Reference: `zen-server/src/features/auth/`
- Reference: `zen-server/src/features/system/`
- Reference: `zen-server/src/features/dashboard/`

- [ ] **Step 1: Add a Phase 1 foundation capability section to `docs/project-map.md`**

The new section should map current ownership to target ownership:

```md
## Phase 1 Capability Map

- `identity`: currently starts from `zen-server/src/features/auth/`
- `access`: currently starts from `zen-server/src/features/system/menu/`, `role/`, and permission-linked user access paths
- `audit`: currently starts from `zen-server/src/features/system/log/`
- `system`: currently starts from `zen-server/src/features/system/dict/` and future config ownership
- `runtime`: currently has no dedicated top-level feature and will be introduced as a new group
```

- [ ] **Step 2: Document the required backend split decisions inline in the plan**

Keep this migration map explicit:

```md
- `auth` -> `identity`
- `system/menu` + `system/role` + access-facing parts of `system/user` -> `access`
- `system/log` -> `audit`
- `system/dict` + future config -> `system`
- new file/resource capability -> `runtime`
```

- [ ] **Step 3: Verify the backend migration map does not introduce business-domain modules**

Run:

```bash
rg -n "tenant|organization|project|workflow|approval|order|content" docs/project-map.md docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
```

Expected: no Phase 1 migration map should introduce those business-domain modules as implementation targets.

- [ ] **Step 4: Commit the backend capability-mapping documentation**

Run:

```bash
git add docs/project-map.md docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
git commit -m "docs: map backend foundation capability groups"
```

Expected: commit succeeds and includes only the mapping updates.

### Task 3: Define The Frontend Capability Realignment

**Files:**
- Modify: `docs/project-map.md`
- Reference: `docs/specs/2026-04-23-admin-foundation-phase-1.md`
- Reference: `zen-web/src/routes/`
- Reference: `zen-web/src/api/`

- [ ] **Step 1: Add a frontend capability-alignment section to `docs/project-map.md`**

The new section should define the target shape:

```md
## Phase 1 Frontend Alignment

- `identity`: `zen-web/src/api/identity/`, `zen-web/src/routes/profile/`
- `access`: `zen-web/src/api/access/`, `zen-web/src/routes/access/`
- `audit`: `zen-web/src/api/audit/`, `zen-web/src/routes/audit/`
- `system`: `zen-web/src/api/system/`, `zen-web/src/routes/system/`
- `runtime`: `zen-web/src/api/runtime/`, `zen-web/src/routes/runtime/`
```

- [ ] **Step 2: Record the initial route/API regrouping rules in the plan**

Document these rules:

```md
- current `auth` frontend API maps into `identity`
- current `system/menu` and `system/role` pages and APIs map into `access`
- current `system/log` maps into `audit`
- current `system/dict` stays in `system`
- current `dashboard` remains outside the first regrouping wave unless needed by a Phase 1 slice
```

- [ ] **Step 3: Verify the alignment still keeps frontend ownership under capability groups rather than technical layers**

Run:

```bash
rg -n "identity|access|audit|system|runtime" docs/project-map.md docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
```

Expected: the capability groups are present as ownership units in both the map and the plan.

- [ ] **Step 4: Commit the frontend capability-alignment documentation**

Run:

```bash
git add docs/project-map.md docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
git commit -m "docs: define frontend foundation capability alignment"
```

Expected: commit succeeds and includes only the frontend-alignment documentation.

### Task 4: Define The First Implementation Slices

**Files:**
- Modify: `docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md`
- Reference: `docs/specs/2026-04-23-admin-foundation-phase-1.md`

- [ ] **Step 1: Define the first backend-first implementation slice**

Record the first slice as:

```md
### Slice 1: Identity Baseline

- move product-capability ownership from `auth` to `identity`
- preserve existing login flow while renaming the capability boundary
- add self-profile and self-password endpoints if missing
- keep the slice free of role and permission management changes
```

- [ ] **Step 2: Define the second implementation slice**

Record the second slice as:

```md
### Slice 2: Access Baseline

- split role/menu/permission ownership out of the catch-all `system` area
- make `access` the explicit owner of menus, roles, and permission-facing assignment paths
- keep data-scope and policy-engine work out of scope
```

- [ ] **Step 3: Define the remaining Phase 1 slices**

Record the remaining slices as:

```md
### Slice 3: Audit Baseline

- lift login and operation logs into `audit`

### Slice 4: System Baseline

- keep dictionaries in `system`
- add system-config ownership
- expose stable options endpoints

### Slice 5: Runtime Baseline

- introduce `runtime` for file upload and file metadata
```

- [ ] **Step 4: Verify the slice order still matches the approved priority**

Run:

```bash
rg -n "Slice 1|Slice 2|Slice 3|Slice 4|Slice 5|identity|access|audit|system|runtime" docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
```

Expected: the slice order reflects `identity -> access -> audit -> system -> runtime`.

- [ ] **Step 5: Commit the implementation-slice plan**

Run:

```bash
git add docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md
git commit -m "docs: define phase 1 foundation implementation slices"
```

Expected: commit succeeds and includes only the rollout-plan updates.

## Slice Readiness Notes

### Slice 1: Identity Baseline

- keep login, logout, current user, and avatar handling as the starting baseline
- add missing self-profile update and self-password change capabilities
- keep admin user CRUD, status changes, role assignment, and password reset outside this slice
- treat login logging as an `audit` dependency, not an `identity` ownership reason

### Slice 2: Access Baseline

- extract role, menu, and permission-facing user-management ownership from the catch-all `system` area
- keep administrative user lifecycle actions with `access`, not `identity`
- preserve the current permission cache and visible-menu behavior while moving capability ownership

## Self-Review Checklist

- The plan preserves the five approved feature groups: `identity`, `access`, `audit`, `system`, `runtime`.
- The plan does not reintroduce business-domain modules into Phase 1.
- The plan keeps the current repository shape and avoids new top-level apps or crates.
- The plan defines explicit migration mappings from current `auth` and `system/*` ownership into Phase 1 capability groups.
- The plan establishes implementation slices in the approved priority order.
