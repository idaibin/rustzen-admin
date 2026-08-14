# Analytics Collection Safety UI

## Profile, authority, and selected source

- Profile: **Feature UI**.
- Product basis: [Analytics Collection Safety](../../product/features/analytics-collection-safety/spec.md).
- Shared visual authority: root `DESIGN.md`; this slice does not restate its
  theme, component, state, layout, or accessibility semantics.
- Selected source identity: accepted current Analytics overview and Details
  surfaces in `apps/web/src/routes/analytics/overview.tsx` and
  `apps/web/src/routes/analytics/details.tsx`, plus the repository-owned
  adopted root `DESIGN.md` baseline.
- Selection status: accepted existing product surface. Rights/use are
  repository-owned; legacy analytics shell and unrelated visual references are
  ignored.
- Target: existing overview/details in populated, loading, empty, error,
  permission, and partial states at 1920x1080, 1440x900, and 390x844 CSS px,
  100% zoom, light/dark, zh-CN/en-US. Public tracker execution has no Admin
  page and is verified separately through browser/network evidence.

The selected source proves current page hierarchy, MetricCard anatomy, table
composition, and DataState ownership. The implementation adds only a
route-local status explanation backed by the verified Insights read contract;
it does not create a second settings system.

## Surface and layout contract

- Keep one `PageHeader` on Analytics overview and Details. Existing metric
  cards remain the factual summary and are not replaced by a collection-status
  dashboard.
- If collection status is available to the permitted operator, render one
  compact semantic status row below the page header or beside existing query
  feedback. It must state enabled/disabled/unavailable using text and status
  semantics; it must not expose a project selector or raw key. The public
  project routing identifier is not a credential and must never be presented as
  one.
- Details keeps the existing route-local table and filters. Paths display
  pathname values only; long paths wrap or use a labeled bounded region rather
  than creating page-level horizontal scrolling.
- Public tracker opt-in is outside the Admin shell. The host page must perform
  the explicit enable call with `consent === true`; a script asset may be
  loaded first, but it remains inert until then. Before enable there is no
  request patch, request, queue, or local visitor/session identifier. Opt-out
  stops sends and removes those local IDs. No consent proof is displayed or
  claimed here.

## Component and data-owner mapping

| Responsibility | Current owner | Decision |
| --- | --- | --- |
| Page title and actions | `PageHeader` | Reuse |
| Overview metrics | `MetricCard` | Reuse; values remain Insights-owned |
| Query feedback | `DataState` | Reuse |
| Collection status explanation | Ant Design `Alert`/`Tag` with semantic theme | Wrap route-local meaning; no shared status component |
| Event details table | Existing route-local table and `DataTableShell` where present | Reuse; keep event columns local |
| Filters and date range | Existing route-local Ant Design controls | Reuse; no global analytics filter store |
| HTTP transport | Existing Insights API client and `apiRequest` | Reuse; Insights chain is Rust `ModuleRouter/Manifest` -> handwritten `apps/web/src/api/insights/contract.ts` -> `scripts/verify-worker-contracts.mjs`; no Admin OpenAPI/Orval |
| Event data and collection settings | Insights tracking/query/settings and Insights SQLite | Insights owns data; Web does not duplicate event storage |
| Visitor-consent bootstrap | Host application shell or deployment integration | Host owns the explicit opt-in entry and tracker injection order; Admin/Insights do not claim consent verification |

No new tracker UI, chart library, event-property viewer, or shared analytics
component is justified by this slice. Any collection-policy management remains
the `insights:manage` owner boundary and requires a separate settings contract.

## State and interaction contract

| State | Presentation | Interaction |
| --- | --- | --- |
| Loading | Existing `DataState` or table loading | Keep filters and shell; do not show zero metrics. |
| Populated | Current metric/table hierarchy plus optional status row | Paths and event names remain text-readable; no raw query values. |
| Empty | `DataState` explains range and collection status when known | Reset range remains available where current route supports it. |
| Error | Alert-semantic `DataState` with retry | Retry calls the owning query and keeps last successful data. |
| Permission | Permission-semantic state or route guard | No status/config details leak to unauthorized users. |
| Partial | Status row or table summary identifies missing categories | Readable metrics/events remain visible; no all-good summary. |
| Tracker rejected | Not an Admin page state | Public endpoint returns explicit 413/429/507 or validation rejection with zero persistence; Analytics read data does not reset to empty. |

Color never carries consent, permission, or data-quality meaning alone. The
status row is informational in this slice; it does not expose a mutation
control unless a future settings spec explicitly authorizes one.

## Accessibility and responsive behavior

- Overview and Details preserve one heading hierarchy and meaningful table
  headers. Collection status is announced as text with status/alert semantics,
  not as an icon-only indicator.
- Retry controls are keyboard reachable with existing visible focus. No
  non-interactive MetricCard gains a tab stop.
