# Scheduled Report Automation UI

The Reports-owned module displays as **自动化 / Automation** in the sidebar,
page search and module-status surfaces. Routes remain `/reports/*`; this naming
change does not expand browser execution or rename existing child pages.

## Profile, authority, and selected source

- Profile: **Feature UI**.
- Product basis: [Scheduled Report Automation](../../product/features/scheduled-report-automation/spec.md).
- Shared visual authority: root `DESIGN.md`; this slice does not restate its
  theme, component, state, layout, or accessibility semantics.
- Selected source identity: accepted current Reports Templates and Runs
  surfaces in `apps/web/src/routes/reports/templates.tsx` and
  `apps/web/src/routes/reports/runs.tsx`, plus the repository-owned `DESIGN.md`
  adopted root `DESIGN.md` baseline.
- Selection status: accepted existing product surface and Ant Design system.
  Rights/use are repository-owned; former standalone report shells and visual
  alternatives are ignored.
- Target: schedule list/form integrated with existing Templates/Runs surfaces
  in loading, populated, empty, error, permission, processing, and partial
  states at 1920x1080, 1440x900, and 390x844 CSS px, 100% zoom, light/dark,
  zh-CN/en-US. The route-local schedule panel and API client are implemented;
  SR-UI-002 has its two named rendered runtime captures. The processing, partial,
  runtime-failure, combined view-only, and retained-evidence matrix is verified
  only when the atomic `verify-reports-ui-state-linux` current manifest matches
  this checkout; otherwise it remains **Not verified**.

The Linux runtime browser gate additionally exercises rendered schedule create
and edit through a container-only route-exact proxy. Both a disconnected request
and an HTTP rejection retain the Modal; the form alert and retained due-time
value are asserted through route-local test identifiers. Each case has its own
Reports run and PNG evidence entry in the schema-2 browser manifest.

The selected source proves current PageCard, table, form, Modal, run-detail,
and DataState ownership. It does not authorize a new workflow builder, a new
page shell, or exact schedule-specific geometry; the schedule panel remains a
route-local adaptation.

Only the notifications-capable `full` selection composes the shared
notification-delivery card on Runs. The `reports` preset (`[access, reports]`)
does not select notifications and has no card. In `full`, the card presents
pending/quarantine counts and charged bytes, the five-counter irreversible-gap
total, nullable formatted timestamps, and explicit loading, permission, error,
healthy, and gap states. Fixed copy uses the existing Chinese/English
localization owner.

Screenshot artifacts remain bounded backend evidence: failed oversize capture
is shown through the existing run failure evidence and does not create a fake
artifact or partial download surface.

## Current implementation and verification status

`apps/web/src/routes/reports/templates.tsx` renders the schedule panel behind
`reports:schedule:view`; its create, edit, enable/disable, and delete controls
remain behind `reports:schedule:manage`. The route reads the stable
`checkPermissions` selector before deriving schedule state, so a schedule-only
viewer can reach the panel without receiving flow-management actions. The
panel renders daily/weekly values, installation timezone, next due, and the
latest occurrence; only an occurrence with `runId` exposes the existing run
link.

The route passes its schedule-manage result to the panel's column composition.
Only a manager assembles the fixed-width actions column. A view-only panel has
no actions column at all, so its 390px table keeps the full due-time and
timezone value such as `10:16 · UTC` visible. The browser flow asserts that
exact value, the absent actions, and no horizontal overflow.

The static seam tests verify selector/gate composition and the route-local retry
behavior: failed/cancelled visibility, one per-source pending key shared by the
list and detail, child selection after success, and source preservation after
failure. The schedule-save seam verifies that an API failure displays an error
without closing the dialog, while successful refresh remains the only path that
announces and closes it. The Reports
browser verifier separately verifies schedule-only viewer/manager delegated
HTTP access while exercising the existing target-backed browser runner. It
does not produce a visual capture of the Web Templates route. Desktop/narrow,
light/dark, keyboard, localization wrapping, and the complete rendered state
matrix are therefore **Not verified**.

