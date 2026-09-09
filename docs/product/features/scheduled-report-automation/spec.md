# Scheduled Report Automation

The owning module is presented to users as **自动化 / Automation**. `Reports`
remains its internal service name; target-backed browser execution, routes,
permissions, templates and run data are unchanged.

## Goal and implementation slice

Allow an operator to repeat an existing Reports flow on a predictable daily or
weekly cadence and inspect the resulting run without introducing a general
workflow engine. A schedule creates an ordinary queued Reports run for one
existing target-backed flow; a missed occurrence is skipped rather than
replayed.

The current Reports worker already claims queued runs and retains steps,
artifacts, live frames, cancellation, and cleanup. This slice adds the
smallest scheduling policy around that loop and keeps browser execution owned
by Reports.

Reports browser execution runs under the dedicated unprivileged `rz-reports`
service account with Chromium sandboxing enabled. Browser sessions use a
deterministic 1440x900 viewport so operator flows and their screenshot evidence
do not depend on a host default. The run target is activated before its first
navigation so page visibility and lifecycle APIs describe the target under
test rather than Chromium's unused bootstrap tab. A screenshot is rejected when
it exceeds 4,000,000 pixels or 4 MiB, and one run may retain at most 16 MiB of
screenshot artifacts; rejected captures leave no artifact row or temporary
file. These execution limits apply equally to ordinary and scheduled runs.


## Evidence status

Historical successful commands, screenshots, and assertions are not current acceptance. A local verification claim requires a source-bound current manifest whose source identity, platform, declared receipts, artifact hashes, and terminal status match the checkout under review. If that manifest is absent, incomplete, or mismatched, the related browser, HTTP, or runtime statement is **Not verified**.

Native systemd startup, browser seccomp under a restricted native Linux profile, deployment after installation, and a deployed scheduler reaching its due time remain **Not verified**.

## Current implementation and verification status

The Reports service and the Reports Templates route now implement this slice:
daily/weekly schedule CRUD, enable/disable, schedule-only view/manage
capabilities, installation-timezone readback, durable occurrence decisions,
and the `enqueued`-to-run link are present. The user-facing module name is
**自动化 / Automation** while Reports remains the service, route, capability,
template, and run-data owner.

Repository tests and local gates are intended to exercise scheduler timing,
idempotency, stale-snapshot handling, persistence, daily/weekly HTTP CRUD,
capability denial, occurrence/run linkage, and browser-rendered Templates
flows. The target-backed Linux Chromium gate is intended to create a
next-minute daily schedule, wait for the persisted `enqueued` occurrence and
`runId`, and follow the Templates link to its exact Runs audit. Its isolated
SQLite `missed` fixture is limited to rendering a due instant, reason, and
no-link state; it does not exercise the scheduler's create-time `effectiveAt`
admission. The Web seams are intended to cover selector and schedule
view/manage gates, terminal retry visibility, shared retry pending state,
direct-child selection, failure preservation, and save retry behavior. The
four-service gate is intended to exercise startup ordering, failure isolation,
gateway contracts, and service database restore.

Those gate outcomes establish HTTP, browser, or local Linux claims only when a
source-bound current manifest matches this checkout. Without that manifest,
the rendered Web route and all gate-derived runtime behavior are **Not
verified**. Real systemd boot and browser seccomp under a confined native Linux
host remain **Not verified**.

Reports flows may use the bounded `assertValue` and `assertAbsent` steps in
addition to the existing browser DSL. `assertValue` compares an element's
native value after normal selector validation and template substitution.
`assertAbsent` succeeds only when Chromium reports the selector as not found;
an invalid selector, CDP fault, or browser failure remains a failed run. These
steps do not allow arbitrary page evaluation. The Rust flow schema and the
handwritten Web `Reports.FlowStep` declaration carry the same two variants.

## Users and scenarios

- An owner or permitted report operator enables a daily or weekly schedule for
  an existing flow with valid non-sensitive input.
