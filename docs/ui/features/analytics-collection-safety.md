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
composition, and DataState ownership. Overview and Details omit the removed
collection-policy status card and its display-only query. Ingestion safeguards and
backend management permissions remain unchanged; no settings system is added.

## Surface and layout contract

- Keep one `PageHeader` on Analytics overview and Details. Existing metric
  cards remain the factual summary and are not replaced by a collection-status
  dashboard.
- Do not show public-collection enabled state, project identifier or allowed-origin
  count as an extra status panel on either Analytics route.
- Details places type and path controls at the content upper right, using automatic
  search without submit/reset buttons. Applied filters start at page one. Other
  reports clear and disable the path input. Loading/error states retain both controls
  and input focus. The table fills available height with separate bottom pagination,
  including an empty successful response. Paths display
  pathname values only; long paths wrap or use a labeled bounded region rather
  than creating page-level horizontal scrolling.
- Public tracker opt-in is outside the Admin shell. The host page must perform
  the explicit enable call with `consent === true`; a script asset may be
  loaded first, but it remains inert until then. Before enable there is no
  request patch, request, queue, or local visitor/session identifier. Opt-out
  stops sends and removes those local IDs. No consent proof is displayed or
  claimed here.
- The public tracker keeps a fixed 1000-event local queue. When full it drops
  the newest event and reports `queue_dropped` through its transport observer.
  Visitor and session identifiers are generated as fixed-length UUID values.
  Each request is limited to 50 events and 64 KiB of UTF-8 JSON.
  Supported fields beyond the existing server bounds, measured in UTF-8 bytes,
  are rejected before queueing and reported as `event_dropped` with
  `field_too_long`; invalid field types or ranges, including object/array
  property values, use `invalid_field`. A 413
  response is reported once as `validation_rejected` and is never retried; only
  explicit 429/507 responses use the bounded retry path.

## Component and data-owner mapping

| Responsibility | Current owner | Decision |
| --- | --- | --- |
| Page title and actions | `PageHeader` | Reuse |
| Overview metrics | `MetricCard` | Reuse; values remain Insights-owned |
| Query feedback | `DataState` | Reuse |
| Event details table | Existing route-local table and `DataTableShell` where present | Reuse; keep event columns local |
| Type and path filters | Existing route-local Ant Design controls | Reuse; no global analytics filter store |
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
| Populated | Current metric/table hierarchy | Paths and event names remain text-readable; no raw query values. |
| Empty | `DataState` distinguishes no activity from no filter matches | Clear or change the current filters directly. |
| Error | Alert-semantic `DataState` with retry | Retry calls the owning query and keeps last successful data for non-permission failures. |
| Permission | Permission-semantic state or route guard | A 403 overrides cached Overview/Details data, including a background refresh; no status/config details leak to unauthorized users. |
| Partial | Table feedback identifies missing categories | Readable metrics/events remain visible; no all-good summary. |
| Tracker rejected | Not an Admin page state | Public endpoint returns explicit 413/429/507 or validation rejection with zero persistence; Analytics read data does not reset to empty. |

Color never carries permission or data-quality meaning alone. Overview and Details
do not expose policy configuration or a collection-status query.

## Accessibility and responsive behavior

- Overview and Details preserve one heading hierarchy and meaningful table
  headers. Empty and error feedback stays text-readable in both themes.
- Retry controls are keyboard reachable with existing visible focus. No
  non-interactive MetricCard gains a tab stop.
- Type/path filters wrap in source order at narrow widths. Pathnames and event
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
not infer consent or expose the raw project key. The tracker transport queue
and field-bound behavior are part of the public script contract, while the UI
does not render their counters or limits.

## Traceable UI deltas