The state-closure gate uses the same Reports components and four-service
runtime. Its manager flow is intended to hold the executor with a bounded series
of individually supported 30-second pauses, capture the active-run processing
state, then close the audit and use the rendered run-cancel control. A matching
current manifest verifies the cancelled pause-step receipt. It shows
a real terminal failure with the failed step, error, and existing Retry control.
It compares the source run, steps, and artifacts before and after the retry so
the selected child cannot replace retained evidence. Its controlled disposable
SQLite fixture renders one `enqueued` row with a real run link and one
`skipped` row with a separate due/reason and no link, proving the partial
summary without claiming scheduler history. A single mobile view-only role
holds both schedule and run read capabilities; it sees neither schedule
`schedule-create`, `schedule-edit`, `schedule-toggle`, or `schedule-delete`
control, nor the run `run-create`, `run-cancel-*`, or Retry control. These
stable control identifiers keep the combined permission assertion tied to the
same rendered viewer flow, and its direct mutation requests are denied.

## Surface and layout contract

- Keep one `PageCard`/page heading per existing Reports surface. The schedule
  list is a bounded panel on Templates or Runs, not a fifth top-level module.
- The schedule list owns cadence, enabled state, next due, last decision, and
  flow identity columns. It uses the current table shell and keeps action
  controls in a compact fixed-width column.
- Create/edit uses the existing Ant Design `Modal` and `Form` rhythm. Fields
  appear in task order: flow, cadence, weekday when weekly, local due time,
  safe input, description, enabled state. The installation timezone is visible
  helper text, not a second timezone selector.
- A schedule row links to the existing Reports run detail. Run evidence remains
  in the current Drawer/Modal flow; schedule UI does not clone artifact or
  browser-step viewers.
- At narrow widths, filters and form fields stack in source order, table
  columns reduce to identity/status/next due/action essentials, and long error
  reasons wrap inside their cell or detail surface.

## Component and data-owner mapping

| Responsibility                                | Current owner                                            | Decision                                                                                                                                            |
| --------------------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Page shell and section heading                | `PageCard`/existing route heading                        | Reuse                                                                                                                                               |
| Schedule list and paging                      | `DataTableShell` + route-local `ProTable`                | Reuse; schedule columns are route-local; the manager-only actions column is assembled separately from view-only columns                             |
| Loading, empty, error, permission, processing | `DataState`                                              | Reuse                                                                                                                                               |
| Schedule form                                 | Existing Ant Design `Form`, `Select`, time/date controls | Reuse; route-local validation                                                                                                                       |
| Enable/disable and outcome labels             | Ant Design `Tag`/`Switch` with semantic theme            | Wrap route-local meaning; no new status component                                                                                                   |
| Destructive confirmation                      | Existing `ConfirmDialog`                                 | Reuse                                                                                                                                               |
| Run link/detail                               | Existing Reports route detail Modal/Drawer               | Wrap/reuse; no new artifact viewer                                                                                                                  |
| Terminal-run retry                            | Existing Reports run action buttons and detail Modal     | Wrap/reuse; no new run form or route                                                                                                                |
| HTTP transport                                | Existing Reports API client and `apiRequest`             | Reuse; Reports chain is Rust `ModuleRouter/Manifest` -> handwritten `apps/web/src/api/reports/contract.ts` -> `scripts/verify-worker-contracts.mjs` |
| Schedule/run data                             | Reports automation and Reports SQLite                    | Reports owns lifecycle and persistence                                                                                                              |

There is no new shared scheduler table, form DSL, status enum, or workflow
component. The route-local composition may be promoted only after a second
compatible consumer exists.

## State and interaction contract

