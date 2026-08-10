# Scheduled Report Automation

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
| --- | --- | --- |
| Daily and weekly are the only cadence choices. | They cover predictable reporting without arbitrary cron complexity. | The form cannot accept seconds, free-form cron, or an unbounded interval. |
| A schedule points to one existing target-backed flow. | Reports already owns target, flow, and run semantics. | No duplicate template, target, or browser DSL is introduced. |
| Missed occurrences are skipped. | Catch-up can overload a small self-hosted installation and hide freshness. | A missed occurrence is visible as skipped with a reason; no retroactive queue flood. |
| Only an `enqueued` occurrence creates an ordinary queued run. | Existing run evidence, cancellation, and retention remain reusable. | Runs use the current status lifecycle and artifact boundary; `skipped` has no run. |
| Installation timezone is the schedule display timezone. | A single self-hosted installation needs one unambiguous clock. | UI shows the timezone; per-schedule timezone is not a first-slice field. |
| No credentials, notifications, or webhooks. | Those require protected storage and delivery contracts not present today. | Secret-looking input remains rejected and delivery integrations stay deferred. |

## Occurrence decision and run boundary

An occurrence decision is separate from a Reports run. Each schedule/time slot
has one durable decision:

| Decision | Required data | Run relationship |
| --- | --- | --- |
| `enqueued` | schedule, resolved due instant, decision time, and run reference | Exactly one ordinary Reports run is created and referenced. |
| `skipped` | schedule, resolved due instant, decision time, and a reason | No run reference exists and no run is created. |

An enqueue transaction either commits the `enqueued` decision and its one run
together or commits neither. A polling or service error leaves the slot
undecided and safely retryable; it must not create a run without an `enqueued`
decision. Run status (`queued`, `running`, `succeeded`, `failed`, `cancelling`,
or `cancelled`) belongs to the existing Reports run lifecycle and is not copied
into the schedule decision.

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
- account credential storage, secret substitution, notification channels,
  webhooks, external queues, or report-center catalog behavior;
- changing browser actions, target validation, run cancellation, artifact
  contents, or run input redaction rules;
- catch-up execution after downtime or a timezone per schedule;
- importing schedules from a former repository or compatibility wrappers.

## Main and failure flows

1. The operator opens the schedule panel on the existing Reports templates or
   runs surface. The list loads with explicit state semantics.
2. Creating or editing validates that the target flow exists, cadence fields
   are complete, and input contains no recognized secret field. Save returns
   the schedule to the list with its next due time.
3. At a due occurrence, Reports resolves the local time and applies the 60-
   second window. It atomically records `enqueued` and creates one normal run,
   or records `skipped` with due/reason and no run. Repeated polls, restarts,
   DST gaps, and DST folds cannot create a duplicate occurrence.
4. A successful run links from an `enqueued` schedule decision to the existing run detail,
   steps, live frame, and artifacts. A failed run keeps its error evidence.
5. A missed occurrence is recorded as `skipped` with only its
   installation-timezone due instant and reason. It has no run reference and is
   not silently represented as `succeeded`.
6. A schedule query or save failure shows an explicit error and retry while
   preserving entered values. Background refresh keeps the last good rows.

## Business rules and permissions

- `reports:schedule:view` gates schedule list/detail; `reports:schedule:manage`
  gates create/update/enable/disable/delete. Backend checks remain authoritative.
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

## UI states and evidence

The UI contract is [Scheduled Report Automation UI](../../../ui/features/scheduled-report-automation.md).
It extends the existing Reports templates/runs surfaces and reuses their
tables, forms, dialogs, and run evidence.

| State | User-visible meaning | Required behavior |
| --- | --- | --- |
| Loading | Schedule list or form data is loading. | Keep the current surface stable; do not show false empty. |
| Populated | Schedules or run links are available. | Show cadence, timezone, next due, and last occurrence decision; show a run link only for `enqueued`. |
| Empty | Query succeeded with no schedules. | Explain how to create one when permitted. |
| Error | List/save/trigger read failed. | Show retry; preserve form values and last good rows. |
| Permission | Caller lacks view/manage capability. | Hide mutation actions and show the existing permission state. |
| Processing | Save, enable/disable, delete, or manual due-check is running. | Disable duplicate actions and retain the selected schedule. |
| Partial | A list contains mixed occurrence outcomes. | Keep each outcome and reason; never summarize as all successful. |

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
- Viewer cannot mutate a schedule, and direct backend mutation without the
  manage capability is rejected.
- Disable/enable and destructive delete have explicit processing and
  confirmation states; historical runs remain intact after schedule deletion.
- Background refresh and retry preserve filters, form values, and last good
  rows; initial failure is never shown as an empty list.
- Fixed copy is localized in Simplified Chinese and English; user-created
  names and runtime error text remain unchanged.
- The linked UI matrix passes desktop/narrow, light/dark, loading/empty/error/
  permission/processing/partial, keyboard focus, localization wrapping, and
  no-overflow checks.

## Verification matrix

| Layer | Evidence | Acceptance |
| --- | --- | --- |
| Source/static | schedule lifecycle, due identity, capability, and client mapping review | No cron parser, secret bypass, duplicate route catalog, or cross-service DB access. |
| Automated | Reports scheduler/service, persistence, input-safety, and contract tests | Daily/weekly, skip, idempotency, and failure evidence pass. |
| HTTP | Real delegated create/list/update/trigger requests | Owner/admin/viewer boundaries and run linkage are observable. |
| Browser | Required Reports templates/runs state matrix | Form validation, processing, retry, run links, partial outcomes, and responsive behavior pass. |
| Runtime/deployment | four-service startup and clock/timezone scenario | `Not verified` until the release environment is started. |

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

- Credential vaulting, notifications, webhooks, datasets, richer expression
  DSL, suspend/resume, and cross-module Report Center behavior.

## Ready for scheduled report automation implementation

The cadence, skip policy, run ownership, permission boundary, failure semantics,
non-goals, and acceptance are fixed. The linked UI contract is ready for
frontend implementation. Browser, HTTP, timezone, deployment, and external
delivery evidence remain `Not verified` until exercised.
