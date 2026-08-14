# Monitoring Incident Diagnostics UI

## Profile, authority, and selected source

- Profile: **Feature UI**.
- Product basis: [Monitoring Incident Diagnostics](../../product/features/monitor-incident-diagnostics/spec.md).
- Shared visual authority: root `DESIGN.md`; this slice does not restate its
  theme, component, state, layout, or accessibility semantics.
- Selected source identity: accepted current Ant Design Monitoring overview
  and Nodes surfaces in `apps/web/src/routes/monitoring/overview.tsx` and
  `apps/web/src/routes/monitoring/nodes.tsx`, together with the repository
  adopted root `DESIGN.md` baseline.
- Selection status: accepted existing product surface and repository-owned
  design system. Approval is the repository's accepted Ant Design migration;
  no external image or brand asset is copied.
- Rights/use: repository-owned source and existing Ant Design assets may be
  reused in this repository. Legacy Radix/shadcn and glass/gradient captures
  are ignored.
- Target: populated, loading, empty, error, permission, and partial incident
  states at 1920x1080, 1440x900, and 390x844 CSS px, 100% zoom, light/dark,
  zh-CN/en-US. Current runtime captures for the new incident region are
  `Not verified` until implementation.

The accepted source proves shell hierarchy, density, status treatment, Drawer
behavior, and existing metric/table composition. It does not prove the exact
geometry of the new incident region, so those values inherit current owners or
remain route-local `proposed` adaptations rather than new shared tokens.

## Surface and layout contract

- Keep one `PageHeader` on the Monitoring overview. Add the incident summary or
  list inside the existing page composition; do not create a second dashboard
  or a new global shell.
- Keep the existing Monitoring metric and node/check panels as their current
  owners. The incident region owns its filters, rows, status tags, and detail
  trigger.
- The incident detail uses the existing Ant Design `Drawer` pattern already
  used by the Nodes surface. The Drawer owns its own vertical overflow and
  focus restoration; the page owns no nested horizontal scroll.
- Source, status, title, and observed time remain visible in a populated row.
  Structured evidence wraps or scrolls inside the detail region without
  clipping the close action.
- At narrow widths, filters stack in source order, table columns reduce to
  stable essentials, and detail content wraps before any bounded code/data
  region scrolls horizontally.

## Component and data-owner mapping

| Responsibility | Current owner | Decision |
| --- | --- | --- |
| Page title and actions | `PageHeader` | Reuse |
| Existing operational metrics | `MetricCard` | Reuse |
| Incident list shell and paging | `DataTableShell` + route-local `ProTable` | Reuse; keep columns local |
| Loading, empty, error, permission, processing | `DataState` | Reuse |
| Status and source labels | Ant Design `Tag`/text with semantic theme | Wrap route-local meaning; no new status component |
| Incident detail | Existing Ant Design `Drawer` pattern in Nodes | Reuse/wrap route-local content |
| HTTP transport | Existing module API client and `apiRequest` | Reuse; Monitor chain is Rust `ModuleRouter/Manifest` -> handwritten `apps/web/src/api/monitor/contract.ts` -> `scripts/verify-worker-contracts.mjs` |
| Incident data | Monitor incidents/Monitor SQLite | Monitor owns data; Web does not duplicate the model |

No new shared table, chart, status, or evidence viewer is justified by this
single consumer. A future second diagnostic surface must prove compatible
semantics before extending a shared owner.

## State and interaction contract

| State | Presentation | Interaction |
| --- | --- | --- |
| Loading | Compact `DataState` in the incident region | Filters and detail actions are disabled; no false empty table. |
| Populated | Route-local table with status, source, title, and time | Keyboard-accessible row/detail action opens Drawer; historical `acknowledged` is read-only. |
| Empty | `DataState` with active-filter explanation | Reset filters remains available when permitted. |
| Error | Alert-semantic `DataState` with retry | Retry calls only the owning query and preserves filters. |
| Permission | Permission-semantic `DataState` or existing route guard | No query retry loop or mutation affordance. |
| Partial | Row-level warning plus summary `DataState` copy | Unavailable detail is explicit; readable rows remain usable. |
| Detail loading/error | Drawer-local `DataState` | List remains mounted; retry keeps selected incident. |