| State      | Presentation                                                | Interaction                                                                                                                                                                                                     |
| ---------- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Loading    | Compact `DataState` in the schedule panel                   | Create/filter actions wait for required data; no false empty.                                                                                                                                                   |
| Populated  | Table with cadence, timezone, next due, and last decision   | Open detail/edit only when capability permits; only `enqueued` decisions link to a run.                                                                                                                         |
| Empty      | `DataState` explains no schedules and allowed create action | Create opens the existing Modal form.                                                                                                                                                                           |
| Error      | Alert-semantic `DataState` with retry                       | Retry preserves filters and any open form values.                                                                                                                                                               |
| Permission | Permission-semantic state or hidden mutation actions        | Viewer can read only when schedule-view is granted.                                                                                                                                                             |
| Processing | Disabled primary/action controls and inline progress        | Duplicate save/toggle/delete cannot start.                                                                                                                                                                      |
| Partial    | Per-occurrence decision/outcome tags and explicit summary   | Skipped items show their due instant and reason separately with no run link; a DST gap without `dueAt` shows `dueLocal` plus the installation timezone; enqueued items link to their run; no all-success label. |
| Validation | Field-local guidance plus form-level error                  | Secret-looking input and cadence errors prevent submission.                                                                                                                                                     |

When a create or edit request fails after local validation, the open Modal
renders a form-level error alert using the server error when available and a
localized fallback otherwise. The alert remains until the next save attempt or
the dialog closes. The Modal does not close or reinitialize fields on failure,
so the operator can correct the retained draft or retry immediately. This
schedule mutation suppresses its global error toast, leaving the form alert as
the only failure presentation. A dialog open cycle initializes its draft once;
flow-option or schedule-prop refreshes never overwrite an active draft, and a
late callback from a closed cycle cannot alter a later reopened dialog.

The form never exposes credential fields, arbitrary cron text, notification
channels, or catch-up controls. A disabled schedule is textually distinct from
a failed run and a skipped occurrence.

The Runs list and Run audit detail show a bilingual Retry action only for
`failed` and `cancelled` terminal runs and only behind `reports:run:manage`.
While the request is pending, both the list and Run audit Retry controls for
that source run are disabled. A successful retry selects the returned direct
child and refreshes the list; a failure is visible through the existing message
feedback without closing the source run. Repeating the action returns the same
direct child even after it is terminal. To continue after a failed or cancelled
child, the operator opens that child and retries it, extending the
source-to-child chain without replacing an earlier link. Retention can clear a
child's source reference when the older source is removed, so the UI does not
promise lineage after source deletion. Retry creates a separate
manual run from the source's persisted snapshot. It does not change the source
evidence or expose a second link from any scheduled occurrence, so it is not
presented as a schedule catch-up action.

### Runs retry Linux Chromium acceptance

The Linux Chromium acceptance closes the rendered terminal-run retry path. It
seeds a failed source run through the real Reports service, then proves
that the Runs-list Retry action creates or returns its direct child and selects
that child's audit. It separately opens the failed source audit and proves its
Retry action selects the same child by its returned ID. Source run, step, and
artifact responses are compared before and after both UI actions, so retry
cannot replace source evidence. The verifier does not assert that the child
remains `queued`: it identifies the direct child by ID and selected audit state.

The same acceptance proves that succeeded and nonterminal runs have no Retry
control, and that a real `reports:run:view`-only user sees no Retry control and
receives a backend rejection for the retry endpoint. It captures a 1440x900
dark/en-US managed view and a 390x844 light/zh-CN view-only surface, asserting
key copy and no horizontal overflow. It retains the six schedule lifecycle
success cases and ten fault-mutation cases. The target-backed Chromium gate now
verifies this browser retry acceptance.

## Accessibility and responsive behavior

- The schedule panel has one descriptive heading; table headers and action labels
  remain associated. Status values include text and are not color-only.
- Form labels, errors, helper text, and the installation timezone are announced
  through existing Ant Design form semantics. Save/cancel actions keep their
  current order and visible focus.
- Modal focus is trapped and restored. Retry and toggle actions are keyboard
  reachable, and processing disables duplicate operations without removing
  context.
