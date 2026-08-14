# Scheduled Report Automation UI

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
  zh-CN/en-US. New schedule runtime captures are `Not verified` until
  implementation.

The selected source proves current PageCard, table, form, Modal, run-detail,
and DataState ownership. It does not authorize a new workflow builder, a new
page shell, or exact schedule-specific geometry; the schedule panel remains a
route-local adaptation.

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

| Responsibility | Current owner | Decision |
| --- | --- | --- |
| Page shell and section heading | `PageCard`/existing route heading | Reuse |
| Schedule list and paging | `DataTableShell` + route-local `ProTable` | Reuse; columns remain route-local |
| Loading, empty, error, permission, processing | `DataState` | Reuse |
| Schedule form | Existing Ant Design `Form`, `Select`, time/date controls | Reuse; route-local validation |
| Enable/disable and outcome labels | Ant Design `Tag`/`Switch` with semantic theme | Wrap route-local meaning; no new status component |
| Destructive confirmation | Existing `ConfirmDialog` | Reuse |
| Run link/detail | Existing Reports route detail Modal/Drawer | Wrap/reuse; no new artifact viewer |
| HTTP transport | Existing Reports API client and `apiRequest` | Reuse; Reports chain is Rust `ModuleRouter/Manifest` -> handwritten `apps/web/src/api/reports/contract.ts` -> `scripts/verify-worker-contracts.mjs` |
| Schedule/run data | Reports automation and Reports SQLite | Reports owns lifecycle and persistence |

There is no new shared scheduler table, form DSL, status enum, or workflow
component. The route-local composition may be promoted only after a second
compatible consumer exists.

## State and interaction contract

| State | Presentation | Interaction |
| --- | --- | --- |
| Loading | Compact `DataState` in the schedule panel | Create/filter actions wait for required data; no false empty. |
| Populated | Table with cadence, timezone, next due, and last decision | Open detail/edit only when capability permits; only `enqueued` decisions link to a run. |
| Empty | `DataState` explains no schedules and allowed create action | Create opens the existing Modal form. |
| Error | Alert-semantic `DataState` with retry | Retry preserves filters and any open form values. |
| Permission | Permission-semantic state or hidden mutation actions | Viewer can read only when schedule-view is granted. |
| Processing | Disabled primary/action controls and inline progress | Duplicate save/toggle/delete cannot start. |
| Partial | Per-occurrence decision/outcome tags and explicit summary | Skipped items show due/reason with no run link; enqueued items link to their run; no all-success label. |
| Validation | Field-local guidance plus form-level error | Secret-looking input and cadence errors prevent submission. |

The form never exposes credential fields, arbitrary cron text, notification
channels, or catch-up controls. A disabled schedule is textually distinct from
a failed run and a skipped occurrence.

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

| ID | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- |
| SR-UI-001 | `source-extracted`: current Reports PageCard/table composition | `Not verified`: schedule panel not yet rendered | Schedule list sits inside existing Templates/Runs shell with no new module page | P1 | Reports route; desktop/narrow composition check |
| SR-UI-002 | `source-extracted`: existing Reports Form/Modal patterns | `Not verified`: cadence validation and timezone copy not yet exercised | Daily/weekly fields, local validation, secret rejection, and focus restoration | P1 | route-local form; deterministic form/browser matrix |
| SR-UI-003 | `source-extracted`: current DataState and run outcome tags | `Not verified`: processing/partial/skipped states not yet rendered | Loading, empty, error, permission, processing, and partial remain distinct; skipped shows due/reason only, enqueued alone links a run | P1 | query/mutation state owner; forced response matrix |
| SR-UI-004 | `source-extracted`: existing run detail linkage | `Not verified`: schedule-to-run evidence not yet rendered | Each `enqueued` occurrence links to existing run evidence; `skipped` has due/reason only and no run link; no cloned viewer | P1 | Reports route/API owner; interaction and permission check |

Exact new geometry, computed styles, and runtime schedule outcomes are
`Not verified` until implementation and browser capture.

## Responsive and verification matrix

| Priority | Viewport | Theme/locale | Surface and state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 @ 100% | light / zh-CN | Templates schedule populated | Columns, actions, next due, and timezone helper align with existing PageCard; no overflow. |
| Required | 1440x900 @ 100% | dark / en-US | Schedule form processing/error | Modal focus, semantic contrast, and error wrapping pass. |
| Required | 390x844 @ 100% | light / zh-CN | Empty/permission schedule list | Create guidance and permission state remain readable; no hidden action. |
| Required | 390x844 @ 100% | dark / en-US | Form validation/partial outcomes | Fields stack, errors wrap, and outcome tags remain text-readable. |

Static checks cover route/API owner, capability visibility, and prohibited
credential/cron fields. Browser validation requires two same-viewport/state
comparison passes after implementation; no runtime evidence is claimed here.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse current Reports shells, forms, tables,
feedback, confirmation, run detail, and theme semantics.

## Ready for dev-frontend scheduled report automation

The selected source, layout ownership, component mapping, states, permission
visibility, responsive/accessibility rules, and acceptance IDs are fixed. The
slice is **Ready for dev-frontend**. Scheduler HTTP behavior, timezone/runtime
evidence, final geometry, and two-pass browser review remain `Not verified`.
