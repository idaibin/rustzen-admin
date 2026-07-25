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

`MetricCard` owns the compact factual metric presentation for dashboard,
Monitoring, and Analytics overviews. Its optional icon tone is semantic, not a
feature-local color. `DataState` owns loading, empty, error, permission, and
processing feedback. Keep form validation, tables, actions, and query state
with their route unless a stable shared responsibility already exists.

## Status semantics

Choose a status anchor by meaning, not appearance: primary for the default
operational accent, success for healthy or completed state, warning for
attention or offline state, danger for unhealthy or failed state, and info for
active informational state. Ant Design status props remain appropriate where
they consume the configured theme context.

## Historical UI artifacts

`docs/ui/` contains task-local or historical mappings and evidence from the
prior UI-standardization task. They are not a second design-system authority;
current shared visual work begins here, then verifies the live implementation.