- An operator can see whether a schedule is enabled, its next due time, its
  last occurrence decision, and the run created only for an enqueued occurrence.
- A viewer can inspect schedules and runs but cannot create, edit, enable,
  disable, or delete a schedule.
- If the service is down at a due time, the next scheduler pass records the
  occurrence as skipped; it does not create a burst of catch-up runs.
- A disabled target produces a visible failed `enqueued` run when the existing
  Reports execution contract accepts it; an invalid flow is rejected before an
  occurrence decision. A skipped occurrence has due/reason only and no run.

## Confirmed decisions and rationale

| Decision | Rationale | Acceptance consequence |
| ------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Daily and weekly are the only cadence choices. | They cover predictable reporting without arbitrary cron complexity. | The form cannot accept seconds, free-form cron, or an unbounded interval. |
| A schedule points to one existing target-backed flow. | Reports already owns target, flow, and run semantics. | No duplicate template, target, or browser DSL is introduced. |
| Missed occurrences are skipped. | Catch-up can overload a small self-hosted installation and hide freshness. | A missed occurrence is visible as skipped with a reason; no retroactive queue flood. |
| Only an `enqueued` occurrence creates an ordinary queued run. | Existing run evidence, cancellation, and retention remain reusable. | Runs use the current status lifecycle and artifact boundary; `skipped` has no run. |
| Installation timezone is the schedule display timezone. | A single self-hosted installation needs one unambiguous clock. | UI shows the timezone; per-schedule timezone is not a first-slice field. |
| No schedule-owned credentials, notification controls, or webhooks. | Those require separate protected storage and delivery policies. | Secret-looking input remains rejected; scheduled runs keep a null personal-notification initiator. |

## Occurrence decision and run boundary

An occurrence decision is separate from a Reports run. Each schedule/time slot
has one durable decision:

| Decision | Required data | Run relationship |
| ---------- | ---------------------------------------------------------------- | ----------------------------------------------------------- |
| `enqueued` | schedule, resolved due instant, decision time, and run reference | Exactly one ordinary Reports run is created and referenced. |
| `skipped` | schedule, resolved due instant, decision time, and a reason | No run reference exists and no run is created. |

An enqueue transaction either commits the `enqueued` decision and its one run
together or commits neither. A polling or service error leaves the slot
undecided and safely retryable; it must not create a run without an `enqueued`
decision. Run status (`queued`, `running`, `succeeded`, `failed`, `cancelling`,
or `cancelled`) belongs to the existing Reports run lifecycle and is not copied
into the schedule decision.

## Terminal-run retry boundary

A failed or cancelled terminal run may be retried by an operator with
`reports:run:manage`. `POST /api/reports/runs/{id}/retry` reads the source
run's persisted `flow_id` and `input_json` snapshot and creates one new,
independent manual queued run. The source run, its steps, artifacts, error and
timestamps remain unchanged.

Retry is idempotent per retained direct source run. A source run has at most one
direct retry child while the source exists. Repeated or concurrent retry requests return that
same child, including after the child has reached a terminal state; they never
create a replacement child. If that child fails or is cancelled, an operator
retries the child itself to create the next link in the retry chain.

Retry never updates a schedule occurrence, changes its unique
`schedule occurrence -> run` association, or evaluates a due slot. Retrying a
run created by an `enqueued` occurrence is therefore not catch-up execution:
the new manual run has no occurrence row. Queued, running, cancelling and
succeeded runs without an existing retry child are rejected. The response
returns the created or already-existing direct child snapshot so the Web Runs
list or detail can navigate to it.

Retention may delete an older source run while retaining a newer child. In that
case the child's lineage reference is cleared, the deleted source cannot be
retried, and no lineage guarantee is made across the deleted record.

