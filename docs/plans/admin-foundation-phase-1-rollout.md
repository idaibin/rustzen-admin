# Admin Foundation Phase 1 Rollout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Phase 1 foundation spec into a staged implementation program that reshapes the current admin shell into a reusable five-group foundation: `identity`, `access`, `audit`, `system`, and `runtime`.

**Architecture:** Keep the existing repository shape and migrate capability ownership in layers. Start with documentation and boundary cleanup, then split backend ownership, then align frontend APIs and routes, and finally close the first reusable baseline with minimal feature slices in priority order.

**Tech Stack:** Markdown, Rust backend features, React frontend routes and API modules, `AGENTS.md`, repository guide docs, `rg`, `find`, `git diff`

---

## File Structure

- Create: `docs/plans/admin-foundation-phase-1-rollout.md`
- Modify: `docs/README.md`
- Modify: `README.md`
- Modify: `docs/agents/current-iteration.md`
- Reference: `docs/specs/admin-foundation-phase-1.md`
- Reference: `docs/specs/documentation-governance.md`
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
- `zen-web/src/routes/system/role.tsx` and `zen-web/src/routes/system/menu.tsx` are already explicit access-control pages and should regroup with admin user management under the future `access` route group.

## Status Snapshot

- done: expose the Phase 1 foundation spec and rollout plan in repository entry docs
- done: add backend and frontend capability alignment to `docs/project-map.md`
- done: confirm the current codebase still maps cleanly into the five approved capability groups
- done: write the dedicated `identity` child spec
- done: write the dedicated `access` child spec
- done: turn the identity findings into an implementation checklist before code refactors begin
- done: define the `access` ownership contract for administrative user management
- done: derive concrete `system/user` split checklists for backend and frontend
- done: decide whether admin user management stays inside the `access` slice or becomes a dedicated later slice
- done: formalize the `audit` child spec before `system` and `runtime`
- done: derive the concrete backend audit ownership checklist from `system/log` and login logging
- done: formalize the `system` child spec before `runtime`
- done: decide whether Slice 4 needs an explicit system ownership subsection
- done: decide that `runtime` is ready to be formalized without another focused audit
- done: formalize the `runtime` child spec and decide whether Slice 5 needs an explicit runtime ownership subsection
- done: complete the Phase 1 documentation loop for all five capability groups

## Administrative User Management Decision

Phase 1 should keep administrative user management inside the `access` slice.

Reasoning:

- the current `system/user` module is access-facing because create, update, status change, password reset, and delete all operate under admin permissions
- the frontend page is already governed by access-control codes and role assignment workflows
- splitting admin user management into a separate slice now would add another ownership boundary before the repository has even completed `identity` and `access`
- the cleaner boundary is `self-service user state -> identity` and `administrator-managed access state -> access`

This means Phase 1 should not introduce a separate `user-admin` capability group.

## Next Documentation Slice Decision

The next child spec should formalize `audit` first.

Reasoning:

- `zen-server/src/features/system/log/` already exists as a bounded backend capability
- `zen-web/src/routes/system/log.tsx` already exists as a bounded frontend surface
- the current login flow already records `AUTH_LOGIN` through the log service, so `identity` currently depends on `audit`
- `system` still needs clearer expansion around config ownership
- `runtime` does not yet have a dedicated top-level feature or frontend surface, so its spec would be more speculative right now

This sets the next documentation order as:

1. `audit`
2. `system`
3. `runtime`

## Post-Audit Documentation Decision

After `audit`, the next child spec should formalize `system` before `runtime`.

Reasoning:

- `zen-server/src/features/system/dict/` already exists as a bounded backend module
- `zen-web/src/routes/system/dict.tsx` and `zen-web/src/api/system/dict/` already exist as bounded frontend surfaces
- the Phase 1 `system` scope already has a concrete current anchor in dictionary management even before config ownership is expanded
- `runtime` is still mostly implicit: current file handling is concentrated in avatar upload helpers and static file serving, not in a dedicated top-level capability
- writing `runtime` first would force a more speculative contract than `system`

This sets the next documentation order after `audit` as:

1. `system`
2. `runtime`

## Runtime Readiness Decision

`runtime` is ready to be formalized now. It does not need another focused current-state audit first.

Reasoning:

- the current backend already has concrete runtime anchors in `common/files.rs`, runtime path helpers in `infra/config.rs`, and static resource serving in `infra/app.rs`
- the current product surface already exposes avatar upload as a file-handling path rather than a purely internal helper
- the current `/resources` prefix and avatar/static directory wiring are enough to define a minimal Phase 1 runtime contract
- `runtime` is still thinner than the other child specs, but the remaining uncertainty is design scope, not discovery scope

This means the next smallest documentation task after `system` is to write the `runtime` baseline spec.

## Concrete Split Checklist

### Backend `system/user` Split

