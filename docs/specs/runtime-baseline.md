# Runtime Baseline Spec

> Status: proposed Phase 1 child spec for the `runtime` capability group

## Goal

Define the smallest reusable `runtime` baseline for `rustzen-admin` so file-resource handling has a clear Phase 1 owner without turning `runtime` into a generic infrastructure bucket.

This spec narrows Phase 1 `runtime` to file upload, resource path conventions, and static resource exposure.

## Ownership Contract

`runtime` owns:

- file upload handling
- file-size and file-type validation for runtime-managed files
- resource path conventions for uploaded files
- static resource exposure for runtime-managed files
- file-storage directory layout used by runtime-managed resources

`runtime` does not own:

- login-state ownership
- role, menu, or user-access management
- audit records
- dictionary data or system configuration
- generic infrastructure helpers unrelated to file resources
- task orchestration, messaging, or cache coordination

`runtime` should own file resources, not every low-level helper in the repository.

## Current Baseline Snapshot

### Backend

Current starting points:

- `zen-server/src/common/files.rs`
- `zen-server/src/infra/app.rs`
- `zen-server/src/infra/config.rs`
- avatar upload path in `zen-server/src/features/auth/handler.rs`

Current baseline already present:

- avatar upload handling
- file-type validation for avatar uploads
- file-size validation for avatar uploads
- resource path construction for uploaded avatars
- static serving for `/resources` and avatar directories

Current boundary issues:

- file handling currently sits across `common/`, `infra/`, and `auth` instead of a dedicated `runtime` capability owner
- the current product-facing upload path is avatar-specific, so the baseline is still narrower than the long-term Phase 1 `runtime` target
- without a dedicated `runtime` spec, file-resource ownership can drift into identity or generic infrastructure

### Frontend

Current starting points:

- `zen-web/src/components/base-user/avatar.tsx`
- `zen-web/src/store/useAuthStore.ts`

Current baseline already present:

- avatar upload UI
- client-side image-type checks
- client-side size checks
- avatar URL update after successful upload

Current boundary issues:

- the current frontend file surface is embedded in the identity/profile experience instead of a dedicated runtime route or API namespace
- there is no dedicated `zen-web/src/api/runtime/` capability surface yet

## Phase 1 Runtime Baseline

The Phase 1 `runtime` slice should establish one explicit capability owner across backend and frontend:

- backend owner: `zen-server/src/features/runtime/`
- frontend API owner: `zen-web/src/api/runtime/`
- frontend route owner: `zen-web/src/routes/runtime/`

The baseline must deliver:

- single-file upload handling
- resource path conventions
- file metadata or file-location contract for uploaded resources
- static resource exposure for uploaded files

The baseline may continue to be consumed first by identity avatar flows, but that does not make `identity` the owner of file-resource handling.

## Backend Boundary Rules

- keep file upload behavior inside `runtime`
- keep file validation rules inside `runtime`
- keep resource-prefix and runtime-managed file path conventions inside `runtime`
- keep auth/profile flows as consumers of runtime file handling, not owners of it
- keep unrelated runtime wiring out of the `runtime` product capability

## Frontend Boundary Rules

- treat avatar upload as a current consumer of `runtime`, not the owner of runtime
- add future runtime API surfaces under `zen-web/src/api/runtime/`
- reserve future runtime pages or tools for `zen-web/src/routes/runtime/`
- keep access, audit, and system support-data pages out of the runtime route group

## Explicit Non-Goals

This spec does not introduce:

- multipart upload
- object-storage abstraction
- media pipelines
- notification delivery
- task scheduling
- generic infrastructure registries

## Backend Implementation Checklist

- create `zen-server/src/features/runtime/` as the Phase 1 owner of file-resource behavior
- move reusable file upload handling out of `common/files.rs` into the future runtime owner
- keep `infra/config.rs` and `infra/app.rs` as runtime wiring while moving product-facing file ownership into `runtime`
- preserve current avatar upload behavior while changing ownership from auth-adjacent helpers to runtime
- keep `/resources` path conventions explicit during the move

## Frontend Implementation Checklist

- create `zen-web/src/api/runtime/` as the dedicated client surface for file-resource ownership
- treat `base-user/avatar.tsx` as a consumer of runtime upload behavior
- preserve current avatar upload validation and response handling during regrouping
- add future runtime pages or tools under `zen-web/src/routes/runtime/` only when file-resource features expand beyond avatar upload

## Exit Condition

The `runtime` baseline is complete when `rustzen-admin` has a clearly bounded file-resource capability with explicit upload and resource-path ownership, and avatar upload is documented as a consumer of runtime rather than the owner of it.
