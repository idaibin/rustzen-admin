# User Role Assignment Readiness UI

The user-list header keeps only username and account-status filters, following the
shared automatic-search and upper-right placement contract. Email and real name
remain account fields in rows/forms; list-filter removal does not alter role assignment
or validation in the dialog. Loading and errors preserve editable filters and focus.

## Profile and evidence

- Profile: **Feature UI**. No shared token, component meaning, or product-wide
  visual-language change is required.
- Product basis: `docs/product/features/user-role-assignment-readiness/spec.md`.
- Current surface: the create-user and edit-user dialogs on the Admin user page.
- Reuse owners: the existing dialog/form composition, shared owners referenced
  from root `DESIGN.md`, and localized built-in role labels.
- Browser evidence covers role loading, populated data, forced failure and retry,
  successful empty data, field preservation, submission readiness, and wide and
  narrow dialog layout after implementation.

## Single job and direction

The operator must know whether role assignment is ready before attempting to
create or update an account. Keep the current compact administration-dialog
language: readable panel, medium density, semantic status color, and actions
next to the affected role region. Do not add decoration, another card system, or
a second dialog.

The role region is the one signature element for this slice: it visibly changes
between retrieval states while preserving the surrounding form. Visual emphasis
comes from status semantics and the retry action, not from animation or color
effects.

## Layout

```text
┌ Create or edit user ────────────────────────────┐
│ Description                                     │
│ Username / email / real name / password/status │
│                                                │
│ Roles                                           │
│ ┌ bounded role region ────────────────────────┐ │
│ │ loading | error + retry | empty | choices  │ │
│ └─────────────────────────────────────────────┘ │
│                              Cancel  Primary    │
└─────────────────────────────────────────────────┘
```

- Preserve the current dialog width and field order.
- The role region owns its own bounded vertical space and overflow. Populated
  choices remain scrollable; state feedback uses the same footprint so the
  footer does not jump materially between states.
- The dialog remains the scroll and focus boundary. Do not introduce page-level
  overflow or nested horizontal scrolling.
- At narrow widths, existing dialog padding and stacked footer behavior remain
  authoritative.

## State and copy contract

| State | Presentation | Primary action |
| --- | --- | --- |
| Loading | Compact localized `Loading roles` status with spinner inside the role region | Disabled |
| Populated | Existing checkbox list and localized role labels | Enabled only after normal form readiness and at least one selected role |
| Empty | Compact localized `No roles available to assign` status; no checkboxes | Disabled |
| Error or permission denial | Compact destructive status and localized `Reload` action | Disabled |
| Submitting | Existing form retained; localized creating or saving state | Disabled until completion |

The error description should tell the operator to retry or contact an owner if
access remains unavailable. Do not expose raw service errors as the only visible
copy. Existing global request feedback may remain additional evidence.

## Interaction and accessibility

- Opening the dialog starts retrieval and announces the loading state politely.
- Retry calls only role retrieval; it does not close the dialog or reset fields.
- The retry button is keyboard reachable and retains the existing visible focus
  treatment.
- Error feedback uses alert semantics; loading and empty feedback use status
  semantics through the existing shared component.
- Checkboxes keep their current label association and target size.
- The primary button stays disabled while roles are loading, failed, empty, or
  the account request is in progress.
- Reduced motion uses the existing system. The loading icon may retain the
  current shared reduced-motion behavior; no new motion is added.

## Shared-system changes

None. Reuse `DataState` in compact mode inside the current role container. Keep
the role-specific state composition local to the user route until another real
consumer proves a shared abstraction.

## Dev-frontend handoff

- Target only the user route's dialog and role picker composition.
- Preserve route paths, capability gates, account and role request contracts,
  list refresh, dialog field values, and existing localized role labels.
- Pass role query pending, error, retry, and successful data state explicitly to
  the role region.
- Disable submission until role data is successfully loaded and non-empty, in
  addition to the existing submission guard.
- Do not add a store, new API module, new shared component, or new token.

## Evaluation gates

- Loading, populated, empty, error, retry, and submitting states are all
  reachable from deterministic component/query inputs.
- No false empty state appears before a successful response.
- Retry preserves every other dialog value.
- Chinese and English fixed copy is present.
- No horizontal overflow at 1440×900 or 1920×1080; narrow-dialog behavior uses
  the existing responsive primitive.
- Frontend format, lint, typecheck, and build checks pass.
- Browser evidence verifies loading, populated, four-request forced failure,
  manual retry, successful empty data, field preservation, disabled submission,
  clean normal-path console/page errors, and 1920x1080 plus 390x667 overflow.
