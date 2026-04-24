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
- [x] derived a concrete backend split checklist for `system/user` during the `access` slice
- [x] derived a concrete frontend regrouping checklist for `system/role`, `system/menu`, and `system/user`
- [x] decided to keep admin user management inside the Phase 1 `access` slice
- [x] added an explicit “admin user management inside access” subsection under Slice 2 in the rollout plan
- [x] aligned the rollout plan task checklist with the documentation work already completed
- [x] decided to formalize the `audit` child spec before `system` and `runtime`
- [x] wrote a dedicated `audit` baseline spec under `docs/specs/`
- [x] added the new audit child spec to the repository documentation indexes
- [x] derived the current backend audit ownership checklist from `system/log` and login logging
- [x] added an explicit audit ownership subsection under Slice 3 in the rollout plan
- [x] decided to formalize the `system` child spec before `runtime` after the audit baseline
- [x] wrote a dedicated `system` baseline spec under `docs/specs/`
- [x] added the new system child spec to the repository documentation indexes
- [x] derived the current backend system ownership checklist from `system/dict` and config-related scope
- [x] added an explicit system ownership subsection under Slice 4 in the rollout plan
- [x] decided that `runtime` is ready to be formalized without another focused current-state audit
- [x] wrote a dedicated `runtime` baseline spec under `docs/specs/`
- [x] added the new runtime child spec to the repository documentation indexes
- [x] derived the current backend runtime ownership checklist from avatar upload and resource-serving paths
- [x] added an explicit runtime ownership subsection under Slice 5 in the rollout plan
- [x] confirmed the Phase 1 documentation loop is complete enough to stop the recurring documentation heartbeat
- [x] synced repository docs for the monorepo directory rename to `zen-core/`, `zen-server/`, and `zen-web/`
- [x] updated `README-zh.md` and `CHANGELOG.md` to clarify current vs historical directory naming
- [x] added `docs/specs/admin-foundation-phase-1.md` and linked it from documentation indexes
- [x] replaced login page banner image at `zen-web/src/assets/login-illustration.png` with the new visual asset
- [x] adjusted login banner presentation in `zen-web/src/routes/login.tsx` to improve focus, spacing, and visual hierarchy

## Next Task Queue

- [ ] start the first code-facing implementation slice from the Phase 1 documentation baseline

## Source Documents For This Iteration

- `docs/README.md`
- `docs/specs/admin-foundation-phase-1.md`
- `docs/specs/audit-baseline.md`
- `docs/specs/identity-baseline.md`
- `docs/specs/access-baseline.md`
- `docs/specs/system-baseline.md`
- `docs/specs/runtime-baseline.md`
- `docs/plans/admin-foundation-phase-1-rollout.md`
