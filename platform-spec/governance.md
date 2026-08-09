# Governance

## Authority boundaries

The repository keeps one owner per semantic layer:

| Layer | Canonical authority | Owner | Boundary |
| --- | --- | --- | --- |
| Product behavior | [role-definition-management Feature Spec](../docs/product/features/role-definition-management/spec.md) | Daibin | User behavior, business rules, failure states, and acceptance. |
| UI slice | [role UI spec](../docs/ui/features/role-definition-management.md) | Daibin / `apps/web` owner | Page composition, interaction, accessibility, and visual states. |
| Shared visual semantics | [DESIGN.md](../DESIGN.md) | Daibin | Shared tokens and component meaning only. |
| HTTP implementation | [role routes](../apps/admin/src/features/system/role/mod.rs) and handlers/services | Admin backend owner | Runtime route registration, validation, and response behavior. |
| SQLite schema | [Admin baseline migration](../apps/admin/migrations/sqlite/0001_init.sql) | Admin backend owner | Tables, views, indexes, and persistence shape. |
| Verification command | [justfile](../justfile) and [verify-platform-spec.sh](../scripts/verify-platform-spec.sh) | Daibin | Static structure checks only. |

`platform-spec/` links to these authorities; it does not copy their tokens,
DTOs, route catalogs, or schemas.

Implementation sources are authoritative descriptions of current behavior, not
judges of their own correctness. When implementation and target behavior differ,
the linked Product/UI authority keeps the target and this ledger records a Gap;
the source must not silently redefine the requirement.

## Owners

- Daibin: keeps this ledger, fixed basis, and static validator
  aligned.
- Admin backend owner: owns role service behavior and the HTTP/SQLite test seam.
- Daibin / `apps/web` owner: owns the role page and UI evidence.
- Daibin: owns [DESIGN.md](../DESIGN.md); no change is needed for
  this feature because the accepted role surface reuses existing semantics.
- Final closer: the task owner records the commands and leaves unknown runtime
  evidence as `Not verified`.

## States and evidence

Use the exact scaffold states `Declared`, `Source-resolved`,
`Artifact-resolved`, `Automated`, `Runtime-resolved`, `Gap`, and `Not
verified`. A source inspection is not a runtime test; a static validator is not
an HTTP, browser, deployment, or rollback proof.

Evidence levels are recorded at the smallest useful boundary:

- `Source-resolved`: a current source, migration, or spec statement was checked
  against the recorded basis.
- `Automated`: a named focused test, validator, or lint command completed against
  the derived worktree package.
- `Runtime-resolved`: an actual authorized service, browser, or deployment
  interaction was exercised on a named artifact and environment; otherwise use
  `Not verified`.

## Change discipline

1. Reconfirm the branch and fixed basis before editing.
2. Change the canonical source owner first; link to it from this baseline.
3. Keep one vertical slice bounded by a requirement, owner, changed-file list,
   test, and runtime acceptance statement.
4. Do not add YAML, JSON, Schema Registry, compatibility wrappers, or copied
   route/DTO/token inventories for machine convenience.
5. Preserve unrelated worktree changes and do not use this baseline to claim
   deployment, production traffic, SSO, browser, or rollback evidence.
6. Review every new entry for authority duplication. Platform-spec may contain
   IDs, acceptance criteria, trace links, evidence references, and Gaps; it must
   not copy API fields, complete permission/route inventories, UI token values,
   implementation inventories, or architecture diagrams from their owners.

## Gap ledger

| ID | Gap | Owner | Status | Closure evidence |
| --- | --- | --- | --- | --- |
| G-PLAT-001 | Deployment, gateway, SSO, browser, production traffic, and rollback evidence is outside this static task. | Daibin / runtime-release owner | `Not verified` | A separate named runtime run is required. |
| G-RDM-001 | The role UI acceptance matrix is not replaced by this backend seam. | Daibin / `apps/web` owner | `Not verified` | Use the linked UI spec and browser evidence. |
| G-PLAT-002 | Build and artifact evidence must be refreshed after every later source, contract, dependency, or toolchain change. The current derived package passed the full gate; this recurring reminder reopens when its basis changes. | Daibin / final closer | `Gap` | Record the new basis, run `just contract-generate`, `just contract-verify`, `just contract-compat`, `cd apps/web && bun run build`, and the relevant Rust workspace checks; inspect `openapi/admin-contract.json`, the baseline/client artifacts, `apps/web/dist`, and `target/` before declaring later work complete. |
