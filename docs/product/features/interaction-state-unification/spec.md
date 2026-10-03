# Interaction state unification

## Outcome and authority

The requested 2026-10-03 Rustzen Admin quality pass covers existing clickable controls
and their real behavior. Baseline: `42d6cb104a68a484685f6e6d9372b1d8e8af9b97`.
Existing feature behavior and permission rules remain authoritative; this specification
clarifies cross-feature interaction acceptance, not new domain capabilities.
`DESIGN.md` remains the shared visual-semantic authority. The companion
[UI contract](../../../ui/features/interaction-state-unification.md) specifies the
input/state verification matrix; source and runtime evidence remain separate.

## Confirmed scope

- Nineteen authenticated routes, login, permission/not-found surfaces, shared shell,
  message center, confirmations, forms, table actions and existing navigation.
- Pointer hover and press feedback; keyboard activation, focus and selection;
  applicable disabled, pending, failure, retry and success states.
- Reuse Ant Design behavior and theme ownership. No replacement component library,
  new routes, API changes, permission expansion or decorative redesign.

## Required behavior

1. A visible enabled action is reachable by its appropriate pointer and keyboard
   interaction. Buttons activate with Enter/Space; links retain link behavior;
   composite menus, selectors and trees retain their library keyboard model.
2. Disabled means the action cannot run, including form submission paths that do not
   click its button. Business-disabled confirmation remains dismissible.
3. One pending non-idempotent action cannot be started again by rapid input, a native
   form submit, an alternate retry button, or the same action in another surface.
   Pending feedback starts immediately and clears on both success and failure.
4. Forms preserve entered data when submission fails. Failure remains recoverable;
   pending modal/drawer writes cannot be dismissed as though cancellation occurred.
   Controls that only read data show query-owned refreshing feedback and preserve
   stale data where the existing feature already supports it.
5. Opening an overlay places focus in it, Tab navigation stays within a modal, Escape
   dismisses when permitted, and closing restores a meaningful target. Removing the
   original row cannot leave keyboard focus stranded on the document body.
6. Search distinguishes text input from result selection and does not dispatch a
   result while Enter is committing an IME candidate.
7. Icon actions and form fields have programmatic names. Opening message detail moves
   focus to that detail; returning restores its opener; following the related record
   dismisses the message drawer so the result is visible.

## Acceptance boundary

Use fresh isolated test databases and self-created records for routed Web flows;
use synthetic fault fixtures for deterministic timing/error cases. Count actual route
components, shared-component inheritance, synthetic API behavior and unverified
backend/native effects separately. Build/tests do not prove browser focus or contrast.
Deletion and credential changes may affect only owned disposable test records.
Do not exercise production deletion, deployment, credential changes, or production services.
No main merge or production deployment is part of this pass.