The rendered retry gate is intended to use a real failed source and compare the
returned direct child by ID rather than assume it remains queued. A Runs-list
retry is expected to select that child audit; retrying from the failed source
audit is expected to select the same direct child. The source run response,
steps, and artifacts must remain unchanged across both actions. Succeeded and
nonterminal runs have no rendered Retry action. A user with
`reports:run:view` but no `reports:run:manage` cannot see Retry and receives
the backend permission rejection. A Runs-only viewer must not trigger a flows
request requiring `reports:flow:view`; the flow column uses the persisted
`flowId` when that optional name lookup is not authorized. Run creation retains
its managed flow-picker behavior.

The target-backed browser gate is intended to cover 1440x900 dark/en-US and
390x844 light/zh-CN, key copy, no horizontal overflow, and no permission error
toast, alongside schedule and fault checks. This rendered acceptance is **Not
verified** unless its current source-bound manifest matches the checkout.

The scheduler resolves local schedule time using the installation timezone and
applies one fixed 60-second lateness window:

- A poll at or after the due instant and no later than 60 seconds after it may
  enqueue the slot once.
- A poll more than 60 seconds late records `skipped` with reason `missed`; it
  never creates a catch-up run.
- A process restart uses persisted decisions and the same window. A committed
  `enqueued` slot is never enqueued again; an undecided slot is evaluated once
  against the current time and then enqueued or skipped.
- A daylight-saving **gap** (a local time that does not exist) records one
  `skipped` decision with reason `dst_gap` and advances to the next valid slot.
- A daylight-saving **fold** (a local time that occurs twice) resolves to the
  earlier offset and creates one slot only; the later duplicate is not a second
  occurrence.

## Scope and non-goals

In scope:

- create, view, update, enable, disable, and delete daily/weekly schedules;
- schedule fields for flow, cadence, local due time, selected weekday when
  weekly, safe input, enabled status, and description;
- next due, last decision, last run when the decision is `enqueued`, skipped
  occurrence, and failure reason;
- idempotent due occurrence creation and ordinary queued run execution;
- list/detail, validation, retry/readback, and explicit destructive confirmation
  for deletion where the existing product permits deletion;
- retention and artifacts through the existing Reports cleanup owner.

Non-goals:

- arbitrary cron, sub-minute schedules, dependency graphs, parallel workflow
  orchestration, or a fifth scheduler service;
- account credential storage, secret substitution, schedule notification
  channels, webhooks, external queues, or report-center catalog behavior;
- changing browser actions, target validation, run cancellation, artifact
  contents, or run input redaction rules;
- catch-up execution after downtime or a timezone per schedule;
- importing schedules from a former repository or compatibility wrappers.

## Main and failure flows

1. The operator opens the schedule panel on the existing Reports templates or
   runs surface. The list loads with explicit state semantics.
2. Creating or editing validates that the target flow exists, cadence fields
   are complete, and input contains no recognized secret field. Save returns
   the schedule to the list with its next due time. A create or update request
   failure stays visible inside the open form, retains every entered field, and
   lets the operator correct or retry the same submission; it does not close
   the dialog or reset the draft.
3. At a due occurrence, Reports resolves the local time and applies the 60-
   second window. It atomically records `enqueued` and creates one normal run,
   or records `skipped` with a separately rendered due instant/reason and no run. Repeated polls, restarts,
   DST gaps, and DST folds cannot create a duplicate occurrence.
4. A successful run links from an `enqueued` schedule decision to the existing run detail,
   steps, live frame, and artifacts. A target-backed Linux Chromium flow creates
   a next-minute schedule, bounded-waits for that real decision and run ID, then
   opens the exact run audit through the rendered Templates link. A failed run
   keeps its error evidence.
5. A missed occurrence is recorded as `skipped` with only its
   installation-timezone due instant and reason. It has no run reference and is
   not silently represented as `succeeded`.
6. A schedule query or save failure shows an explicit error and retry while
   preserving entered values. Background refresh keeps the last good rows.

## Business rules and permissions

- `reports:schedule:view` gates schedule list/detail; `reports:schedule:manage`
  gates create/update/enable/disable/delete. Backend checks remain authoritative.