| ID | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- |
| AC-UI-001 | User requirement: activity-only overview/details | Current routes omit the policy card | No policy-status card or display-only policy request; preserve activity hierarchy | P1 | Analytics composition and query inventory |
| AC-UI-002 | `source-extracted`: existing `DataState` semantics | Colima `linux/arm64` route-exact browser fixture passed the ten-case minimum closure: loading, Details empty, 403/500, filter page reset, background 500 data retention, and background 403 cache hiding. Other matrix coverage remains `Not verified`. | Loading, empty, error, permission, and partial remain distinct; no failed-to-empty conversion | P1 | source-bound fixture/browser manifest |
| AC-UI-003 | `source-extracted`: current event detail table | Source plus local Insights HTTP fixture verified: a legal pathname is queryable; query/fragment-bearing paths and non-pathname referrers reject before storage; the detail target uses only fixed safe event fields. Browser visual rendering remains `Not verified`. | Display pathname/event fields without query strings, page text, or arbitrary properties | P1 | Insights router HTTP fixture and route-local field behavior test; browser visual/data fixture check remains open |
| AC-UI-004 | User requirement: upper-right automatic filters | Source-resolved type/path controls | Shared responsive placement; no submit/reset button; other reports clear path | P1 | Filter, focus, empty-state and pagination checks |
| AC-UI-005 | `source-extracted`: public tracker is outside Admin UI | Linux Chromium host gate covers pre-opt-in, opt-in, opt-out, and real 413/429 through Admin -> Insights; Linux 507 is explicitly `Not verified` when safe capacity injection is unavailable | No tracker initialization/request patch/request/IDs before enable; pathname event persists after opt-in; opt-out restores hooks and clears IDs; 413/429 persist zero rows; Rust route seam covers 507 row preservation | P1 | independent `verify-analytics-tracker-linux.sh`, minimal host fixture, Insights route/query evidence |

Runtime observations are recorded in [local verification](../../guides/local-verification.md).
They do not certify production host-tracker opt-in, the complete state matrix or
all locale/viewport combinations.

## Responsive and verification matrix

### Passed minimum Analytics UI state-matrix closure

The route-exact controllable Insights fixture behind the real Admin UI passed
ten cases on Colima `linux/arm64`. The closure covers Overview/Details initial
loading, successful Details empty, exact 403 permission and 500 error states,
Details filter queries resetting pagination to page one, and a Details
background-refresh 500 retaining the prior row rather than rendering empty,
and Overview/Details background 403 responses hiding previously cached data.
The fixture's explicit `eventsFailAfterFirstStatus` and
`overviewFailAfterFirstStatus` controls affect only the second successful read
and accept only `403` or `500`; ordinary refresh uses `500` and permission
revocation uses `403`. Ordinary filter and pagination reads remain successful.

The source-bound manifest records fixture receipts, response modes, and the
query/page sequence. It contains only 1440x900 dark/en Overview success and
390x844 light/zh Details empty captures. All other viewport, theme, locale, and
state combinations, plus production and native systemd, remain `Not verified`.

| Priority | Viewport | Theme/locale | Surface and state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 @ 100% | light / zh-CN | Overview populated | Four existing metrics retain hierarchy with no policy-status panel. |
| Required | 1440x900 @ 100% | dark / en-US | Details populated/partial | Filters, paths, and status text remain readable with AA contrast. |
| Required | 390x844 @ 100% | light / zh-CN | Overview empty/error/permission | No false zero metrics; retry and permission copy wrap without overflow. |
| Required | 390x844 @ 100% | dark / en-US | Details loading/empty | Table/filter content remains reachable; focus is visible. |
| Required | Browser network | light / zh-CN | Public tracker before/after opt-in | No initialization, request patch, or request before opt-in; rejected payloads contain no stored query/text fields. |

The last row is runtime/network evidence, not a visual screenshot claim. The
independent Linux host gate writes source-bound evidence to
`target/rz/analytics-tracker/current/manifest.json`; its 507 status remains
explicitly `Not verified` unless a safe runtime capacity fixture exists. Two
same-viewport UI comparison passes and the tracker request audit are required
after implementation.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse the root `DESIGN.md`, current Analytics
components, `DataState`, route-local tables, and semantic theme tokens.

## Implementation status

The current UI contract preserves collection safety while removing the status panel.
Details uses automatic type/path filters, stable input controls and page-one queries.
Public host-tracker and deployment acceptance remain separate from these local UI changes.
