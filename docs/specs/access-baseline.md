# Access Baseline Spec

> Status: proposed Phase 1 child spec for the `access` capability group

## Goal

Define the smallest reusable `access` baseline for `rustzen-admin` so menus, roles, permissions, and administrative user access management stop living under the catch-all `system` ownership model.

This spec narrows Phase 1 `access` to authorization and access-facing administration.

## Ownership Contract

`access` owns:

- roles
- menus
- permission codes
- role-menu assignment
- user-role assignment
- visible-menu resolution for the current user
- backend permission enforcement contracts
- administrative user access management

`access` does not own:

- login-state ownership
- self-service profile updates
- self-service password change
- dictionary data
- system configuration
- file upload and file metadata
- data-scope or policy-expression engines

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

Current boundary issues:

- `zen-server/src/features/system/mod.rs` still groups `menu`, `role`, `log`, `dict`, and `user` under one technical bucket instead of capability ownership.
- `zen-server/src/features/system/user/` mixes access-facing administration with generic user lifecycle handling.
- the current route and module names still hide `access` under `system`.

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

Current boundary issues:

- `zen-web/src/routes/system/role.tsx` and `zen-web/src/routes/system/menu.tsx` are clearly `access` pages but still live under the `system` route group.
- `zen-web/src/routes/system/user.tsx` contains role assignment, status changes, password reset, and delete actions, so it is an admin access-management surface rather than an identity page.
- `zen-web/src/api/system/index.ts` still exposes access ownership through the `systemAPI` barrel.

## Phase 1 Access Baseline

The Phase 1 `access` slice should establish one explicit capability owner across backend and frontend:

- backend owner: `zen-server/src/features/access/`
- frontend API owner: `zen-web/src/api/access/`
- frontend route owner: `zen-web/src/routes/access/`

The baseline must deliver:

- role management
- menu management
- role-menu assignment
- user-role assignment
- visible-menu resolution for the current user
- backend permission checks used by protected routes and actions

The baseline may continue to consume shared user records and current-user identity state, but that does not change `access` ownership.

## Backend Boundary Rules

- keep roles, menus, and permission-facing assignment paths inside `access`
- keep permission cache management and visible-menu resolution as `access` dependencies
- keep administrative user-role and status management in the `access` slice when they shape what a user can do
- keep self-service account updates out of `access`
- keep logs and dictionaries out of `access`

## Administrative User CRUD Ownership

`system/user` must be split by ownership intent instead of by record type.

Administrative user actions belong to `access` when they determine or constrain what a user can access.

That means the future `access` slice owns:

- create admin-managed user accounts
- assign and replace user roles
- change user status
- perform administrator-triggered password reset
- delete admin-managed user accounts
- provide admin user option lists used by access-facing forms

The future `access` slice does not own:

- current-user session data
- self-profile edits
- self-password change
- self-avatar update

The key rule is simple:

- user record ownership does not automatically mean `identity`
- self-service actions belong to `identity`
- administrator-managed access actions belong to `access`

## Frontend Boundary Rules

- move role and menu management into a dedicated `access` route group
- treat the admin user-management page as part of `access`, not `identity`
- keep self-profile pages out of `access`
- replace the `systemAPI` ownership signal with an explicit `access` API namespace during the slice

## Explicit Non-Goals

This spec does not introduce:

- policy builders
- condition-based permission rules
- data-scope permissions
- tenant-aware authorization
- workflow approvals
- business-domain ownership models

## Backend Implementation Checklist

- create `zen-server/src/features/access/` as the Phase 1 owner of menu, role, and access-facing user-management capabilities
- move role and menu routes out of the `system` capability name
- define the access-owned part of `system/user` around user-role assignment, status changes, and admin-side password reset
- keep current permission cache and route-permission behavior intact while moving capability ownership
- leave self-service account updates outside the slice

## Concrete Backend Split Checklist For `system/user`

- move the current `list_users` admin endpoint into the future `access` ownership boundary
- move `create_user` into `access` because account creation currently includes initial role assignment
- move `update_user` into `access` because it replaces assigned roles
- move `update_user_status` into `access`
- move `update_user_password` into `access` as an administrator-triggered reset path
- move `delete_user` into `access`
- move `get_user_options` and `get_user_status_options` into `access` while they are consumed by admin management flows
- keep permission codes explicit during the move instead of hiding them behind a new compatibility layer
- do not move self-profile or self-password paths into this slice if they are introduced later under `identity`

## Frontend Implementation Checklist

- create `zen-web/src/api/access/` as the dedicated client surface for access ownership
- create `zen-web/src/routes/access/` for role, menu, and admin user-management pages
- move role and menu pages out of the `system` route group
- keep administrative user-management actions with `access`
- keep profile and self-password flows out of the access route group

## Concrete Frontend Regrouping Checklist

- move `zen-web/src/routes/system/role.tsx` into the future `access` route group
- move `zen-web/src/routes/system/menu.tsx` into the future `access` route group
- move `zen-web/src/routes/system/user.tsx` into the future `access` route group
- move `zen-web/src/api/system/role/` into the future `access` API namespace
- move `zen-web/src/api/system/menu/` into the future `access` API namespace
- move `zen-web/src/api/system/user/` into the future `access` API namespace
- keep `systemAPI.user` out of any self-service profile flow
- preserve existing permission-code checks on role, menu, and user-management actions during regrouping

## Exit Condition

The `access` baseline is complete when `rustzen-admin` has a dedicated access-control surface across backend and frontend without hiding authorization ownership under the generic `system` label.