- A view-only schedule list does not assemble an actions column. It keeps each
  due time and timezone fully readable; mutation controls and their fixed table
  width are absent rather than CSS-hidden.
- Owner receives both boundaries. Admin receives them only through the normal
  Reports management policy. Viewer receives view only. Custom roles follow
  the existing generated capability catalog.
- At create or update time, a schedule cannot reference a missing, disabled, or
  invalid flow. If an already scheduled flow becomes disabled before execution,
  the created run fails according to current Reports run semantics.
- One schedule has at most one decision per occurrence. Only `enqueued` carries
  a run reference; `skipped` carries due/reason and no run. Retries must not
  duplicate a run or invent a skipped run.
- The installation timezone is displayed and applied consistently. Stored
  timestamps remain unmodified UTC values.
- Secret-looking fields remain rejected by the existing Reports input policy;
  scheduling must not create a path around that boundary.
- Credential-shaped input names, including `password`, `pwd`, `pass`, `token`,
  `secret`, `auth`, `bearer`, and `key` aliases with prefixes or suffixes, are
  rejected before a schedule or run is persisted.

## UI states and evidence

The UI contract is [Scheduled Report Automation UI](../../../ui/features/scheduled-report-automation.md).
It extends the existing Reports templates/runs surfaces and reuses their
tables, forms, dialogs, and run evidence.

| State | User-visible meaning | Required behavior |
| ---------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Loading | Schedule list or form data is loading. | Keep the current surface stable; do not show false empty. |
| Populated | Schedules or run links are available. | Show cadence, timezone, next due, and last occurrence decision; show a run link only for `enqueued`. |
| Empty | Query succeeded with no schedules. | Explain how to create one when permitted. |
| Error | List/save/trigger read failed. | Show retry; preserve form values and last good rows. |
| Permission | Caller lacks view/manage capability. | A view-only list omits the actions column; browser acceptance at 390px asserts the complete `10:16 · UTC` due value and no horizontal overflow. |
| Processing | Save, enable/disable, or delete is running. | Disable duplicate actions and retain the selected schedule. |
| Partial | A list contains mixed occurrence outcomes. | Keep each outcome and reason; never summarize as all successful. |

### SR-UI-002 rendered form acceptance

The dedicated SR-UI-002 Linux Chromium gate uses a fresh Admin/Reports data
root and the real Templates route. It is separate from the wider Admin browser
gate so its form assertions and evidence can evolve without enlarging that
general-purpose verifier.

The SR-UI-002 assertions are current evidence only when its source-bound `current`
manifest matches this checkout. Otherwise its Colima platform, screenshots, and form
assertions are **Not verified**.

- A schedule manager at 1440x900, dark/en-US sees the installation timezone and
  creates a daily schedule, then changes it to weekly with a weekday. The
  persisted row must show the matching cadence, local due time, and installation
  timezone.
- Local incomplete or malformed-object input failures do not issue a schedule
  mutation and do not change the schedule count. The gate records the proxy
  mutation count and the before/after API row count. A secret-looking input is
  intentionally a server policy rejection: it reaches the Reports API, creates
  no row, retains the dialog draft, and is recorded separately from the local
  no-request cases.
- Cancelling a create dialog restores focus to its `schedule-create` trigger;
  a successful edit restores focus to that schedule's `schedule-edit` trigger.
  These are direct browser active-element assertions, not an inference from a
  subsequent click.
- A schedule-view-only user at 390x844, light/zh-CN sees the schedule panel and
  timezone copy but no management trigger or form. Its capture asserts no
  horizontal overflow.

For a create or update save failure, the form shows the returned error when
available, otherwise localized save-failure guidance. This is distinct from
request-layer feedback: the schedule mutation suppresses its global error toast
and the form is the sole presentation owner. The form-level error remains
visible while the dialog is open, and retry uses the retained draft without
reinitializing it. Each open dialog cycle owns its save callbacks; a late
completion from a closed cycle cannot close, announce, or overwrite a reopened
draft.