- move `list_users` into the future `access` owner
- move `create_user` into `access`
- move `update_user` into `access`
- move `update_user_status` into `access`
- move `update_user_password` into `access` as an admin reset path
- move `delete_user` into `access`
- move `get_user_options` and `get_user_status_options` into `access`
- keep future self-profile and self-password paths out of this split

### Frontend Regrouping

- move `zen-web/src/routes/system/role.tsx` into the future `access` route group
- move `zen-web/src/routes/system/menu.tsx` into the future `access` route group
- move `zen-web/src/routes/system/user.tsx` into the future `access` route group
- move `zen-web/src/api/system/role/` into the future `access` API namespace
- move `zen-web/src/api/system/menu/` into the future `access` API namespace
- move `zen-web/src/api/system/user/` into the future `access` API namespace
- keep self-profile and self-password UI under the future `identity` route group

### Task 1: Expose The Phase 1 Plan In Documentation Entry Points

**Files:**
- Modify: `docs/README.md`
- Modify: `README.md`
- Modify: `docs/agents/current-iteration.md`
- Create: `docs/plans/admin-foundation-phase-1-rollout.md`

- [x] **Step 1: Add the new rollout plan to `docs/README.md`**

Update the `Plans` section so it includes:

```md
### Plans

Sequencing and delivery planning:

- `plans/documentation-governance-rollout.md`
- `plans/admin-foundation-phase-1-rollout.md`
```

- [x] **Step 2: Add the new Phase 1 spec and plan to the root `README.md` documentation entry list**

Add entries such as:

```md
- [docs/plans/admin-foundation-phase-1-rollout.md](./docs/plans/admin-foundation-phase-1-rollout.md): rollout plan for the first admin foundation capability phase
- [docs/specs/admin-foundation-phase-1.md](./docs/specs/admin-foundation-phase-1.md): Phase 1 product-capability spec for the admin foundation
```

- [x] **Step 3: Update `docs/agents/current-iteration.md` so the current documentation focus includes Phase 1 foundation planning**

Add or adjust content so it clearly states:

```md
## In Scope

- phase-1 product-capability planning for `identity`, `access`, `audit`, `system`, and `runtime`
- rollout planning for turning the current admin shell into a reusable foundation
```

- [x] **Step 4: Verify the new plan is discoverable from repository entry docs**

Run:

```bash
rg -n "admin-foundation-phase-1" README.md docs/README.md docs/agents/current-iteration.md
```

Expected: matches exist in the root `README.md`, `docs/README.md`, and the current-iteration doc.

- [ ] **Step 5: Commit the documentation-entry updates**

Run:

```bash
git add README.md docs/README.md docs/agents/current-iteration.md docs/plans/admin-foundation-phase-1-rollout.md
git commit -m "docs: add phase 1 foundation rollout plan"
```

Expected: commit succeeds and only Phase 1 planning entry docs are included.

### Task 2: Produce The Backend Capability Migration Map

**Files:**
- Modify: `docs/project-map.md`
- Reference: `docs/specs/admin-foundation-phase-1.md`
- Reference: `zen-server/src/features/auth/`
- Reference: `zen-server/src/features/system/`
- Reference: `zen-server/src/features/dashboard/`

- [x] **Step 1: Add a Phase 1 foundation capability section to `docs/project-map.md`**

The new section should map current ownership to target ownership:

```md
## Phase 1 Capability Map

- `identity`: currently starts from `zen-server/src/features/auth/`
- `access`: currently starts from `zen-server/src/features/system/menu/`, `role/`, and permission-linked user access paths
- `audit`: currently starts from `zen-server/src/features/system/log/`
- `system`: currently starts from `zen-server/src/features/system/dict/` and future config ownership
- `runtime`: currently has no dedicated top-level feature and will be introduced as a new group
```

- [x] **Step 2: Document the required backend split decisions inline in the plan**

Keep this migration map explicit:

```md
- `auth` -> `identity`
- `system/menu` + `system/role` + access-facing parts of `system/user` -> `access`
- `system/log` -> `audit`
- `system/dict` + future config -> `system`
- new file/resource capability -> `runtime`
```

- [x] **Step 3: Verify the backend migration map does not introduce business-domain modules**

Run:

```bash
rg -n "tenant|organization|project|workflow|approval|order|content" docs/project-map.md docs/plans/admin-foundation-phase-1-rollout.md
```

Expected: no Phase 1 migration map should introduce those business-domain modules as implementation targets.

- [ ] **Step 4: Commit the backend capability-mapping documentation**

Run:

```bash
git add docs/project-map.md docs/plans/admin-foundation-phase-1-rollout.md
git commit -m "docs: map backend foundation capability groups"
```

Expected: commit succeeds and includes only the mapping updates.

### Task 3: Define The Frontend Capability Realignment

**Files:**
- Modify: `docs/project-map.md`
- Reference: `docs/specs/admin-foundation-phase-1.md`
- Reference: `zen-web/src/routes/`
- Reference: `zen-web/src/api/`

- [x] **Step 1: Add a frontend capability-alignment section to `docs/project-map.md`**

The new section should define the target shape:

