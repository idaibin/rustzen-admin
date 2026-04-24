# Audit Baseline Spec

> Status: proposed Phase 1 child spec for the `audit` capability group

## Goal

Define the smallest reusable `audit` baseline for `rustzen-admin` so action history and traceability stop hiding under the generic `system` label and become an explicit Phase 1 capability.

This spec narrows Phase 1 `audit` to reusable admin action records and log-query surfaces.

## Ownership Contract

`audit` owns:

- login logs
- operation logs
- audit export for recorded logs
- reusable action-history query surfaces
- key admin action traceability

`audit` does not own:

- permission decisions
- session ownership
- dictionary data
- system configuration
- file upload and file metadata
- workflow history or reporting pipelines

`audit` records what happened. It does not decide whether an action is allowed.

## Current Baseline Snapshot

### Backend

Current starting points:

- `zen-server/src/features/system/log/mod.rs`
- `zen-server/src/features/system/log/handler.rs`
- `zen-server/src/features/system/log/service.rs`
- `zen-server/src/features/system/log/types.rs`

Current baseline already present:

- paginated log list
- CSV export for logs
- structured log write command
- login-event recording through `LogService::record_operation`

Current boundary issues:

- log ownership still lives under `system/log` instead of an explicit `audit` capability
- login flow in `zen-server/src/features/auth/handler.rs` directly writes `AUTH_LOGIN` records through the current system log service
- `audit` already has inbound dependencies from `identity`, but that relationship is not yet documented as a first-class capability boundary

### Frontend

Current starting points:

- `zen-web/src/routes/system/log.tsx`
- `zen-web/src/api/system/log/api.ts`
- `zen-web/src/api/system/log/types.d.ts`

Current baseline already present:

- log list page
- action-type filter
- log export action

Current boundary issues:

- the log page still lives under the `system` route group even though it is an `audit` surface
- the log API still sits under the `system` namespace instead of an explicit `audit` namespace

## Phase 1 Audit Baseline

The Phase 1 `audit` slice should establish one explicit capability owner across backend and frontend:

- backend owner: `zen-server/src/features/audit/`
- frontend API owner: `zen-web/src/api/audit/`
- frontend route owner: `zen-web/src/routes/audit/`

The baseline must deliver:

- login log capture
- operation log capture
- paginated log listing
- log export

The baseline may continue to consume actions emitted by `identity`, `access`, and future `system` or `runtime` flows, but that does not change `audit` ownership.

## Backend Boundary Rules

- keep log query and export behavior inside `audit`
- keep reusable log-write contracts inside `audit`
- keep login-event recording as an `audit` responsibility even when triggered by `identity`
- keep permission checks out of `audit`
- keep business workflow history out of Phase 1 `audit`

## Frontend Boundary Rules

- move the log page into a dedicated `audit` route group
- move the log API into a dedicated `audit` namespace
- keep audit pages read-oriented in Phase 1
- keep role, menu, and user-management actions out of the audit route group

## Explicit Non-Goals

This spec does not introduce:

- async audit pipelines
- archive tiers
- audit analytics center
- workflow trace viewers
- business-domain event history

## Backend Implementation Checklist

- create `zen-server/src/features/audit/` as the Phase 1 owner of log list, export, and structured log write behavior
- move `system/log` route ownership into the future `audit` capability name
- keep `LogWriteCommand` as the reusable write contract for Phase 1 audit records
- preserve `AUTH_LOGIN` logging from the current auth flow while moving ownership naming from `system/log` to `audit`
- keep `LogService::record_operation()` as the integration point consumed by `identity` and later by `access`
- keep partition management and export behavior inside `audit`

## Frontend Implementation Checklist

- create `zen-web/src/api/audit/` as the dedicated client surface for audit ownership
- move `zen-web/src/routes/system/log.tsx` into the future `audit` route group
- move `zen-web/src/api/system/log/` into the future `audit` API namespace
- preserve the existing action filter and export behavior during regrouping
- keep the audit UI read-oriented and avoid mixing write actions into it

## Exit Condition

The `audit` baseline is complete when `rustzen-admin` has a dedicated audit surface across backend and frontend, and login plus operation logging are documented as `audit` ownership rather than a generic system utility.