- Long flow names, descriptions, and failure reasons wrap without clipping;
  the modal owns its vertical scroll and no page-level horizontal scroll is
  introduced.
- Existing reduced-motion and theme behavior remains authoritative. No new
  animation, glow, gradient, or decorative progress treatment is added.

## API and data ownership

The frontend consumes `apps/web/src/api/reports/contract.ts` after Reports route
registration and schedule types are extended. The Reports chain is
`Rust ModuleRouter/Manifest -> handwritten contract.ts ->
scripts/verify-worker-contracts.mjs`; this module does not use Admin OpenAPI or
Orval. Reports owns schedule state, due occurrence identity, decision/run
linkage, and persistence. Admin owns delegation and capability reconciliation.

## Traceable UI deltas

| ID        | Selected source                                                                                    | Current runtime                                                                                                                                                                                                                                                    | Target contract                                                                                                                                                                                                           | Priority | Owner and validation                                                                                                                                                   |
| --------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SR-UI-001 | `source-extracted`: current Reports PageCard/table composition                                     | Verified in Linux Chromium at 1440x900 and 390x844                                                                                                                                                                                                                 | Schedule list sits inside existing Templates/Runs shell with no new module page                                                                                                                                           | P1       | Reports route; target-backed browser gate                                                                                                                              |
| SR-UI-002 | `source-extracted`: existing Reports Form/Modal patterns                                           | Verified by the dedicated Colima Linux/arm64 Chromium gate: manager 1440x900 dark/en-US and schedule-view-only 390x844 light/zh-CN                                                                                                                                 | Daily/weekly fields, local validation, secret rejection, timezone copy, and focus restoration                                                                                                                             | P1       | dedicated `verify-schedule-form-linux` gate; atomic `current` manifest and route-local form/browser matrix                                                             |
| SR-UI-003 | `source-extracted`: current DataState and run outcome tags                                         | Verified target-backed `enqueued` rendering; controlled isolated SQLite fixture covers `missed`                                                                                                                                                                    | Loading, empty, error, permission, processing, and partial remain distinct; skipped shows distinct due/reason fields; a null `dueAt` falls back to `dueLocal` plus installation timezone; enqueued alone links a run      | P1       | real scheduler poll plus isolated fixture; forced response matrix remains outside browser scope                                                                        |
| SR-UI-004 | `source-extracted`: existing run detail linkage                                                    | Verified target-backed rendered schedule-to-run link                                                                                                                                                                                                               | Each `enqueued` occurrence links to existing run evidence; `skipped` has distinct due/reason fields, with `dueLocal` plus installation timezone when `dueAt` is null, and no run link; no cloned viewer                   | P1       | Templates link opens exact Runs audit; isolated fixture proves no-link state                                                                                           |
| SR-UI-005 | `source-extracted`: existing Runs action and Run audit controls                                    | Verified in source, deterministic behavior tests, worker HTTP contracts, and the target-backed Linux Chromium gate: failed/cancelled visibility, shared per-source pending, exact child selection, permission denial, and source-evidence preservation are covered | Failed/cancelled runs offer bilingual managed retry; shared per-source pending prevents duplicate list/detail actions, repeated requests select the same direct child, and a terminal child can start the next chain link | P1       | route-local behavior test, Reports worker HTTP retry-chain contract, and target-backed Chromium retry gate                                                             |
| SR-UI-006 | `source-extracted`: existing Reports DataState, schedule outcome, run audit, and Retry composition | Verified only by a matching current `verify-reports-ui-state-linux` manifest                                                                                                                                                                                       | Manager processing, partial decision rows, runtime failure, retry source preservation, and a combined schedule/run view-only role remain distinct on their rendered surfaces                                              | P1       | Linux Chromium state-closure gate with atomic manifest, four run-step receipts, controlled partial fixture, manager/mobile screenshots, and source before/after hashes |

`SR-UI-006` is verified locally only when its named gate's current manifest
matches this checkout. Native systemd, confined native-host seccomp, deployed
scheduling, and the remaining visual matrices outside this bounded state-closure
evidence remain **Not verified**.