```md
## Phase 1 Frontend Alignment

- `identity`: `zen-web/src/api/identity/`, `zen-web/src/routes/profile/`
- `access`: `zen-web/src/api/access/`, `zen-web/src/routes/access/`
- `audit`: `zen-web/src/api/audit/`, `zen-web/src/routes/audit/`
- `system`: `zen-web/src/api/system/`, `zen-web/src/routes/system/`
- `runtime`: `zen-web/src/api/runtime/`, `zen-web/src/routes/runtime/`
```

- [x] **Step 2: Record the initial route/API regrouping rules in the plan**

Document these rules:

```md
- current `auth` frontend API maps into `identity`
- current `system/menu` and `system/role` pages and APIs map into `access`
- current `system/log` maps into `audit`
- current `system/dict` stays in `system`
- current `dashboard` remains outside the first regrouping wave unless needed by a Phase 1 slice
```

- [x] **Step 3: Verify the alignment still keeps frontend ownership under capability groups rather than technical layers**

Run:

```bash
rg -n "identity|access|audit|system|runtime" docs/project-map.md docs/plans/admin-foundation-phase-1-rollout.md
```

Expected: the capability groups are present as ownership units in both the map and the plan.

- [ ] **Step 4: Commit the frontend capability-alignment documentation**

Run:

```bash
git add docs/project-map.md docs/plans/admin-foundation-phase-1-rollout.md
git commit -m "docs: define frontend foundation capability alignment"
```

Expected: commit succeeds and includes only the frontend-alignment documentation.

### Task 4: Define The First Implementation Slices

**Files:**
- Modify: `docs/plans/admin-foundation-phase-1-rollout.md`
- Reference: `docs/specs/admin-foundation-phase-1.md`

- [x] **Step 1: Define the first backend-first implementation slice**

Record the first slice as:

```md
### Slice 1: Identity Baseline

- move product-capability ownership from `auth` to `identity`
- preserve existing login flow while renaming the capability boundary
- add self-profile and self-password endpoints if missing
- keep the slice free of role and permission management changes
```

- [x] **Step 2: Define the second implementation slice**

Record the second slice as:

```md
### Slice 2: Access Baseline

- split role/menu/permission ownership out of the catch-all `system` area
- make `access` the explicit owner of menus, roles, and permission-facing assignment paths
- keep data-scope and policy-engine work out of scope
```

- [x] **Step 3: Define the remaining Phase 1 slices**

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

- [x] **Step 4: Verify the slice order still matches the approved priority**

Run:

```bash
rg -n "Slice 1|Slice 2|Slice 3|Slice 4|Slice 5|identity|access|audit|system|runtime" docs/plans/admin-foundation-phase-1-rollout.md
```

Expected: the slice order reflects `identity -> access -> audit -> system -> runtime`.

- [ ] **Step 5: Commit the implementation-slice plan**

Run:

```bash
git add docs/plans/admin-foundation-phase-1-rollout.md
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

#### Admin User Management Inside Access

- keep administrator-managed user creation inside `access`
- keep user-role assignment inside `access`
- keep administrator-triggered password reset inside `access`
- keep status change and delete actions inside `access`
- keep self-profile and self-password flows out of this slice

### Slice 3: Audit Baseline

- move log ownership out of the generic `system` area
- make `audit` the explicit owner of login and operation logs
- preserve the current list and export surfaces while changing capability ownership

#### Audit Ownership Inside Slice 3

- keep `AUTH_LOGIN` recording as an `audit` responsibility even when triggered by `identity`
- keep structured log-write contracts inside `audit`
- keep log list and export behavior inside `audit`
- keep permission decisions and workflow history out of this slice

### Slice 4: System Baseline

- keep dictionary ownership inside the `system` capability
- reserve product-facing configuration ownership for `system`
- keep support-data contracts explicit while avoiding another catch-all bucket

#### System Ownership Inside Slice 4

- keep dictionary CRUD and lookup behavior inside `system`
- keep option-source behavior inside `system`
- keep future product-facing system configuration inside `system`
- keep access, audit, identity, and runtime ownership out of this slice

### Slice 5: Runtime Baseline

- keep file-resource handling inside the `runtime` capability
- keep upload and resource-path conventions explicit
- avoid turning `runtime` into a generic infrastructure bucket

#### Runtime Ownership Inside Slice 5

- keep reusable file upload handling inside `runtime`
- keep file validation rules inside `runtime`
- keep resource-prefix and file-path conventions inside `runtime`
- keep avatar upload as a consumer of runtime, not the owner of it
- keep access, audit, identity, and system ownership out of this slice

## Self-Review Checklist

- The plan preserves the five approved feature groups: `identity`, `access`, `audit`, `system`, `runtime`.
- The plan does not reintroduce business-domain modules into Phase 1.
- The plan keeps the current repository shape and avoids new top-level apps or crates.
- The plan defines explicit migration mappings from current `auth` and `system/*` ownership into Phase 1 capability groups.
- The plan establishes implementation slices in the approved priority order.
