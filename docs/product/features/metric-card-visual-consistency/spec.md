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

- All factual overview metrics reuse one shared `MetricCard` component.
- The numeric value is the dominant content; title and icon identify the metric.
- Supporting copy is optional and is not required for the current metric set.
- Each metric receives its own semantic tone. A page-wide default tone is not
  acceptable.
- The accepted direction is a neutral title region plus a separate lightly
  tinted value region, not a fully tinted card.
- Existing real values and request boundaries remain authoritative. A failed
  request is never converted into a successful zero state.
- Shared visual semantics belong to root `DESIGN.md`; exact slice geometry and
  visual acceptance belong to the linked UI contract.

## Scope and non-goals

In scope:

- the four Dashboard account metrics;
- the three permission-gated Dashboard system-resource metrics;
- one Dashboard account-overview panel placed before the runtime-module panel;
- the five Monitoring overview metrics;
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
3. The card keeps the title and icon together in its identity region and renders
   the value alone as the strongest element in its value region.
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
- Dashboard presents the account-overview panel before the runtime-module panel.
- Metric order, localization, and permission behavior remain unchanged.
- The runtime-module query keeps its 15-second background polling without showing
  a redundant refresh-interval label in the panel.
- The removed Analytics quality/performance trio (error count, average duration,
  and P95 duration) remains deferred by the existing product decision and is not
  reintroduced by this visual slice.

## Acceptance criteria

- Dashboard renders 4 account metrics and, for an authorized owner, 3 host
  resource metrics; Monitoring renders 5 and Analytics renders 4 existing
  metrics through the same exported `MetricCard` implementation.
- Dashboard groups the four account metrics inside one account-overview `Card`
  before the runtime-module `Card`; the runtime-module panel has no visible
  refresh-interval label.
- Every card contains a visible title, a semantically related icon, and a
  dominant tabular numeric value; supporting copy remains optional.
- Adjacent metric categories visibly differ through their semantic icon/value/
  value-surface treatment, while the identity surface remains neutral.
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
- The shared theme stylesheet can own component-level tone tokens when a generic
  semantic anchor does not remain distinct in both themes.

### Open questions

- None. Daibin confirmed the linked UI contract's geometry and per-metric
  palette mapping on 2026-07-31 with the instruction to begin execution.

### Rejected

- Applying the same pink or any other single tone to every metric.
- Tinting the entire card with one color.
- Keeping the current compact 130px treatment when it does not match the accepted
  hierarchy.
- Duplicating the component in each route.

### Deferred

- Trend comparisons, click-through behavior, animation, and configurable card
  density.
- Server uptime, load average, process metrics, remote-node metrics, and
  detailed SQLite/runtime-directory storage on the Dashboard.
- Reintroduction or redesign of Analytics error count, average duration, and P95
  duration metrics.

## Ready for operational metric-card alignment

The product scope, data boundary, affected consumers, failure semantics,
non-goals, and acceptance criteria are fixed. Visual source, proposed geometry,
proposed palette mapping, responsive rules, and UI evidence are owned by
`docs/ui/features/metric-card-visual-consistency.md`; its owner-confirmation
gate is closed.