## User-visible data effects

Schedules and occurrence decisions are stored in Reports-owned SQLite. Only an
`enqueued` decision creates one existing Reports run and its normal
steps/artifacts; a `skipped` decision stores due/reason and no run reference.
No Admin database or other module database is joined. Deleting a schedule does
not delete historical runs or artifacts unless a separate existing retention
policy removes them.

## Affected product surfaces and dependencies

- Reports owns scheduling decisions, cadence, due-occurrence identity, run
  creation, browser execution, failure evidence, and retention.
- Admin owns delegation, capability reconciliation, and the Web shell.
- `apps/web` owns the schedule panel in Reports templates/runs and its API
  client; route-local form/table state remains local until a second consumer
  proves reuse.
- Reports HTTP contract chain is `Rust ModuleRouter/Manifest -> handwritten
  apps/web/src/api/reports/contract.ts -> scripts/verify-worker-contracts.mjs`.
  It does not use the Admin OpenAPI/Orval chain.
- Existing `PageCard`, `DataState`, `DataTableShell`/`ProTable`, Ant Design
  `Form`/`Modal`/`Select`/`Tag`, `ConfirmDialog`, and `formatDateTime` remain
  the component owners.

## Acceptance criteria

- An authorized operator can create and edit a daily or weekly schedule for an
  existing flow and sees the installation timezone and next due time.
- Invalid cadence, missing flow, disabled target, or secret-looking input is
  rejected with localized guidance and no schedule/run side effect.
- A due occurrence produces one `enqueued` decision plus one ordinary queued
  run, or one `skipped` decision containing due/reason and no run; scheduler
  retries cannot duplicate either decision.
- A poll more than 60 seconds late, a restart after that window, a DST gap, or
  a DST fold follows the fixed one-time rules above and never enqueues a
  catch-up/duplicate run.
- Successful, failed, and cancelled `enqueued` outcomes link to or preserve the
  existing Reports run evidence; `skipped` preserves due/reason with no run;
  a mixed list remains visibly partial.
- Only failed or cancelled terminal runs can create a first direct retry child.
  Repeat or concurrent requests return that same child forever; retrying a
  terminal child creates the next chain link. Every child copies its direct
  source's persisted flow/input snapshot, and source evidence plus schedule
  occurrence associations remain unchanged.
- The Runs list and detail use the same retry state owner: the source run ID
  determines one shared pending key, success selects the returned direct child,
  and failure leaves the source selected while exposing the returned error.
- Viewer cannot mutate a schedule, and direct backend mutation without the
  manage capability is rejected.
- Disable/enable and destructive delete have explicit processing and
  confirmation states; historical runs remain intact after schedule deletion.
- Background refresh and retry preserve filters, form values, and last good
  rows; initial failure is never shown as an empty list.
- Fixed copy is localized in Simplified Chinese and English; user-created
  names and runtime error text remain unchanged.
- The linked UI matrix is intended to cover desktop/narrow, light/dark,
  loading/empty/error/permission/processing/partial, keyboard focus,
  localization wrapping, and no-overflow checks; it is **Not verified** without
  a matching current source-bound manifest.

## Verification matrix

