# Current Iteration

> Current repository documentation iteration state

## Focus

Roll out and align the documentation governance structure and establish the first reusable product-capability phase for `rustzen-admin`.

## In Scope

- tighten root `AGENTS.md`
- tighten `zen-server/AGENTS.md`
- tighten `zen-web/AGENTS.md`
- expose `docs/goals/`, `docs/plans/`, `docs/specs/`, and `docs/agents/`
- seed the first goal and agent documents
- add a stable documentation index
- align repository guide docs with the new document layers
- define Phase 1 capability groups: `identity`, `access`, `audit`, `system`, and `runtime`
- turn the Phase 1 spec into a staged rollout plan

## Out Of Scope

- backend feature refactors
- frontend feature refactors
- deployment changes unrelated to documentation governance

## Exit Conditions

- the new docs areas exist
- entry documents point to them
- `AGENTS.md` responsibilities are clear and non-overlapping
- documentation entrypoints describe both code and docs placement clearly
- the first product-capability phase has a written spec and rollout plan

## Execution Mode

- read this file first on each recurring iteration
- read the linked Phase 1 spec and rollout plan before selecting work
- execute one smallest safe task per wake-up
- update both this file and the relevant `docs/plans/` or `docs/specs/` document before stopping
- do not touch unrelated non-doc changes in the working tree

## Recently Completed

- [x] reviewed the current backend `auth` and `system/*` implementation against the Phase 1 capability map
- [x] reviewed the current frontend `auth` and `system/*` APIs and routes against the Phase 1 capability map
- [x] updated the Phase 1 rollout plan with migration findings from the current codebase
- [x] wrote a dedicated `identity` baseline spec under `docs/specs/`
- [x] wrote a dedicated `access` baseline spec under `docs/specs/`
- [x] added the new child spec to the repository documentation indexes
- [x] derived the backend identity implementation checklist from the current `auth` module
- [x] derived the frontend identity implementation checklist from the current login flow and missing profile surface
- [x] turned the current `system/user` findings into an `access` ownership contract

## Next Task Queue

- [ ] define how administrative user CRUD will be owned during the future `access` slice
- [ ] derive a concrete backend split checklist for `system/user` during the `access` slice
- [ ] derive a concrete frontend regrouping checklist for `system/role`, `system/menu`, and `system/user`
- [ ] decide whether the Phase 1 rollout plan should split admin user management into its own implementation slice or keep it inside `access`

## Source Documents For This Iteration

- `docs/README.md`
- `docs/specs/2026-04-23-admin-foundation-phase-1.md`
- `docs/specs/2026-04-23-identity-baseline.md`
- `docs/specs/2026-04-23-access-baseline.md`
- `docs/plans/2026-04-23-admin-foundation-phase-1-rollout.md`
