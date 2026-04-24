# Identity Baseline Spec

> Status: proposed Phase 1 child spec for the `identity` capability group

## Goal

Define the smallest reusable `identity` baseline for `rustzen-admin` so the repository has a clear self-service account boundary before broader capability refactors begin.

This spec narrows Phase 1 `identity` to current-user session and self-service account state.

## Ownership Contract

`identity` owns:

- login
- logout
- current-user session data
- self profile data
- self password change
- self avatar update

`identity` does not own:

- role management
- menu management
- permission assignment
- administrative user CRUD
- user status changes
- administrator-triggered password reset

Those administrative capabilities remain outside the `identity` baseline and should not be pulled into this slice just because they operate on user records.

## Current Baseline Snapshot

### Backend

Current starting points:

- `zen-server/src/features/auth/mod.rs`
- `zen-server/src/features/auth/handler.rs`
- `zen-server/src/features/auth/service.rs`
- `zen-server/src/features/auth/repo.rs`

Current baseline already present:

- login endpoint
- logout endpoint
- current-user query
- avatar upload/update

Current baseline still missing:

- self-profile update beyond avatar
- self-password change

Current boundary issues:

- `AuthService::get_login_info()` returns permission-bearing session data, so the current `auth` area still exposes `access` output.
- login flow writes through `features::system::log::LogService`, which is an `audit` dependency crossing the current capability boundary.

### Frontend

Current starting points:

- `zen-web/src/routes/login.tsx`
- `zen-web/src/api/auth/api.ts`
- `zen-web/src/store/useAuthStore.ts`

Current baseline already present:

- login page
- login API
- logout API
- current-user fetch

Current baseline still missing:

- dedicated profile route group
- dedicated identity API namespace
- self-profile edit surface
- self-password change surface

Current boundary issues:

- `zen-web/src/routes/system/user.tsx` is an administrative user-management page and must not become the Phase 1 profile page.
- `zen-web/src/api/system/user/api.ts` is admin-facing and must not become the Phase 1 identity API surface.

## Phase 1 Identity Baseline

The Phase 1 `identity` slice should establish one explicit capability owner across backend and frontend:

- backend owner: `zen-server/src/features/identity/`
- frontend API owner: `zen-web/src/api/identity/`
- frontend route owner: `zen-web/src/routes/profile/`

The baseline must deliver:

- login
- logout
- current-user query
- self-profile read/update
- self-password change
- self avatar update

The baseline may continue to consume `access` output for the current-user session payload, but that does not change `identity` ownership.

## Backend Boundary Rules

- keep authentication and current-user session handling inside `identity`
- keep self-service account changes inside `identity`
- keep login logging as an `audit` dependency
- keep permission caching and permission resolution as `access` dependencies
- keep administrator-driven user lifecycle actions out of `identity`

## Frontend Boundary Rules

- keep login flow and current-user bootstrap inside `identity`
- add a dedicated profile route instead of reusing admin user-management pages
- keep role assignment, status changes, and admin password reset out of the profile surface
- keep `system/user` as an admin page until a dedicated `access` regrouping slice moves it

## Explicit Non-Goals

This spec does not introduce:

- MFA
- email or phone verification
- password recovery flow
- third-party login
- tenant-aware identity variants
- policy or role editing

## Initial Implementation Checklist

1. define the backend `identity` module boundary from the current `auth` feature
2. preserve existing session behavior while moving ownership naming from `auth` to `identity`
3. add missing self-profile update capability
4. add missing self-password change capability
5. create frontend `identity` API and profile route entrypoints
6. keep admin user-management actions out of the slice

## Backend Implementation Checklist

- create `zen-server/src/features/identity/` as the Phase 1 owner of the current `auth` baseline
- move login, logout, current-user, and avatar handling out of the `auth` capability name
- keep the current session payload intact while treating permissions and visible menus as `access` dependencies
- add a self-profile update path for profile fields beyond avatar
- add a self-password change path for the current user
- keep login-event recording as an `audit` integration point
- leave `zen-server/src/features/system/user/` in place for admin-side user CRUD, password reset, status changes, and role-linked updates

## Frontend Implementation Checklist

- create `zen-web/src/api/identity/` as the dedicated client surface for the identity slice
- move login, logout, and current-user fetch into the new identity API namespace
- create `zen-web/src/routes/profile/` for self-service account pages
- add a self-profile edit surface
- add a self-password change surface
- keep `zen-web/src/routes/system/user.tsx` as an admin page during the identity slice
- keep `zen-web/src/api/system/user/api.ts` out of the profile flow

## Exit Condition

The `identity` baseline is complete when `rustzen-admin` has a dedicated self-service identity surface across backend and frontend without mixing in administrative user management.
