# Auth And Account Baseline Spec

> Status: current Phase 1 child spec for the `auth` and `account` baseline

## Goal

Keep `rustzen-admin` positioned as a high-quality Rust admin starter, not a general IAM platform.

This baseline covers only the account features an admin starter needs first:

- `auth`: login, logout, and current-session bootstrap
- `account`: current administrator profile, avatar, and password changes

## Ownership Contract

`auth` owns:

- username/password login
- token creation
- logout cache cleanup
- current-user session data
- permission-bearing session bootstrap

`account` owns:

- current-user avatar update
- current-user profile update
- current-user password change

Neither `auth` nor `account` owns:

- role management
- menu management
- permission assignment
- administrative user CRUD
- user status changes
- administrator-triggered password reset
- OAuth2, OIDC, SAML, SSO, MFA, SCIM, tenant, or organization models

Those larger IAM concerns are outside the current product scope.

## Current Baseline Snapshot

### Backend

Current owners:

- `zen-server/src/features/auth/`
- `zen-server/src/features/account/`

Current baseline:

- `/api/auth/login`
- `/api/auth/logout`
- `/api/auth/me`
- `/api/account/avatar`
- `/api/account/profile`
- `/api/account/password`

`auth` may return permission-bearing session data because the current frontend needs menu and permission bootstrap after login. This is a starter-level contract, not an IAM platform boundary.

### Frontend

Current owners:

- `zen-web/src/api/auth/`
- `zen-web/src/api/account/`
- `zen-web/src/routes/login.tsx`
- `zen-web/src/routes/profile.tsx`
- `zen-web/src/store/useAuthStore.ts`

Current baseline:

- login page
- logout action
- current-user fetch
- profile edit surface
- password change surface
- avatar upload action

## Boundary Rules

- Keep login and session bootstrap in `auth`.
- Keep current-user profile, avatar, and password changes in `account`.
- Keep role, menu, permission, and admin user management for the later `rbac` slice.
- Keep login-event recording as an `audit` integration point.
- Do not introduce IAM-platform protocol surfaces in Phase 1.

## Explicit Non-Goals

This spec does not introduce:

- MFA
- password recovery
- email or phone verification
- third-party login
- OAuth2 or OIDC provider behavior
- SAML or enterprise SSO
- tenant, organization, workspace, or federation models

## Exit Condition

The baseline is complete when `rustzen-admin` has a clear starter-level login and current-account surface without using IAM as a product-capability group.
