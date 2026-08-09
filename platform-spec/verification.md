# Verification

## Basis and package

The fixed basis is the pre-change source snapshot
`feat/platform-spec-workflow` @ `5c0479bb4f586434440eca050a66526b2021d68c`.
The checks below, when marked `Automated`, ran against the derived worktree
package layered on that snapshot. They do not claim that uncommitted package
files are present in the recorded `HEAD`.

## Commands

| Command | Purpose | Status |
| --- | --- | --- |
| `just verify-platform-spec` | Fixed files, headings, IDs, exact-case links, owner placeholders, and state words. | `Automated` on the derived worktree package. |
| `cargo test -p rustzen-admin role_management_rejects_deletion_of_assigned_custom_role` | Real Admin Axum/SQLite role refusal seam. | `Automated` on the derived worktree package. |
| `cargo test -p rustzen-admin soft_delete_rechecks_assignments_after_stale_count` | Dual-connection SQLite stale-count and deleted-role assignment conditions. | `Automated` on the derived worktree package. |
| `cargo test -p rustzen-admin role_list_exposes_assignment_count_and_deletable_state` | Real Admin Axum/SQLite role-list projection for assignment count and deletion affordance. | `Automated` on the derived worktree package. |
| `bun test src/routes/system/-role-delete-state.test.ts` | Frontend zero/positive assignment-count deletion-state derivation. | `Automated` on the derived worktree package. |
| `npx -p @google/design.md@0.3.0 designmd lint --format json DESIGN.md` | Shared visual authority lint. | `Automated`; zero findings. |
| `just contract-generate`, `just contract-verify`, and `just contract-compat` | Code-first OpenAPI, generated TypeScript client, and updated response contract compatibility. | `Artifact-resolved`/`Automated` after the current run. |
| `cd apps/web && bun run build` | Frontend production artifact for the changed role page. | `Artifact-resolved`; build passed with the repository's existing chunk-size warning. |
| `CARGO_TARGET_DIR="$(pwd)/target/platform-spec" just contract-verify` and `just contract-compat` | Existing Admin route/client generation, tests, and baseline compatibility. | `Automated`; passed for the current derived worktree package. |
| `CARGO_TARGET_DIR="$(pwd)/target/platform-spec" just check` | Repository format, frontend, platform-spec, workspace check, clippy, and tests. | `Automated`; the full repository gate passed for the current derived worktree package. |
| Isolated `rz-admin serve` + owner login + headless Chrome CDP | Historical local role list/create observation. | `Not verified`; no fixed artifact and browser script are recorded for this package. |

## Static acceptance

The no-dependency script checks the six allowlisted Markdown files, required
headings, unique `REQ-###`/`GAP-###` IDs, local relative links, owner
placeholders, and the exact status words
`Declared`, `Source-resolved`, `Artifact-resolved`, `Automated`,
`Runtime-resolved`, `Gap`, and `Not verified`.
It intentionally does not parse or duplicate API schemas, DTOs, route catalogs,
tokens, or runtime state.

## Requirement exit matrix

| Layer | Requirement | Source | Automated | Artifact | Runtime | Related gap | Current conclusion |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Platform | REQ-PLAT-001–REQ-PLAT-002 | `Source-resolved` | `Automated` | not applicable: Markdown-only baseline | `Not verified` | G-PLAT-001 | `Automated` |
| Role refusal seam | REQ-RDM-001–REQ-RDM-002 | `Source-resolved` | `Automated` | `Not verified` | `Not verified` | G-RDM-001, G-PLAT-001 | `Automated` for the HTTP/SQLite seam; browser acceptance remains `Not verified` |
| Role rules and list projection | REQ-RDM-003–REQ-RDM-005 | `Source-resolved` | `Automated` for focused seams and full repository gate | `Artifact-resolved` for generated contract/client and frontend build | `Not verified` | G-RDM-001, G-PLAT-001, G-PLAT-002 | API/client/build evidence is current for this derived package; browser acceptance remains `Not verified`, and later changes reopen G-PLAT-002 |

No row is release-ready until every required artifact/runtime column has
evidence and its related gaps are closed or explicitly waived by their owner.

## Runtime acceptance

The selected HTTP/SQLite integration seam is the only runtime-adjacent check in
this bounded package. It must prove the refusal response and unchanged database
state after deleting an assigned custom role. It does not prove a deployed
runtime; that boundary remains `Not verified`.

## Evidence log

| Evidence | Owner | State | Notes |
| --- | --- | --- | --- |
| Fixed branch/SHA and source inspection | Platform spec owner | `Source-resolved` | Basis is recorded in [PROJECT.md](./PROJECT.md). |
| Markdown validator result | Platform spec owner | `Automated` | `just verify-platform-spec` returned PASS. |
| Role HTTP/SQLite seam | Admin backend owner | `Automated` | Derived worktree package returned passing HTTP refusal and dual-connection SQLite tests. |
| Role list assignment projection | Admin backend/frontend owners | `Automated` | The real list seam returned zero/positive counts and matching `deletable` values; the frontend derivation test covered both states. |
| Code-first contract artifacts | Admin backend/frontend owners | `Artifact-resolved` | `just contract-generate`, `just contract-verify`, and `just contract-compat` are recorded for the updated OpenAPI baseline and generated client. |
| Shared DESIGN lint | Design-system owner | `Automated` | Official `@google/design.md@0.3.0` lint returned zero findings. |
| Admin route/client contract | Admin backend/frontend owners | `Automated` | Contract generation/normalization and checks passed; benchmark remains ignored by its repository contract. |
| Repository baseline check | Repository owners | `Automated` | Full default gate passed for the current derived worktree package. G-PLAT-002 reopens only after a later source, contract, dependency, or toolchain change. |
| Isolated role browser run | Frontend/runtime owners | `Not verified` | A prior local observation is historical context only; no fixed artifact and browser script are recorded here. |
| Deployed gateway/SSO, injected browser failures, screenshots, production and rollback evidence | Runtime/release owners | `Not verified` | Not exercised by the isolated local run. |

## Not verified

No static command in this directory upgrades a `Declared`, `Source-resolved`,
or `Not verified` entry into runtime truth. Record a named command and artifact when
those environments are actually exercised.

## Follow-up build reminder

G-PLAT-002 is intentionally retained as a recurring development reminder. The
current evidence belongs only to the fixed pre-change branch
`feat/platform-spec-workflow` @ `5c0479bb4f586434440eca050a66526b2021d68c`
plus this derived worktree. Any later source, contract, dependency, or toolchain
change invalidates that evidence. Before declaring later work complete, record
the new basis, rerun the contract, frontend build, and Rust workspace commands,
and inspect the generated OpenAPI/client plus current `apps/web/dist` and
`target/` artifacts.