- Date/path filters wrap in source order at narrow widths. Pathnames and event
  names wrap or use an explicitly labeled bounded region; query strings are
  not displayed because they are not collected.
- Light/dark semantic colors meet the existing contrast contract. Reduced
  motion remains the application default; no new animation is introduced.
- Locale changes may expand labels; the parent layout owns reflow and no page
  introduces horizontal overflow.

## API and data ownership

The frontend consumes `apps/web/src/api/insights/contract.ts` after the Rust
`ModuleRouter`/Manifest and tracking validator are updated. The contract chain
is `Rust ModuleRouter/Manifest -> handwritten
apps/web/src/api/insights/contract.ts -> scripts/verify-worker-contracts.mjs`;
this module does not use Admin OpenAPI or Orval. The UI does not define
project-key, origin, payload, rate, or storage schemas.
Insights owns installation policy (`collection_enabled`, `project`, normalized
`origin`), event identity, storage, retention, aggregation, and collection
status. The host bootstrap owns visitor opt-in and tracker injection; Admin owns
delegation and read/manage capability enforcement.

The 30-request/300-event per-project/source limits are process-local Insights
runtime admission guards. They reset when the Insights process restarts and are
not durable cross-restart quotas or hard maxima. Body, batch, storage-budget,
and free-disk limits remain hard fail-closed boundaries. The Analytics UI does
not present the process-local rate window as a persistent quota or make any of
these values configurable.

The public tracker transport is a separate Insights contract: a browser
preflight to `OPTIONS /api/insights/track` is allowed only for an enabled,
configured project with the normalized origin in policy. It returns exact
non-wildcard `Access-Control-Allow-Origin`, `POST`, and
`content-type, x-rustzen-project-key` headers with `Vary: Origin`. A `POST`
echoes the origin only after project-key/origin validation, including later
business errors; denied origins receive no allow headers. The Analytics UI does
not infer consent or expose the raw project key.

## Traceable UI deltas

| ID | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- |
| AC-UI-001 | `source-extracted`: current Analytics `PageHeader`/MetricCard hierarchy | `source-extracted`: route-local collection status card is rendered for `insights:manage` | Preserve overview hierarchy; add only a compact route-local status explanation when available | P1 | Analytics routes; same-state composition and contrast check |
| AC-UI-002 | `source-extracted`: existing `DataState` semantics | `source-extracted`: policy query uses loading, error, and permission `DataState` states; partial runtime remains `Not verified` | Loading, empty, error, permission, and partial remain distinct; no failed-to-empty conversion | P1 | query state owner; forced response/browser matrix |
| AC-UI-003 | `source-extracted`: current event detail table | `Not verified`: pathname-only display not yet exercised | Display pathname/event fields without query strings, page text, or arbitrary properties | P1 | Insights client/route-local table; data fixture and localization check |
| AC-UI-004 | `source-extracted`: standard light/dark semantic surfaces | `Not verified`: dark/narrow status-row wrapping not yet captured | Status copy and filters reflow at required viewports without new tokens | P1 | root theme + route layout; viewport/contrast check |
| AC-UI-005 | `source-extracted`: public tracker is outside Admin UI | `Not verified`: host enable/opt-out and rejection statuses not yet exercised | No tracker initialization/request patch/request/IDs before enable; opt-out removes IDs; 413/429/507 persist zero events | P1 | host integration + Insights contract; browser/network and HTTP safety matrix |

New exact geometry, runtime computed styles, and public tracker network proof
remain `Not verified` after this source implementation; the status card's
permission and request gating are covered by source and query tests.

## Responsive and verification matrix

| Priority | Viewport | Theme/locale | Surface and state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 @ 100% | light / zh-CN | Overview populated | Four existing metrics retain hierarchy; status explanation, if present, does not create a second dashboard. |
| Required | 1440x900 @ 100% | dark / en-US | Details populated/partial | Filters, paths, and status text remain readable with AA contrast. |
| Required | 390x844 @ 100% | light / zh-CN | Overview empty/error/permission | No false zero metrics; retry and permission copy wrap without overflow. |
| Required | 390x844 @ 100% | dark / en-US | Details loading/empty | Table/filter content remains reachable; focus is visible. |
| Required | Browser network | light / zh-CN | Public tracker before/after opt-in | No initialization, request patch, or request before opt-in; rejected payloads contain no stored query/text fields. |

The last row is runtime/network evidence, not a visual screenshot claim. Two
same-viewport UI comparison passes and the tracker request audit are required
after implementation.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse the root `DESIGN.md`, current Analytics
components, `DataState`, route-local tables, and semantic theme tokens.

## Implementation status

The selected source, data/permission ownership, state contract, component
mapping, responsive/accessibility rules, and acceptance IDs remain fixed. The
route-local collection status card and manage-gated query are implemented for
the existing overview/details surfaces. Public opt-in behavior, server
rejection, final geometry, and browser network evidence remain `Not verified`
until runtime validation.