## Responsive and verification matrix

| Priority | Viewport         | Theme/locale  | Surface and state                | Acceptance                                                                                                                                                               |
| -------- | ---------------- | ------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Required | 1920x1080 @ 100% | light / zh-CN | Templates schedule populated     | Columns, actions, next due, and timezone helper align with existing PageCard; no overflow.                                                                               |
| Required | 1440x900 @ 100%  | dark / en-US  | Target-backed schedule lifecycle | Daily create, weekly edit, disable/enable and delete; key copy and no horizontal overflow verified.                                                                      |
| Required | 390x844 @ 100%   | light / zh-CN | Schedule-view-only list          | No actions column is assembled; `10:16 · UTC` is fully visible, mutation controls are absent, and there is no horizontal overflow.                                       |
| Required | 1440x900 @ 100%  | dark / en-US  | Managed Runs retry               | List retry selects the exact direct-child audit; key copy and no horizontal overflow verified.                                                                           |
| Required | 390x844 @ 100%   | light / zh-CN | Runs-view-only list              | Retry is hidden, no unauthorized flow lookup or permission-error toast is rendered, and the backend rejects the same user; key copy and no horizontal overflow verified. |
| Required | 390x844 @ 100%   | dark / en-US  | Form validation/partial outcomes | Fields stack, errors wrap, and outcome tags remain text-readable.                                                                                                        |
| Required | 1440x900 @ 100%  | dark / en-US  | SR-UI-002 manager form           | Timezone copy, daily create, weekly edit/weekday, local no-request failures, secret-policy rejection, cadence row, exact focus restoration, and no horizontal overflow.  |
| Required | 390x844 @ 100%   | light / zh-CN | SR-UI-002 schedule-view-only     | Panel/timezone copy visible; create and form unavailable; no horizontal overflow.                                                                                        |

Static checks cover route/API owner, capability visibility, and prohibited
credential/cron fields. The dedicated SR-UI-002 Linux Chromium gate passed with
an atomic `current` manifest for `linux/arm64`: it records API count deltas and
proxy mutation receipts, distinguishes local no-request validation from the
intentional server-side secret rejection, and directly asserts the active
trigger after cancel/save. It captures the manager 1440x900 dark/en-US and
schedule-view-only 390x844 light/zh-CN surfaces without horizontal overflow.
Native systemd/seccomp remain outside this visual acceptance.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse current Reports shells, forms, tables,
feedback, confirmation, run detail, and theme semantics.

## Ready for dev-frontend scheduled report automation

The selected source, layout ownership, component mapping, states, permission
visibility, responsive/accessibility rules, and acceptance IDs are implemented
in the current Reports/Web slice. Scheduler HTTP behavior, the defined Templates
success lifecycle, and Runs retry are verified; other response matrices, native
systemd, and native-host seccomp remain **Not verified**.

## Linux Chromium success-path acceptance

Verified by the Linux Chromium gate: the real Reports service and Web route, never the fault proxy, create a daily schedule, edit it to weekly, disable and re-enable it, then delete it. The gate also creates a next-minute daily schedule against a healthy target-backed flow, bounded-waits for the real scheduler to persist `enqueued` plus its run ID, and follows the rendered Templates link into that exact Runs audit. A controlled SQLite fixture in the disposable verifier renders one `missed` decision with separate due/reason fields and proves it has no run link; it is not claimed as an API-created historical occurrence. A schedule-view-only session sees the panel but no create, edit, toggle, or delete action. Evidence captures `1440x900` dark/en-US and `390x844` light/zh-CN, with exact screenshot dimensions, bound SHA-256 values, key copy, and no horizontal overflow.

## Notification delivery health

In `full`, the existing Runs page places the same compact delivery card above its table.
It uses the Reports run-view permission, shows only aggregate delivery state,
and exposes loading, forbidden and recoverable-error Retry states. No menu,
page or notification payload is added.
