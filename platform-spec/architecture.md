# Platform Architecture Boundary

## Current architecture

The relevant role path is a single Admin vertical slice:

`apps/web/src/routes/system/role.tsx` → `apps/web/src/api/system/role/` →
Admin route registration → role handler → role service → role repository →
Admin SQLite `roles`, `menus`, `role_menus`, and `user_roles` relations.

The route registration remains the HTTP authority. The product Feature Spec and
UI spec describe behavior and states; they do not create a second route or DTO
catalog.

## Contract boundaries

- Capability checks are registered through Admin's existing
  `ContractRouter`/`Require(...)` path and shared `crates/auth/` policy.
- Role validation stays in `apps/admin/src/features/system/role/service.rs`.
- Role persistence stays in `apps/admin/src/features/system/role/repo.rs`.
- The role-list projection derives `assignedUserCount` from `user_roles` in the
  existing repository query and derives `deletable` in the response DTO; the
  delete transaction rechecks that relation and remains final authority.
- Permission-cache refresh remains the existing Admin permission service seam.
- The integration test uses an in-memory SQLite pool and the real Axum route
  stack; it does not introduce a new harness or runtime process.

## Runtime truth boundary

The architecture statement is `Source-resolved` against the recorded
pre-change source basis and the selected tests are `Automated` only when they
are recorded in [verification.md](./verification.md) for the derived worktree
package. This does not establish `Runtime-resolved` behavior for a deployed
service. Live gateway, SSO, browser, deployment, production traffic, and
rollback acceptance remain `Not verified` in this package.

## Requirement ledger

| Owner | Related requirement | Architecture boundary | Changed files | Tests |
| --- | --- | --- | --- | --- |
| Admin backend owner | REQ-RDM-001 | Preserve the existing Admin route/service/repository/SQLite ownership chain and reject role-assignment/soft-delete interleavings. | `apps/admin/src/features/system/role/{repo,service}.rs`, `apps/admin/src/features/system/user/repo.rs`, `apps/admin/src/infra/app.rs` test seam | `cargo test -p rustzen-admin soft_delete_rechecks_assignments_after_stale_count` and `cargo test -p rustzen-admin role_management_rejects_deletion_of_assigned_custom_role` |
| Admin backend/frontend owners | REQ-RDM-005 | Extend the existing role-list operation and client projection with assignment visibility while keeping deletion enforcement in the transaction. | `apps/admin/src/features/system/role/{repo,types}.rs`, `openapi/admin-contract.json`, `apps/web/src/api/{generated/admin-contract.ts,system/role}`, `apps/web/src/routes/system/role*.ts*` | `cargo test -p rustzen-admin role_list_exposes_assignment_count_and_deletable_state`, `bun test src/routes/system/-role-delete-state.test.ts`, and contract generation/compatibility commands |
| Daibin | REQ-PLAT-001 | Keep platform documentation descriptive and source-linked, without copied route/DTO/schema inventories. | `platform-spec/*.md` | `just verify-platform-spec` |
