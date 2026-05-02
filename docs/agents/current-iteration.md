# Current Iteration

> Current repository documentation iteration state

## Focus

Cancel the Phase 1 IAM-style direction and keep `rustzen-admin` positioned as a practical Rust admin starter.

## In Scope

- keep login and current-session bootstrap under `auth`
- keep current administrator profile, avatar, and password changes under `account`
- document `system/role`, `system/menu`, and `system/user` as the current RBAC baseline implementation
- align Phase 1 specs, rollout plan, project map, and current code with the reduced scope

## Out Of Scope

- OAuth2, OIDC, SAML, SSO, MFA, SCIM, tenant, organization, workspace, and federation features
- moving role, menu, or admin user-management code during this cleanup
- deployment changes unrelated to the auth/account naming cleanup

## Execution Mode

- read this file first on each recurring iteration
- read `docs/specs/admin-foundation-phase-1.md` and `docs/plans/admin-foundation-phase-1-rollout.md` before selecting work
- execute one smallest safe task per wake-up
- update this file and the relevant plan/spec before stopping
- do not touch unrelated working-tree changes

## Recently Completed

- [x] replaced the Phase 1 IAM-style product-capability direction with starter-level `auth`, `account`, and `rbac` boundaries
- [x] moved backend login/session code back under `zen-server/src/features/auth/`
- [x] moved backend current-account profile, avatar, and password code under `zen-server/src/features/account/`
- [x] moved frontend login/session requests back under `zen-web/src/api/auth/`
- [x] moved frontend current-account profile and password requests under `zen-web/src/api/account/`
- [x] kept `/profile` as the current-account page without making it an IAM platform surface

## Next Task Queue

- [ ] keep the RBAC baseline documented as existing functionality and avoid path-only migration work

## Source Documents For This Iteration

- `docs/README.md`
- `docs/specs/admin-foundation-phase-1.md`
- `docs/specs/auth-account-baseline.md`
- `docs/specs/rbac-baseline.md`
- `docs/specs/audit-baseline.md`
- `docs/specs/system-baseline.md`
- `docs/specs/runtime-baseline.md`
- `docs/plans/admin-foundation-phase-1-rollout.md`
