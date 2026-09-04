# Operational Metric Card Visual Consistency

## Goal and implementation slice

Present factual operational metrics with one recognizable hierarchy across the
Dashboard, Monitoring overview, and Analytics overview: metric identity first,
the numeric value as the strongest content, and a distinct semantic color for
each metric meaning.

This is one connected frontend presentation slice. It does not change the data
returned by the existing Dashboard, System Status, Monitoring, or Analytics
APIs.

## Users and scenarios

- An operator scans account totals on the Dashboard and can distinguish total,
  active, recent-login, and pending-user metrics without reading every value.
- An owner scans the Dashboard and can distinguish the Admin host's CPU,
  memory, and disk pressure without opening the detailed System Status page.
- An operator scans Monitoring health metrics and can distinguish inventory,
  healthy, attention, failure, and incident meanings.
- An operator scans Analytics activity metrics and can distinguish page,
  visitor, event, and request measures while retaining the same card grammar.
- The same hierarchy remains understandable in Simplified Chinese and English,
  light and dark themes, and stacked narrow layouts.

## Confirmed decisions

- Factual count cards reuse one shared `MetricCard` component; host resource
  percentages remain in the existing progress-based summary.
- Every metric retains a visible title, numeric value, and semantically related
  icon. Supporting copy remains optional.
- Each metric receives its own semantic category; a page-wide default category
  is not acceptable and color is never the only carrier of meaning.
- Existing real values and request boundaries remain authoritative. A failed
  request is never converted into a successful zero state.
- Shared component meaning, hierarchy, tone treatment, and visual semantics
  belong only to root `DESIGN.md`. The linked UI contract owns route-local
  mapping, state coverage, responsive composition, and acceptance.

## Scope and non-goals

In scope:

- the four Dashboard account metrics;
- the three permission-gated Dashboard system-resource metrics;
- the Dashboard account metric row preceding resource and module summaries;
- the four Monitoring overview metrics;
- the four current Analytics core-activity metrics;
- one semantic tone contract and one optional-supporting-copy API;
- responsive reflow without document-level horizontal overflow;
- light/dark and Chinese/English presentation of the existing values.

Non-goals:

- module availability/status rows on the Dashboard;
- charts, resource progress cards, tables, forms, query states, or arbitrary
  Ant Design `Card` content;
- API response, persistence, permission-policy, navigation, or metric-definition
  changes;
- adding new metrics, secondary trends, percentages, sparklines, or actions;
- redesigning every general-purpose card in the system.

## Main and failure flows

1. Each route loads data through its existing query boundary.
2. A populated response maps each existing metric to a title, value, icon, and
   semantic tone and renders the shared `MetricCard`.
3. The card presents the verified value without adding substitute or decorative
   data.
4. At narrower widths, the parent grid changes column count while the component
   preserves its hierarchy and does not introduce horizontal overflow.
5. Initial loading, error, retry, background-refresh error, and successful empty
   behavior remain owned by the existing route/query boundary. The card does not
   invent substitute data or new feedback states.

## Business rules and data effects

- Metric values come only from the current route API responses.
- Dashboard system-resource metrics use the existing System Status response and
  render only when the current user already has `system:status:view`.
- Zero is valid only for a successful populated response.
- Tone communicates category or operational meaning and must not be the only
  carrier of a health/status fact.
- Card presentation has no write-side data effect.
- Dashboard presents account metrics before the resource and runtime-module summaries.
- Metric order, localization, and permission behavior remain unchanged.
- The runtime-module query keeps its 15-second background polling without showing
  a redundant refresh-interval label in the panel.
- The removed Analytics quality/performance trio (error count, average duration,
  and P95 duration) remains deferred by the existing product decision and is not
  reintroduced by this visual slice.

## Acceptance criteria

- Dashboard renders four account count cards plus three permission-gated host resource
  values. Monitoring renders four count cards; Analytics renders four activity count cards.
- Dashboard count cards form one row without a redundant enclosing account Card.
  Resource and module summaries follow; a module state remains readable as text.
- Every card contains a visible title, a semantically related icon, and a
  dominant tabular numeric value; supporting copy remains optional.
- Adjacent metric categories remain distinguishable without relying on color
  alone.
- No route introduces a local metric-card clone, local card palette, or page-wide
  single-tone override.
- Existing loading, error, retry, populated-zero, and background refresh behavior
  is unchanged.
- The implementation meets every `MC-*` item in
  `docs/ui/features/metric-card-visual-consistency.md` and completes two
  same-viewport visual comparison passes before completion is claimed.
- Frontend format, lint, typecheck, production build, desktop target, narrow
  breakpoint, light/dark, Chinese/English, focus, contrast, and horizontal
  overflow checks pass for the affected surfaces.

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- Existing `@ant-design/icons` assets are sufficient; a new icon library is not
  required.
- The existing theme adapter can implement the semantic roles approved in root
  `DESIGN.md` without becoming a second authority.

### Open questions

- None for product behavior. Visual decisions are resolved through the adopted
  root `DESIGN.md` and the route-local UI contract.

### Rejected

- Applying the same pink or any other single tone to every metric.
- Duplicating the component in each route.

### Deferred

- Trend comparisons, click-through behavior, animation, and configurable card
  density.
- Server uptime, load average, process metrics, remote-node metrics, and
  detailed SQLite/runtime-directory storage on the Dashboard.
- Reintroduction or redesign of Analytics error count, average duration, and P95
  duration metrics.

## Ready for operational metric-card alignment

The product scope, data boundary, affected consumers, failure semantics, and
non-goals are fixed. Shared visual semantics are owned by root `DESIGN.md`;
route-local mapping, state coverage, responsive rules, and UI evidence are
owned by `docs/ui/features/metric-card-visual-consistency.md`.
