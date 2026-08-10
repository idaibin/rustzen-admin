---
version: 1.0
name: "RustZen Admin"
description: "Repository-owned shared visual semantics for the Ant Design administration console"
---

# RustZen Admin Design System

## Authority

This document is the single shared visual-semantics entry point for RustZen
Admin. The selected source is the current accepted Admin UI / Ant Design
migration. It was approved by Daibin on 2026-07-25 through the current
repository request, and its rights status is repository-owned.

`apps/web/src/styles/theme.css` implements the concrete CSS custom-property
values. Do not duplicate those values in other persistent documentation.
`apps/web/src/components/theme-provider.tsx` remains the only theme state and
persistence owner.

## Visual anchors

- `#themes-and-surfaces`: standard light and dark themes only; use compact,
  solid semantic surfaces without gradients, glow, glass, or ambient imagery.
- `#layout-and-density`: the application layout owns global width; routes own
  vertical scrolling; use one `PageHeader` or `PageCard` title per surface.
- `#component-semantics`: reuse `MetricCard` for factual compact operational
  metrics, `DataState` for query feedback, and route-local Ant Design forms
  and tables for feature behavior.
- `#status-semantics`: use `--primary`, `--status-success`,
  `--status-warning`, `--status-danger`, and `--status-info` rather than
  literal status colors. The theme stylesheet owns their light and dark values.
- `#implementation-naming`: use PascalCase for exported React components and
  types, lower-kebab-case for component files and directories, TanStack route
  files that match their path, and Ant Design / Pro Components as the sole UI
  component library. Tailwind is limited to layout and spacing composition.
  Recharts is approved only as a factual trend or chart-rendering utility for
  the current Analytics and Monitoring consumers; it does not own shared UI
  components, interaction patterns, or theme semantics. Chart containers,
  loading/empty/error/permission states, and colors remain governed by this
  document and the existing shared components.

## Themes and surfaces

Light is the default theme. Cards, dialogs, forms, and tables use the shared
semantic background, foreground, panel, border, and muted anchors implemented
in the theme stylesheet. Preserve the existing compact readable density and
semantic status treatment.

## Layout and density

The authenticated layout owns the global page-width and overflow boundary.
Overview and detail routes use `PageHeader`; list and management routes use
`PageCard`. Do not duplicate a page title, create nested dashboards, or add
decorative KPI grids.

## Component semantics

`MetricCard` owns factual operational metrics on Dashboard, Monitoring, and
Analytics overview surfaces. Every instance uses the same two-zone anatomy:

- a neutral identity zone groups one semantic icon with the metric title;
- a separated, lightly tinted value zone makes the number the dominant element;
- icon, value, divider, and value-zone tint derive from one semantic tone;
- supporting copy is optional and must not be inserted merely to fill space;
- the component owns its internal spacing, type hierarchy, radius, border, and
  tone treatment, while the consuming route owns grid columns and inter-card
  gaps.

Use the component palette `blue`, `green`, `violet`, `amber`, and `red` by
metric category. These names describe the metric-card palette and do not
redefine global success, warning, or danger status semantics. Do not assign one
tone to an entire page, add route-local card variants, place decorative
gradients or glow inside a metric card, or use `MetricCard` for module
availability, progress, charts, forms, and arbitrary content.

`apps/web/src/styles/theme.css` owns each palette tone's foreground, soft
surface, and divider values. Its five metric tones must remain visibly distinct
in both light and dark themes. Routes never define metric-card color values.

`DataState` owns loading, empty, error, permission, and processing feedback.
Keep form validation, tables, actions, and query state with their route unless
a stable shared responsibility already exists.

## Status semantics

Choose a status anchor by meaning, not appearance: primary for the default
operational accent, success for healthy or completed state, warning for
attention or offline state, danger for unhealthy or failed state, and info for
active informational state. Ant Design status props remain appropriate where
they consume the configured theme context.

## Historical UI artifacts

`docs/ui/` contains current feature-local contracts plus retained historical
mappings and evidence. It is not a second design-system authority: shared
visual semantics begin here, while each UI slice records only its local
composition and validation evidence.

## Cross-feature interaction semantics

The four current feature slices reuse the same visual grammar even when their
business meanings stay with different module owners:

- `PageHeader` owns one overview/detail heading and its actions. `PageCard`
  owns list and management surfaces. A feature must not add a second page title
  or a decorative dashboard shell.
- `DataState` owns loading, empty, error, permission, and processing feedback.
  Query owners keep the last successful data visible during background refresh
  and supply retry. A failed request is never styled as a successful empty
  result.
- `DataTableShell` with a route-local ProTable owns paginated lists. Columns,
  filters, and business actions remain local until two real consumers prove the
  same semantics.
- Ant Design `Drawer`, `Modal`, `Form`, `Alert`, `Tag`, and `ConfirmDialog`
  remain the existing owners for detail, edit, warning, status, and destructive
  confirmation behavior. Wrap them locally when a feature needs composition;
  do not create a generic diagnostics or file-browser component.

When an operation affects multiple named items, `partial` is a semantic outcome
with explicit item-level success/failure and retry or review guidance. It is
never a new color, a generic success banner, or a replacement for `DataState`.

## Responsive and accessibility baseline

The authenticated layout owns global width and overflow; each route owns its
vertical scroll region. Feature contracts name their own viewport matrix, while
the shared baseline is:

- desktop acceptance at 1920x1080 and 1440x900 CSS px, 100% zoom;
- narrow acceptance at 390x844 CSS px, with content allowed to grow vertically;
- no document-level horizontal overflow; a bounded long-content region may
  scroll only when its owner and accessible label are explicit;
- visible keyboard focus, real button targets, focus trapping/restoration for
  dialogs and drawers, and text-first status meaning;
- Simplified Chinese and English copy that can wrap without clipping;
- standard light/dark themes and existing reduced-motion behavior, with no
  feature-specific gradient, glow, or decorative motion.

Exact dimensions, responsive transformations, and state copy belong to the
linked feature UI contract. This section stabilizes cross-page semantics only.

## Reuse and extension gate

Feature work uses the following decisions in order:

1. **Reuse** an existing owner when its semantics already match.
2. **Extend** an owner only when the new variant remains product-wide and has a
   stable second consumer.
3. **Wrap** an existing Ant Design or shared owner for one feature's local
   composition and state meaning.
4. **New** shared components are allowed only after the repository shared-
   capability gate proves at least two compatible consumers and an independent
   test. A route-local component is preferred until then.

No feature in the current product slice changes shared tokens or introduces a
new component library. API and persistence contracts remain with their module
owners; the visual system consumes their verified states without redefining
DTOs or business lifecycles.