| Layer | Evidence | Acceptance |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Source/static | schedule lifecycle, due identity, capability, and client mapping review | No cron parser, secret bypass, duplicate route catalog, or cross-service DB access. |
| Automated | Reports scheduler/service, persistence, input-safety, and contract tests | Intended to cover daily/weekly, skip, idempotency, and failure evidence; results require current test output. |
| HTTP | Focused worker verifier creates, lists, reads, retries terminal runs, updates, enables/disables, and deletes daily/weekly schedules, then reads real occurrence/run state | Intended to cover schedule view/manage denial, retry denial for non-terminal runs, `enqueued`/`skipped`, source immutability, and run linkage. It is **Not verified** without a matching current source-bound manifest. |
| Browser | Reports browser verifier, SR-UI-002 form gate, schedule permission seam, and Reports state-closure gate | Intended to cover Templates create/edit/disable/enable/delete, schedule-view-only hiding, next-minute `enqueued` linkage, manager no-request failures, secret-policy rejection, timezone/cadence display, focus restoration, two viewports, `missed` rendering, retry behavior, processing/partial/runtime-failure/view-only rendering, and retained evidence. It is **Not verified** without a matching current source-bound manifest. |
| Runtime/deployment | four-service verifier plus Colima Linux Reports gate | A matching current source-bound manifest is required for local four-process and Linux browser evidence; otherwise it is **Not verified**. Real systemd and native-host browser seccomp remain **Not verified**. |

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- The existing Reports run queue remains the only execution queue.
- The installation timezone is configured and readable by the Reports process.

### Open questions

- Whether an operator needs a manual `run now` action for a schedule is left to
  the existing Reports run-manage capability and is not required for this
  scheduling slice.

### Rejected

- Arbitrary cron strings and catch-up storms.
- Storing credentials in schedule input or using hidden browser-side secrets.
- Treating a missed or failed occurrence as success.

### Deferred

- Credential vaulting, schedule-configured notifications, webhooks, datasets, richer expression
  DSL, suspend/resume, and cross-module Report Center behavior.

## Ready for scheduled report automation implementation

The cadence, skip policy, run ownership, permission boundary, failure semantics,
and non-goals are implemented in the current Reports/Web slice. Focused local
Rust, HTTP, and script seams are required before this status can be claimed.
The stated Templates success lifecycle, state-closure rendering, and two
viewports require matching current manifests; otherwise they are **Not verified**.
Other response matrices, real systemd installation, native-host browser seccomp,
deployment after installation, and deployed due-time scheduling remain **Not verified**.

## Current Linux Chromium acceptance

The release gate is intended to exercise the target-backed Reports schedule
lifecycle: create daily, edit weekly, disable, enable, and delete. It is also
intended to cover schedule-view-only management hiding, desktop dark/en-US and
mobile light/zh-CN screenshots without horizontal overflow, and Runs retry from
list and audit surfaces with direct-child selection, terminal visibility,
view-only denial, and source-evidence preservation. The existing ten
mutation-failure cases remain required and do not substitute for these success
paths. These claims are **Not verified** unless a matching current
source-bound manifest is present. Other visual matrices, native systemd, and
native-host seccomp remain **Not verified**.

## Reports state-closure Linux Chromium acceptance

`just verify-reports-ui-state-linux` is a local state-closure gate. Its assertions
are accepted only from a current source-bound manifest that matches this checkout;
without that manifest they are **Not verified**. When run against a matching
manifest, the manager desktop flow is intended to hold a controlled 30-second
executor pause, capture active processing, cancel it, and retain the cancelled
pause-step receipt. It is also intended to capture terminal runtime failure,
its failed step and error, and the Retry action. Before and after retry it
compares immutable source run, steps, and artifacts snapshots by file hash, so
opening a retry child cannot replace retained source evidence. The manifest
binds each of the four browser runs to its complete run-step receipt and its
uniquely named, exact-viewport screenshot.

The same gate adds two controlled rendering fixtures to the disposable Reports
SQLite database: one `enqueued` occurrence with a real run link and one
`skipped` occurrence with a distinct due/reason and no link. With a matching
manifest, these fixtures are intended to cover the Templates partial summary
and row-level distinction only; they are not scheduler API history. A single
view-only role receives schedule and run read capabilities, sees neither `schedule-create`, `schedule-edit`,
`schedule-toggle`, `schedule-delete`, `run-create`, `run-cancel-*`, nor Retry
controls at the mobile surface, and receives 403 responses for the
corresponding direct mutations. This local gate does not verify deployed
scheduling, native systemd, or confined native-host browser seccomp; those
paths remain **Not verified**.
