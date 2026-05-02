# Admin Foundation Phase 1 Rollout Implementation Plan

**Goal:** Keep `rustzen-admin` as a practical Rust admin starter by delivering the minimum reusable admin foundation: `auth`, `account`, `rbac`, `audit`, `system`, and `runtime`.

**Architecture:** Keep the existing monorepo shape. Do not introduce a general IAM product-capability group. Do not rename paths just to match capability labels.

**Tech Stack:** Markdown, Rust backend features, React frontend routes and API modules, `AGENTS.md`, repository guide docs, `rg`, `git diff`

---

## Current-State Mapping

Current backend feature ownership:

- `auth/`
- `account/`
- `dashboard/`
- `system/`

Current frontend route ownership:

- `login`
- `profile`
- `index`
- `system/dict`
- `system/log`
- `system/menu`
- `system/role`
- `system/user`

Current frontend API ownership:

- `auth/`
- `account/`
- `dashboard/`
- `system/`

Phase 1 target ownership:

- `auth`
- `account`
- `rbac`
- `audit`
- `system`
- `runtime`

## Product Boundary Decision

Phase 1 should not build an IAM platform.

The accepted starter-level boundary is:

- `auth`: login, logout, current-user session bootstrap
- `account`: current administrator profile, avatar, and password changes
- `rbac`: the existing roles, menus, permission codes, and admin user-management needed for access control
- `audit`: login and operation logs
- `system`: dictionaries and future system configuration
- `runtime`: file/resource path handling

Explicitly out of scope:

- OAuth2, OIDC, SAML, SSO, MFA, SCIM
- tenant, organization, workspace, federation
- policy engines, data-scope builders, enterprise directory sync

## Current Migration Findings

### Backend Findings

- `zen-server/src/features/auth/` owns login, logout, token creation, permission-bearing session bootstrap, and current-user lookup.
- `zen-server/src/features/account/` owns current-user avatar, profile update, and password change.
- login flow records `AUTH_LOGIN` through `features::system::log::LogService`, so login flow currently depends on the future `audit` owner.
- `zen-server/src/features/system/menu/`, `zen-server/src/features/system/role/`, and access-facing parts of `zen-server/src/features/system/user/` are the current RBAC implementation.
- `zen-server/src/features/system/user/` owns admin-side user CRUD, role assignment, status changes, and password reset. These are RBAC/admin capabilities, not current-account self-service capabilities.

### Frontend Findings

- `zen-web/src/api/auth/api.ts` covers login, logout, and current-user fetch.
- `zen-web/src/api/account/api.ts` covers current-account profile update and password change.
- `zen-web/src/routes/login.tsx` is the login entrypoint.
- `zen-web/src/routes/profile.tsx` is the current-account self-service surface.
- `zen-web/src/routes/system/user.tsx` is an administrative user-management page with role assignment, password reset, status changes, and delete actions.
- `zen-web/src/routes/system/role.tsx` and `zen-web/src/routes/system/menu.tsx` are RBAC pages.
- The current `system/*` routes and APIs are acceptable RBAC carriers for Phase 1.

## Status Snapshot

- done: complete the initial documentation governance loop
- done: define the Phase 1 starter groups
- done: write the auth/account baseline spec
- done: write the RBAC baseline spec
- done: write the audit, system, and runtime baseline specs
- done: implement the first auth/account code slice
- next: keep RBAC documented as existing functionality and avoid path-only migration work

## RBAC Baseline Checklist

- keep `system/role` as the current role-management carrier
- keep `system/menu` as the current menu and permission-code carrier
- keep `system/user` as the current admin user-management and user-role assignment carrier
- keep `zen-core::permission`, backend `Require(...)`, and frontend permission-code checks as the permission enforcement path
- keep current-account profile, avatar, and password paths in `account`
- do not create `rbac` directories or routes unless behavior work requires it

## Slice Order

### Slice 1: Auth And Account Baseline

- keep login, logout, and current-user bootstrap in `auth`
- keep profile, avatar, and password self-service in `account`
- treat login logging as an `audit` dependency
- do not introduce IAM-platform naming or protocol surfaces

### Slice 2: RBAC Baseline

- document the existing role, menu, permission-code, and admin user-management behavior as the RBAC baseline
- preserve current `system/*` routes and API modules
- only change RBAC code for missing or incorrect behavior

### Slice 3: Audit Baseline

- document `system/log` as the current audit carrier
- keep log writing as the integration point used by auth/account/RBAC actions
- only change audit code for missing or incorrect behavior

### Slice 4: System Baseline

- keep dictionary management and future system configuration under `system`
- keep RBAC, audit, account, and runtime ownership out of this slice

### Slice 5: Runtime Baseline

- document `common/files.rs` and runtime static serving as the current runtime file-resource carrier
- keep account avatar upload as a consumer of runtime file handling
- only change runtime code for missing or incorrect behavior

## Next Task

Keep Slice 2 as a documentation and validation step unless a concrete RBAC behavior gap is found.