The UI never exposes acknowledge, resolve, suppress, delete, or policy-edit
controls in this slice. Status color is paired with text and does not carry the
meaning alone.

## Accessibility and responsive behavior

- The incident region has a descriptive heading and the table has stable
  column/header associations. Status text is present in the accessibility tree.
- Detail triggers are real keyboard-focusable buttons with visible focus; the
  Drawer traps focus and restores it to the trigger on close.
- Loading uses status semantics, errors use alert semantics, and permission
  feedback identifies the missing boundary without exposing incident data.
- Long titles/details wrap or use an explicitly labeled bounded scroll region;
  no text is hidden solely by color or clipped behind the close action.
- Existing reduced-motion and theme behavior remains authoritative; no new
  animation, gradient, glow, or hover lift is introduced.
- Required keyboard, zoom/reflow, localization, and touch-target checks are
  included in the matrix below.

## API and data ownership

The frontend consumes the Monitor route contract through the existing
`apps/web/src/api/monitor/contract.ts` owner after the Rust `ModuleRouter` and
Manifest are extended. The verification chain is
`Rust ModuleRouter/Manifest -> handwritten contract.ts ->
scripts/verify-worker-contracts.mjs`; this module does not use Admin OpenAPI or
Orval. This UI document does not create a second path catalog or redefine DTOs.
Monitor owns incident status, filtering semantics, pagination, and structured
evidence. Admin owns delegation and permission enforcement.

## Traceable UI deltas

| ID | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- |
| MI-UI-001 | `source-extracted`: current Monitoring `PageHeader` and page-width contract | `Not verified`: new incident region is not yet rendered | One incident region inside the existing overview composition; no duplicate page title | P1 | Monitoring route; same-state DOM/layout check at desktop and narrow widths |
| MI-UI-002 | `source-extracted`: Nodes Drawer pattern | `Not verified`: incident detail Drawer not yet rendered | Detail keeps list context, focus restoration, bounded content, and explicit error/retry | P1 | route-local Drawer content; keyboard/focus and overflow check |
| MI-UI-003 | `source-extracted`: `DataState` loading/empty/error/permission semantics | `Not verified`: incident state matrix not yet exercised | Loading, empty, error, permission, and partial remain distinct | P1 | `DataState` owner and query state; forced response matrix |
| MI-UI-004 | `source-extracted`: current table and Tag semantics | `Not verified`: new columns/status rendering | Status/source/title/time are text-first and localized; no color-only meaning | P1 | route-local table; accessibility tree and contrast check |

Evidence levels are intentionally conservative: exact new geometry and runtime
behavior are not claimed before frontend implementation and browser capture.

## Responsive and verification matrix

| Priority | Viewport | Theme/locale | Surface and state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 @ 100% | light / zh-CN | Monitoring overview populated | Incident region aligns with existing content inset; table actions are reachable; no horizontal overflow. |
| Required | 1440x900 @ 100% | dark / en-US | Overview loading/error/permission | Feedback remains legible; status contrast and focus pass. |
| Required | 390x844 @ 100% | light / zh-CN | Overview empty/partial | Filters stack, essential columns remain readable, detail trigger is reachable. |
| Required | 390x844 @ 100% | dark / en-US | Nodes detail Drawer populated/error | Drawer content wraps or bounded-scrolls; focus returns on close. |

Static validation covers route/client ownership, state derivation, and absence
of mutation controls. Browser validation must capture the selected accepted
surface and implementation at the same viewport/state for two passes; no such
runtime evidence exists in this documentation-only change.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse current `DESIGN.md`, `PageHeader`,
`MetricCard`, `DataState`, table shell, Drawer, and semantic theme tokens.

## Ready for dev-frontend monitor incident diagnostics

The selected source, ownership, state contract, component mapping, responsive
rules, accessibility requirements, and acceptance IDs are fixed. The slice is
**Ready for dev-frontend**. Browser/runtime evidence, final composed geometry,
HTTP permission behavior, and the two comparison passes remain `Not verified`
until implementation.
