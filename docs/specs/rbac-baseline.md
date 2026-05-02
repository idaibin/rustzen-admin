# RBAC Baseline Spec

> Status: proposed Phase 1 child spec for the `rbac` capability group

## Goal

Define the smallest reusable RBAC baseline for `rustzen-admin` using the role, menu, permission-code, and admin user-management functions already present in the project.

This spec keeps Phase 1 focused on starter-level role-based access control, not a general IAM or policy platform.

## Ownership Contract

`rbac` owns:

- roles
- menus
- permission codes
- role-menu assignment
- user-role assignment
- visible-menu resolution for the current user
- backend permission enforcement contracts
- administrative user management needed for assigning and constraining access

`rbac` does not own:

- login-state ownership
- current-account profile updates
- current-account password change
- dictionary data
- system configuration
- file upload and file metadata
- policy-expression engines
- tenant-aware authorization

Administrative user management belongs here only where it changes what a user can access.

## Current Baseline Snapshot

### Backend

Current starting points:

- `zen-server/src/features/system/menu/`
- `zen-server/src/features/system/role/`
- access-facing parts of `zen-server/src/features/system/user/`
- `zen-server/src/infra/permission.rs`

Current baseline already present:

- role CRUD
- menu CRUD
- role-menu assignment during role create and update
- permission cache refresh and permission lookups
- admin-side user-role assignment through `system/user`

Current placement:

- `system/role`, `system/menu`, and access-facing parts of `system/user` are the current RBAC implementation.
- Keep this placement for now. Do not rename directories or routes just to match the capability label.

### Frontend

Current starting points:

- `zen-web/src/routes/system/role.tsx`
- `zen-web/src/routes/system/menu.tsx`
- `zen-web/src/routes/system/user.tsx`
- `zen-web/src/api/system/role/`
- `zen-web/src/api/system/menu/`
- `zen-web/src/api/system/user/`

Current baseline already present:

- role management page
- menu management page
- admin user-management page with role assignment
- role and menu option fetching used by admin forms

Current placement:

- `system/role`, `system/menu`, and `system/user` pages are the current RBAC UI.
- `systemAPI.role`, `systemAPI.menu`, and `systemAPI.user` are the current RBAC client API surface.
- Keep this placement for now. Do not rename routes or API namespaces just to match the capability label.

## Phase 1 RBAC Baseline

The Phase 1 `rbac` baseline is an ownership label for existing functionality:

- backend implementation: `zen-server/src/features/system/role/`, `zen-server/src/features/system/menu/`, and access-facing parts of `zen-server/src/features/system/user/`
- frontend implementation: `zen-web/src/routes/system/role.tsx`, `zen-web/src/routes/system/menu.tsx`, `zen-web/src/routes/system/user.tsx`
- frontend API implementation: `zen-web/src/api/system/role/`, `zen-web/src/api/system/menu/`, `zen-web/src/api/system/user/`

The baseline must deliver:

- role management
- menu management
- role-menu assignment
- user-role assignment
- visible-menu resolution for the current user
- backend permission checks used by protected routes and actions

The baseline may continue to consume shared user records and current-user auth state, but that does not change RBAC ownership.

## Boundary Rules

- keep roles, menus, and permission-facing assignment paths inside `rbac`
- keep permission cache management and visible-menu resolution as RBAC dependencies
- keep administrative user-role and status management in the RBAC slice when they shape what a user can do
- keep current-account updates out of `rbac`
- keep logs and dictionaries out of `rbac`

## Administrative User CRUD Ownership

`system/user` must be split by ownership intent instead of by record type.

Administrative user actions belong to `rbac` when they determine or constrain what a user can access.

That means the current RBAC baseline covers:

- create admin-managed user accounts
- assign and replace user roles
- change user status
- perform administrator-triggered password reset
- delete admin-managed user accounts
- provide admin user option lists used by RBAC forms

The current RBAC baseline does not own:

- current-user session data
- self-profile edits
- self-password change
- self-avatar update

The key rule is simple:

- self-service actions belong to `account`
- administrator-managed access actions belong to `rbac`

## Explicit Non-Goals

This spec does not introduce:

- policy builders
- condition-based permission rules
- data-scope permissions
- tenant-aware authorization
- workflow approvals
- business-domain ownership models

## Implementation Rule

- Do not create `zen-server/src/features/rbac/`, `zen-web/src/api/rbac/`, or `zen-web/src/routes/rbac/` during Phase 1 unless a real implementation need appears.
- Treat the current `system/*` paths as existing RBAC carriers.
- Only change code when behavior is missing or incorrect.

## Exit Condition

The `rbac` baseline is complete when the existing role, menu, permission-code, and admin user-management behavior is documented clearly and remains free of IAM-platform scope.
