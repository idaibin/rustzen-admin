# Operational Metric Card Route Contract

Status: adopted shared authority; implementation verification pending for the current Worktree.

## Authority and selected source

- Profile: Feature UI.
- Product basis:
  [Operational Metric Card Visual Consistency](../../product/features/metric-card-visual-consistency/spec.md).
- Shared visual authority: root `DESIGN.md` at approved SHA-256
  `724874eec2c075c5cda20800458bc8cca89447a841076d375cd85d4f58d9fd88`.
- Selected source: the adopted shared `MetricCard` meaning plus the current factual
  Dashboard, Monitoring overview, and Analytics overview surfaces.
- Approval: daibin, human product/repository owner, approved the exact DESIGN content
  hash through controller task `019ff4d6-8b8a-7100-bada-b8cb46d3ed17`.
- Rights/use: repository-owned source and approved shared authority.
- Use: existing factual metrics, route-local ordering/grid, icons, localized labels,
  semantic category mapping, query states, and current API ownership.
- Ignore: the superseded 2026-07-31 crop as shared component authority, its inferred
  pixels, old 176 px/two-zone geometry, historical glass styling, speculative metrics,
  and any route-local shared-component variant.

This Feature UI contract does not repeat shared component anatomy, tokens, typography,
radius, spacing, theme, state vocabulary, or accessibility semantics from DESIGN.

## Route-local composition

### Dashboard

The route keeps this order:

1. one page heading;
2. four account metrics;
3. a permission-gated Admin-host resource summary;
4. runtime module availability.

The factual account and host-resource values remain in their current query boundaries.
Module availability is not a `MetricCard`. Detailed resource diagnostics stay on System
Status.

### Monitoring overview

The route keeps one summary row for registered, online, offline, unhealthy-check, and,
when permitted, active-incident counts. The active-incident evidence surface follows the
summary. Refresh metadata remains secondary to the page title and current operational
state.

### Analytics overview

The route keeps its current factual activity metrics and query ownership. It does not
reintroduce removed quality/performance metrics or add trends, confidence, or prediction
data.

Parent routes own grid columns, gaps, ordering, labels, values, icon selection, and
business meaning. They reuse the single shared `MetricCard`; they do not clone or
redefine it.

## Metric category mapping

| Surface | Metric | Category |
| --- | --- | --- |
| Dashboard | Total users | blue |
| Dashboard | Active users | green |
| Dashboard | Today's logins | violet |
| Dashboard | Pending users | amber |
| Dashboard | CPU | blue |
| Dashboard | Memory | green |
| Dashboard | Disk | amber |
| Monitoring | Registered nodes | blue |
| Monitoring | Online nodes | green |
| Monitoring | Offline nodes | amber |
| Monitoring | Unhealthy checks | red |
| Monitoring | Active incidents | violet |
| Analytics | Page views | blue |
| Analytics | Unique visitors | violet |
| Analytics | Total events | green |
| Analytics | API requests | amber |

Categories distinguish factual groupings; health and failure meaning still includes text
outside color-only presentation. Routes use their existing icon package and do not add a
second icon system.

## Applicable state matrix

| Surface | Loading | Populated | Empty/zero | Error | Permission | Background refresh |
| --- | --- | --- | --- | --- | --- | --- |
| Dashboard account metrics | Required | Required | Successful zero values | Required with retry | Page access owner | Keep last successful values when available |
| Dashboard host resources | Required when permitted | Required | Successful zero values | Required with retry | Region absent without `system:status:view` | Keep last successful values when available |
| Dashboard modules | Required | Required including unavailable module | Not applicable; fixed modules remain named | Required with retry | Existing navigation/capability owner | Existing polling keeps visible data |
| Monitoring summary | Required | Required | Registered-node zero remains distinguishable from request failure | Required with retry | Incident metric is permission-gated | Keep last successful summary and expose retry |
| Analytics summary | Required | Required | Successful zero values | Required with retry | Existing route/capability owner | Keep last successful summary when available |

A failed initial request never becomes a valid zero/empty metric surface. Permission
rules remain with product/source owners and are not redefined here.

## Viewport and accessibility acceptance

| Priority | Viewport | Theme/locale coverage | Required surfaces |
| --- | --- | --- | --- |
| Primary | 1920x1080 CSS px, 100% zoom | light, zh-CN; representative keyboard/focus and state checks | Dashboard, Monitoring, representative data table |
| Compatibility | 1440x900 CSS px, 100% zoom | light/dark and zh-CN/en-US wrapping across representative surfaces | Dashboard, Monitoring, representative data table |
| Excluded from this task | smaller viewports | best effort | No completion claim; a future slice may promote a named viewport |

At required viewports:

- parent grids preserve source/reading order and do not create document-level horizontal
  overflow;
- labels, values, hints, and status text remain readable without hiding primary actions;
- the shared cards remain non-interactive and do not enter the tab order;
- visible focus is verified on nearby interactive controls;
- final composed foreground/background contrast is checked in light and dark;
- loading, error/retry, successful zero, permission-gated, and background-refresh paths
  are exercised where the table above marks them applicable.

## Acceptance-to-owner mapping

| ID | Acceptance | Owner | Decision | Verification |
| --- | --- | --- | --- | --- |
| MC-001 | One shared factual metric component across three overview domains | `MetricCard` and current route consumers | Reuse | Source scan plus rendered consumer count |
| MC-002 | Dashboard order and factual metric scope remain unchanged | Dashboard route | Reuse current product behavior | DOM order and populated runtime |
| MC-003 | Monitoring metric/incident hierarchy and refresh states remain explicit | Monitoring overview route | Reuse/wrap route-local composition | Loading, populated, error, permission, refresh runtime |
| MC-004 | Analytics retains only current factual activity metrics | Analytics overview route | Reuse | Source and populated runtime |
| MC-005 | Category mapping is consistent without color-only status | Route data plus existing theme adapter | Reuse DESIGN semantics | Light/dark computed color and text checks |
| MC-006 | Required desktop viewports have no document overflow or clipped actions | Shell, route grids, shared component | Reuse/adjust minimally | Computed geometry at both required viewports |

## Readiness

Ready for `dev-frontend` metric-card route alignment.

Shared-system changes beyond adopting root DESIGN: none. Exact implementation geometry is
validated against the adopted DESIGN and required viewports; it is not redefined here.
Runtime completion requires two comparison passes, computed geometry/style checks, the
applicable state matrix, and no unresolved P0/P1 finding.
