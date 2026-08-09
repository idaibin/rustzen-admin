# Role Definition Management

## Scope

This vertical slice covers custom-role persistence, the immutable built-in role
boundary, and assignment visibility before deletion. Product behavior is
authoritative in the
[Feature Spec](../../docs/product/features/role-definition-management/spec.md);
the accepted page behavior is authoritative in the
[UI spec](../../docs/ui/features/role-definition-management.md).

## Fixed basis and owners

| Field | Value |
| --- | --- |
| Fixed basis | Pre-change source basis: `feat/platform-spec-workflow` @ `5c0479bb4f586434440eca050a66526b2021d68c` |
| Product owner | Daibin |
| Backend owner | Admin backend owner |
| Frontend owner | Daibin / owner of `apps/web/src/routes/system/role.tsx` |
| Data owner | Admin SQLite migration and role repository |
| Test owner | Admin backend owner |

The fixed basis above is pre-change source context. The implementation and test
evidence below describes the derived worktree package and does not attribute
uncommitted files to that `HEAD`.

## Requirements

| ID | Requirement | Owner | Acceptance | Evidence |
| --- | --- | --- | --- | --- |
| REQ-RDM-001 | An authorized operator can create a custom role with an assignable permission. | Admin backend owner | HTTP create succeeds and the role is persisted in SQLite. | `Automated` by the focused integration test. |
| REQ-RDM-002 | A role assigned to a user cannot be deleted. | Admin backend owner | HTTP delete returns the business refusal; the role and assignment remain. | `Automated` by the focused integration test. |
| REQ-RDM-003 | Built-in role immutability and owner-only permission rules remain backend-owned. | Admin backend owner | Direct mutation requests are rejected by the existing service path. | `Source-resolved`; broader HTTP coverage is `Not verified`. |
| REQ-RDM-004 | Permission cache refresh remains coupled to successful role mutation. | Admin backend owner | Existing service calls refresh after create/update/delete success. | `Source-resolved`; live cache behavior beyond the test is `Not verified`. |
| REQ-RDM-005 | The existing role list exposes assignment visibility for safe deletion affordances. | Admin backend/frontend owners | A real list response includes `assignedUserCount` and `deletable`; custom roles with positive counts are disabled in the UI while zero-count roles keep confirmation, and the delete transaction remains authoritative. | `Automated` by the Axum/SQLite list seam, frontend derivation test, and regenerated code-first contract artifacts. |

## Interface definition

REQ-RDM-005 extends the existing code-first operation; it does not add a route,
request body, permission, or response envelope.

| Contract item | Definition |
| --- | --- |
| Operation | `GET /api/system/roles` (`listRoles`) |
| Authorization | Existing `system:role:list` requirement |
| Existing envelope | Existing paginated Admin response remains unchanged |
| `assignedUserCount` | Required integer, zero or greater; current `user_roles` count for the role |
| `deletable` | Required boolean; true only for a non-built-in role with zero current assignments |
| Consumer rule | Display the count; disable the delete affordance when false |
| Enforcement rule | `DELETE /api/system/roles/{id}` rechecks role identity and assignments transactionally and remains authoritative during races |

The Rust route/DTO remains the authoring authority. `openapi/admin-contract.json`
and the generated TypeScript client are derived artifacts checked in the same
change.

## Gaps

| Owner | Related gap | Gap | Status | Acceptance needed |
| --- | --- | --- | --- | --- |
| Daibin / `apps/web` owner | G-RDM-001 | Browser error injection, empty state, four consecutive 503 responses, retry field preservation, and screenshot evidence are not exercised by the current local run. | `Not verified` | Complete the remaining linked UI matrix on the same artifact basis. |
| Daibin / final closer | G-PLAT-002 | Build evidence becomes stale after any later source, contract, dependency, or toolchain change. The current derived package passed the full gate, but every later development run must refresh it. | `Gap` | Before declaring later work complete, record its basis, rerun the named contract/build/workspace commands, and inspect the generated OpenAPI/client plus current build artifacts. |

The cross-cutting deployment, gateway, SSO, production traffic, and rollback
gap is defined as G-PLAT-001 in [governance.md](../governance.md).

## Evidence and changed files

- `Source-resolved`: role route, handler, service, repository, migration, and
  product/UI specifications linked above were checked against the source basis
  and the listed derived package edits.
- `Automated`: one Axum request path with an in-memory SQLite database covers
  create, assignment, refused delete, and post-failure persistence; the dual
  connection SQLite test covers stale-count rejection and deleted-role
  assignment refusal.
- Changed implementation files:
  `apps/admin/src/features/system/role/{repo,types}.rs`,
  `apps/admin/src/features/system/role/service.rs`,
  `apps/admin/src/features/system/user/repo.rs`, and the existing
  `apps/admin/src/infra/app.rs` test seam.
- Changed contract/consumer files: `openapi/admin-contract.json`,
  `openapi/baselines/contract-admin-native-all-refact-modules-mvp.json`,
  `apps/web/src/api/generated/admin-contract.ts`,
  `apps/web/src/api/system/role/types.d.ts`,
  `apps/web/src/routes/system/role.tsx`, and the route-local deletion-state
  helper/test.
- Changed documentation files: the six files under `platform-spec/` plus the
  minimum repository entry links described in the root docs.
- No new endpoint, route directory, token list, YAML, or Schema Registry is
  added; the existing role-list DTO and generated contract are extended in
  place.

## Tests

Focused command:

```text
cargo test -p rustzen-admin role_management_rejects_deletion_of_assigned_custom_role
cargo test -p rustzen-admin soft_delete_rechecks_assignments_after_stale_count
cargo test -p rustzen-admin role_list_exposes_assignment_count_and_deletable_state
bun test src/routes/system/-role-delete-state.test.ts
```

The test is an integration seam inside the existing Admin app test module; it
uses the real route registration and SQLite migration, not a fake success.

## Runtime acceptance

A prior local browser observation of the current embedded Web build is retained
as historical context only. No fixed artifact and browser script are recorded
for this package, so it is `Not verified` and cannot support a runtime
acceptance claim.

A deployed gateway, SSO session, production traffic result, screenshot, injected
failure state, or rollback result remains `Not verified`. Re-run the browser
check whenever the embedded artifact, role route, or runtime basis changes.
The recurring build-freshness reminder is tracked as G-PLAT-002 in
[governance.md](../governance.md) and [verification.md](../verification.md).
