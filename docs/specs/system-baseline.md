# System Baseline Spec

> Status: proposed Phase 1 child spec for the `system` capability group

## Goal

Define the smallest reusable `system` baseline for `rustzen-admin` so global support data has a clear Phase 1 owner without turning `system` back into a catch-all capability bucket.

This spec narrows Phase 1 `system` to dictionary management and future system-configuration ownership.

## Ownership Contract

`system` owns:

- dictionary management
- dictionary option sources consumed by forms and pages
- future system-configuration ownership
- stable support data consumed by other capability groups

`system` does not own:

- login-state ownership
- role, menu, or user-access management
- audit records
- file upload and file metadata
- generic infrastructure utilities

`system` should own support data, not unrelated feature surfaces.

## Current Baseline Snapshot

### Backend

Current starting points:

- `zen-server/src/features/system/dict/mod.rs`
- `zen-server/src/features/system/dict/handler.rs`
- `zen-server/src/features/system/dict/service.rs`
- `zen-server/src/infra/config.rs`

Current baseline already present:

- dictionary CRUD
- dictionary status updates
- dictionary options endpoints
- dictionary-by-type lookup
- runtime configuration structure through `Config`

Current boundary issues:

- dictionary ownership still lives under the generic `system` grouping instead of an explicit `system` capability contract
- `Config` currently exists as infrastructure state, but Phase 1 product-facing system configuration management is not yet defined
- without a dedicated `system` spec, future config ownership could drift back into infra or into unrelated feature groups

### Frontend

Current starting points:

- `zen-web/src/routes/system/dict.tsx`
- `zen-web/src/api/system/dict/api.ts`
- `zen-web/src/api/system/dict/types.d.ts`

Current baseline already present:

- dictionary management page
- dictionary CRUD API
- dictionary option lookup API

Current boundary issues:

- the current frontend surface only covers dictionaries, not a broader system-config surface
- `system` still needs a clearer contract so it does not absorb access or audit pages again

## Phase 1 System Baseline

The Phase 1 `system` slice should establish one explicit capability owner across backend and frontend:

- backend owner: `zen-server/src/features/system/`
- frontend API owner: `zen-web/src/api/system/`
- frontend route owner: `zen-web/src/routes/system/`

The baseline must deliver:

- dictionary management
- dictionary options endpoints
- stable support data for downstream forms and pages
- a documented ownership path for future system configuration

The baseline may continue to rely on infrastructure configuration loading internally, but product-facing configuration ownership still belongs to `system`.

## Backend Boundary Rules

- keep dictionary CRUD and lookup behavior inside `system`
- keep support-data option sources inside `system`
- keep product-facing configuration ownership with `system`, not with generic infra helpers
- keep roles, menus, and user-access administration out of `system`
- keep logs out of `system`

## Frontend Boundary Rules

- keep dictionary management inside the `system` route group
- keep dictionary API ownership inside the `system` namespace
- reserve future system-config pages for the `system` route group
- keep access, audit, and identity pages out of the `system` route group

## Explicit Non-Goals

This spec does not introduce:

- feature-flag frameworks
- dynamic schema engines
- tenant configuration
- runtime file/resource ownership
- access-control UI

## Backend Implementation Checklist

- keep `zen-server/src/features/system/dict/` as the current Phase 1 anchor for `system`
- preserve dictionary CRUD, options, and type-based lookup under `system`
- define future product-facing system configuration management under `system`, not under `infra/config.rs`
- treat `infra/config.rs` as runtime wiring, not as the full product-facing system capability
- keep support-data contracts explicit and avoid turning `system` into a mixed owner for access or audit behavior

## Frontend Implementation Checklist

- keep `zen-web/src/routes/system/dict.tsx` as the current Phase 1 anchor for `system`
- keep `zen-web/src/api/system/dict/` as the current API anchor for `system`
- add future system-config pages under `zen-web/src/routes/system/`
- add future system-config API modules under `zen-web/src/api/system/`
- keep dictionary and future config surfaces separate from access, audit, and identity regrouping work

## Exit Condition

The `system` baseline is complete when `rustzen-admin` has a clearly bounded support-data capability centered on dictionaries, with a documented ownership path for future system configuration and no drift back into a generic catch-all bucket.
