# User Role Assignment Readiness

## Goal and implementation slice

Make role assignment in the create-user and edit-user dialogs understandable
before submission. The slice distinguishes role loading, load failure, a valid
empty result, and available roles without changing the role or permission model.

## Users and scenarios

- An authorized operator opens the create-user dialog and must assign at least
  one enabled role before creating the account.
- An authorized operator opens the edit-user dialog and must understand whether
  the existing role selection is ready to review or change.

## Confirmed decisions

- Creating a user requires at least one selected role.
- The role selector presents enabled assignable roles using localized built-in
  role names and unchanged user-created role names.
- Non-owner operators do not receive the owner role as an assignable option;
  direct attempts to submit an empty role set or assign owner without owner
  authority fail at the backend boundary.
- The interface supports Simplified Chinese and English.
- Loading, empty, error, permission, and retry states must not be presented as
  successful empty data.
- This slice improves an existing Admin journey; it does not add a new product
  capability or widen role-management permissions.

## Scope and non-goals

In scope:

- show a distinct loading state while roles are being retrieved;
- show a distinct failure state with a retry action when roles cannot be loaded;
- explain a successful empty result and prevent submission that cannot satisfy
  the role requirement;
- preserve selected roles and normal dialog input while retrying;
- keep the last successful role choices visible with a retry notice after a
  background refresh failure, preserving the form and selected roles;
- preserve the edit draft when the same user record refreshes while the dialog
  remains open; initialize from the latest record after close and reopen, and
  isolate drafts when switching users or opening a create session;
- prevent duplicate submission while role data is not ready, fetching, has a
  failed refresh, or the form is already being submitted.

Non-goals:

- changing the built-in role taxonomy or capability model;
- changing request or response shapes for user, role, or permission APIs;
- adding role creation inside the user dialog;
- redesigning the user list, global form system, or permission model;
- changing error envelopes or unrelated account validation.

## Main and failure flows

1. Opening the dialog begins role retrieval and shows a bounded loading state in
   the role area.
2. When roles load, the operator can review and change the selection.
3. When no enabled assignable role exists, the role area explains that result
   and submission remains unavailable.
4. When retrieval fails, the role area shows an error and retry action; the
   dialog remains open and other entered values are preserved.
5. If a background refresh fails after roles have loaded, keep the last
   successful role choices visible with the shared refresh notice and retry
   action; preserve dialog values and selected roles, and block submission.
6. A successful retry clears the error and makes the role choices ready.
7. Form validation still explains that at least one role is required before a
   ready form can be submitted.
8. Refreshing the same user does not overwrite an open edit draft; closing and
   reopening uses the latest user record. Editing another user starts from that
   user's record, and each create session starts with empty account fields and
   role selection.

## Business rules and permissions

- Existing create-user and update-user visibility rules remain authoritative.
- Failure to retrieve role choices never grants access or substitutes an empty
  role assignment.
- The backend rejects an empty role set for create and update requests.
- The client must not claim that no roles exist when retrieval failed or is
  still in progress.
- A server permission denial is presented as a role-loading failure; this slice
  does not infer or repair permission grants.

## UI states and evidence

- **Loading:** concise progress feedback inside the role selector region.
- **Populated:** current checkbox selection with localized built-in role labels.
- **Empty:** successful retrieval with no assignable roles and an explanation
  that creation or update cannot proceed.
- **Error or permission denial:** failure message and retry action without
  closing or resetting the dialog.
- **Background refresh failure:** cached roles and current selection remain
  visible with the shared refresh notice and retry action; submission stays
  disabled until refresh succeeds.
- **Submitting:** primary action disabled until the current request finishes.

## User-visible data effects

No new data is stored. A successful create or update keeps the existing account
and role-assignment behavior.

## Affected product surfaces and dependencies

- Admin user management create and edit dialogs.
- Existing shared loading, empty, error, and retry vocabulary.
- Existing role option retrieval and account submission behavior.

## Acceptance criteria

- Opening either dialog never shows a false empty-role result while retrieval is
  pending.
- A failed role request displays an error and a working retry action.
- A failed background refresh keeps the last successful role choices visible,
  preserves all form fields and selected roles, and disables submission until a
  successful retry completes.
- Refreshing the same user while editing preserves all current field values and
  role selection. Close/reopen loads the latest record; switching users isolates
  the form from the prior draft, and create opens with empty values.
- Retrying does not discard username, email, real name, password, status, or
  selected role input already present in the dialog.
- A successful empty response is visually distinct from failure and prevents a
  submission that requires a role.
- A populated response preserves current role-selection and account submission
  behavior.
- All new fixed copy has Simplified Chinese and English variants.
- Existing permission visibility, request contracts, and list refresh behavior
  remain unchanged.

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- The existing role-option response remains the authority for enabled assignable
  roles.
- The same readiness states are appropriate for both create and edit dialogs.

### Open questions

- Whether a future product flow should let an authorized operator create a role
  from this dialog. This does not block the current slice.

### Rejected

- Treating request failure as an empty role list.
- Automatically submitting a user without roles.
- Expanding this slice into role-management or permission-policy changes.

### Deferred

- Field-level server-error mapping beyond the existing global request feedback.
- A broader form-validation framework.

## Ready for user-dialog role-readiness implementation

The slice is ready. Product behavior, failure semantics, permission boundaries,
non-goals, and acceptance are fixed. Visual composition and implementation
details remain with the UI and frontend owners.
