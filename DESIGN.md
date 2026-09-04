---
name: "Rustzen Admin"
description: "Shared visual semantics for the Rustzen self-hosted operations console"
colors:
  canvas: "#F2F5FA"
  sidebar: "#FBFCFE"
  header: "#FFFFFF"
  content: "#FFFFFF"
  surface-subtle: "#F5F7FB"
  selected-child: "#EAF2FF"
  primary: "#1769E8"
  foreground: "#172033"
  foreground-muted: "#64748B"
  success: "#16A34A"
  warning: "#F59E0B"
  danger: "#E5484D"
  info: "#4F46E5"
typography:
  page-title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 26px
    fontWeight: 600
    lineHeight: 34px
  section-title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 18px
    fontWeight: 600
    lineHeight: 26px
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 14px
    fontWeight: 400
    lineHeight: 22px
  supporting:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 13px
    fontWeight: 400
    lineHeight: 20px
  table-header:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 13px
    fontWeight: 600
    lineHeight: 18px
  metric-value:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: 28px
    fontWeight: 600
    lineHeight: 36px
    fontFeature: "tnum"
spacing:
  tight: 4px
  compact: 8px
  related: 12px
  component: 16px
  large: 20px
  page: 24px
  section: 32px
rounded:
  control: 8px
  panel: 10px
  full: 9999px
components:
  shell-sidebar:
    backgroundColor: "{colors.sidebar}"
    width: 260
  shell-header:
    backgroundColor: "{colors.header}"
  shell-content:
    backgroundColor: "{colors.content}"
    padding: "{spacing.page}"
  input-default:
    backgroundColor: "{colors.surface-subtle}"
    rounded: "{rounded.control}"
    height: 36px
  button-primary:
    backgroundColor: "{colors.primary}"
    rounded: "{rounded.control}"
    height: 36px
  icon-button:
    rounded: "{rounded.control}"
    width: 32px
    height: 32px
  navigation-child-active:
    backgroundColor: "{colors.selected-child}"
    textColor: "{colors.primary}"
    rounded: "{rounded.control}"
---

# Rustzen Admin Design System

## Overview

Rustzen Admin is a restrained, precise operations console for developer-operators and
small technical teams. It prioritizes operational status, readable data, clear actions,
and explicit failure evidence. Refinement comes from typography, alignment, consistent
spacing, solid background layers, and state clarity rather than decoration.

### Authority and status

This file is the repository-root Web boundary's sole shared visual-semantic authority.
Product specifications own behavior, permissions, route inventory, failure semantics,
and acceptance outcomes. Feature UI specifications own page-local composition, states,
responsive behavior, and acceptance. Generated concepts, screenshots, prompts, CSS,
and `.codex/reviews` files are evidence or task material, not parallel authorities.

This revision is a candidate until a named non-implementer approves its exact SHA-256.
Any later byte change invalidates that approval and requires a new one.

### Visual direction

- Use quiet operational density: compact enough for administration work, with enough
  breathing room to scan status, tables, and actions.
- Use Ant Design surfaces, borders, and state fills for hierarchy. Do not add a parallel
  card-border grid, decorative shadow system, gradients, glass, glow, hover lift, bento
  filler, or oversized hero treatment.
- Keep one primary accent. Success, warning, danger, information, focus, and chart
  series remain separate semantic roles.
- Preserve real information architecture and behavior. A visual design must not invent
  routes, modules, permissions, metrics, or actions.

## Colors

Light is the canonical and default theme. The authenticated shell maps `canvas`,
`sidebar`, `header`, and `content` through the shared theme and repository-owned AppShell.
The user-selected layout has a canvas gutter separating sidebar, global header and content.
Ant Design supplies components, not the application geometry.

- `surface-subtle` is the visible fill for search inputs and table headers. Inputs must never visually dissolve into their parent surface.
- `selected-child` marks only the active second-level route. A top-level leaf may use it
  when active, but an expanded parent does not receive the same background.
- `primary` is reserved for primary actions, active text, and focus indication.
- `foreground` is used for titles and important data; `foreground-muted` is used for
  descriptions, hints, timestamps, and secondary metadata.
- `success`, `warning`, `danger`, and `info` are semantic, not decorative. Pair every
  status color with a readable label and a consistent status icon.
- Pale semantic fills are allowed when meaning is explicit. Do not color every card or
  put arbitrary multicolor squares behind icons.
- Borders and hairlines come from Ant Design component/theme tokens or factual
  visualization. Do not add route-local separator colors or duplicate a component's
  built-in border ownership.

Dark mode, where retained by the product, must map the same semantic roles and preserve
surface, focus, hover, disabled, and overlay contrast. It is compatibility behavior, not
the canonical design-source theme for the 1920x1080 page suite.

## Typography

### Nodes onboarding

The Nodes onboarding drawer uses Ant Design ordered steps. It never renders a secret
input, copied shell block, token, or direct Agent-start command. The execution primary
action remains disabled while the console cannot obtain signed release files or provision
a local root-only secret.

Use the declared system-compatible sans-serif tokens. Do not use marketing display type
inside the authenticated console.

- `page-title`: one route title per page; never repeat it in the global header.
- `section-title`: compact panel and section heading.
- `body`: default data, controls, and table cells.
- `supporting`: descriptions, timestamps, hints, and metadata.
- `table-header`: one line, visually quieter and smaller than row content.
- `metric-value`: factual operational numbers with tabular figures.

Primary actions and status meaning must remain visible when Simplified Chinese or English
copy expands. Supporting copy may wrap. Long identifiers and values wrap, truncate with
an accessible reveal, or scroll only inside an explicitly owned region.

## Layout

### Canonical AppShell

The primary verification frame is 1920x1080 CSS pixels, with 1440x900 compatibility.
The user-selected reference defines three separate rounded surfaces: a full-height left
sidebar, a global header at the top right, and one content panel below the header.
The repository-owned AppShell uses CSS Grid with 16px outer clearance and 16px gaps,
a fixed 260px sidebar, a 64px header, and 12px panel radii. Desktop navigation remains
expanded; narrow-screen navigation opens in an Ant Design Drawer.
Ant Design Menu, Button, Avatar, Dropdown and Drawer retain component behavior; ProLayout
must not impose its default geometry on this shell.

Below 768px, outer clearance and gaps are 8px, the header is 56px, and navigation opens
in an Ant Design Drawer. The content panel owns vertical scrolling; its inner page has
24px padding on desktop and 16px on narrow screens. Header and sidebar remain stationary.
Route switching preserves the same shell and must not introduce document horizontal scrolling.

### Spacing ownership

Use the Ant Design-aligned spacing steps `tight` 4px, `compact` 8px, `related` 12px,
`component` 16px, `large` 20px, `page` 24px, and `section` 32px. Choose the smallest
step that preserves the relationship: icons and labels use compact spacing, related
controls use compact or related spacing, component internals use component spacing,
desktop page inset uses page spacing, and major groups may use large or section spacing.

Dimensions, icon sizes, typography, and radii are not spacing tokens. Shell owns chrome
and broad content boundaries; page owns composition and vertical scroll; component owns
internal spacing; overlay owns stacking, focus, collision, and dismissal. Never stack
equivalent insets or introduce a route-local spacing scale.

### Shared dimensions and density

- Default control and primary page action are 36px; icon button is 32x32px.
- Ant Design `Menu` owns the 40px item height and the indentation of parent and child rows.
- Ant Design `Table` owns row density through its configured cell padding; routes do not
  hard-code a competing universal row height.
- Page headings, actions, panels, empty states, and table headers align to the same page
  content edge.
- Document-level horizontal scrolling is forbidden. A bounded table or long-content
  owner may scroll horizontally when accessible.

## Elevation & Depth

Rustzen Admin is flat and tonal. Depth comes from separate shell panels, Ant Design surfaces and
borders, whitespace, and semantic state fills. Sidebar, header, content, cards, tables,
and overlays retain their configured Ant Design treatment; routes do not add a parallel
shadow, gradient, glass, or glow system.

Raised overlays may use the existing accessible overlay treatment. Overlays are absent
from default page acceptance; an open state must define trigger, initial focus, trap,
dismissal, collision handling, and focus restoration.

## Shapes

- Use `control` 8px for inputs, ordinary buttons, active menu items, chips, and compact
  interactive surfaces.
- Use `panel` 10px for configured Cards and bounded panels. The AppShell panels use 12px radii.
- Use `full` only for status dots, avatars, and semantically circular controls.
- Do not mix unrelated radii or add decorative icon tiles. A pale circle is allowed only
  for an explicit factual metric or semantic status.

## Components

Ant Design is the default component source for the Web application. Prefer existing
`antd` and `@ant-design/pro-components` components and their supported composition
patterns before introducing custom controls, tables, pagination, forms, overlays, or
feedback surfaces. Tailwind owns layout and spacing only. A custom component is allowed
only when the owning product requirement cannot be met by the installed Ant Design
stack; document the concrete gap and preserve Ant Design tokens, semantics, and
accessibility behavior.

### AppShell and global header

The AppShell owns shared desktop geometry and solid background separation. The global
header stays on one horizontal alignment line and never shows the route title.

The `shell-sidebar`, `shell-header`, and `shell-content` component tokens bind the three
persistent shell surfaces to their shared fill, sizing, and inset semantics.

- Page search uses the repository's filled Ant Design `Button` at the header's left and opens the Ant Design search `Modal`. Preserve its search icon, readable
  label, keyboard shortcut, grouped results, empty state, and keyboard selection; do
  not replace it with a fixed full-width input copied from generated assets.
- `input-default`, `button-primary`, and `icon-button` define the shared filled input,
  primary page action, and compact icon-action geometry.
- Search remains at the left; language, theme, and avatar/account align at the right.
  The shell owns responsive spacing; never use absolute screenshot coordinates.
- Language and account are text-plus-icon controls; theme is an icon button. Do not wrap
  each control in a heavy pill.

### Sidebar navigation

- The sidebar brand uses a stable 28px logo; brand text is the source-owned 16px semibold label.
  Logo and text share one visual centerline.
- Navigation icons use the installed Ant Design icon rendering and one coherent outline family.
- Ant Design route items render the repository-provided icon for both parent and child
  items. Do not strip child icons or replace the Ant Design Menu with a parallel list.
- An expanded parent stays transparent. When a descendant is active, parent icon, label,
  and chevron use `primary`, but only the active child gets `selected-child` background.
- `navigation-child-active` is the only shared filled child-navigation state.
- Children use Ant Design Menu indentation and keep one stable label edge. They have no
  bullets, leading dots, tree rails, timelines, or icons outside the route-provided
  Ant Design icon family.
- Chevron changes direction without shifting row geometry. Expanding one group must not
  change another primary row's height or padding.

### Iconography

Use one outline family visually compatible with the repository-owned Ant Design Icons.
Do not mix outline, filled, cartoon, 3D, or duotone families. The product owner is the
repository frontend; library package rights govern reuse. When a required semantic icon
is absent, use a reviewed icon from the same family rather than generating SVG source.

- Navigation and control icon sizing follows the owning Ant Design component. Larger
  factual summary icons are allowed only when the owning shared component defines them.
- Status uses check-circle, warning-triangle, close-circle, or info-circle plus text.
- Familiar table actions use icon-only controls—view, edit, history, play,
  enable/disable, delete, deploy—with tooltip and accessible name.

### PageHeader and PageCard

PageHeader owns one title, a description aligned beside it on desktop, and directly related
actions. On narrow screens the description may wrap below the title. It is
inside the white content panel, with no separate background, outer inset, or title card.
PageCard composes the title and an unpadded ProCard content region; the shell alone
owns the page inset. It does not redefine columns, queries or permissions. Search and filter
controls sit at the upper right, beside the title and page actions. Use compact inline
controls on desktop and wrap within the panel on narrow screens; no full-width filter strip.
Text searches apply after 300 ms without typing, pause during IME composition, and clear
immediately. Select/date changes apply immediately. Filter changes reset pagination before
querying; do not show Search or Reset submit buttons. The user list offers username and
status only; monitoring summaries have no search control.

Ordinary content cards use the content surface with subtle borders; metric cards use the
shared blue, green, violet and amber tone fills and their dark-theme counterparts.
Module health uses a status indicator plus text rather than a large green tile. Empty
states use a compact blue icon surface and readable copy, not a gray illustration.
The four global alert controls share one configuration panel with outlined number inputs.
The table stays on the content fill with a subtle header. Do not split the route title
and table into two standalone panels.

### AppShell navigation

Use the repository-owned route data and Ant Design Menu for authenticated navigation.
Keep permission filtering, route order, icons, active keys, submenu behavior and keyboard
navigation. Navigation icons remain 14px. Desktop sidebar collapse is not provided; the mobile
AppShell Drawer owns narrow-screen navigation while the Menu owns indentation, submenu arrows
and popup behavior.

### MetricCard and DataState

`MetricCard` is only for compact factual metrics with label, dominant value, and optional
hint. Icon and tone support meaning; they are not decoration or status by themselves.

Shared data/process states keep these meanings distinct: loading, populated, empty,
error, permission, processing, and background refresh. Background refresh retains the
last successful data, identifies stale/failure state, and offers retry. Feature contracts
declare only states the real behavior exposes.

### DataTableShell

`DataTableShell` owns bounded horizontal overflow around a route-local table. The route
owns columns, filters, pagination, selection, and business actions.

- Use the repository-owned Ant Design `ProTable` / `Table` and `Pagination` components.
  Do not draw or maintain a parallel custom table or pager. The route source remains
  authoritative for whether pagination is enabled, disabled with `pagination={false}`,
  or hidden on a single page.

- Table header uses `surface-subtle`, `table-header`, and a single line. Rebalance widths
  or shorten labels instead of wrapping.
- Rows use whitespace and hover/state fills, not divider lines or a border grid.
- Menu administration is a flat navigation inventory because the current data owner
  supplies flat rows. Render it with `ProTable` and `pagination={false}`. If product
  behavior later introduces hierarchical `children`, use Ant Design expandable tree
  data and its built-in expand control rather than simulated indentation.
- Action columns use Ant Design fixed-column behavior when needed and reserve enough
  route-owned width for the real actions, tooltips, and a readable right inset. Do not
  impose one universal pixel width or clip actions into the page edge.
- A paginated data-list page uses one full-height table owner. Keep Ant Design pagination
  inside that owner and align it to the owner's bottom edge; record count stays left and
  pagination stays right. Do not stretch rows or invent records to fill unused height.
  Pages with `pagination={false}` do not render a placeholder page button.

### Feedback and details

- Transient success, information, warning, and error feedback uses the existing Ant
  Design `App` message or notification APIs and floats without consuming document-flow
  height. Persistent page state may use `Alert` only when it must remain in the reading
  flow.
- Contextual diagnostics, evidence, and row details use Ant Design `Drawer` or `Modal`
  when they should not reduce the table's available height. Preserve trigger, dismissal,
  focus, collision, and focus-restoration behavior defined by the owning feature.

## Do's and Don'ts

- Do keep operational hierarchy, status, timestamps, failure evidence, actions, and
  recovery visible.
- Do use the Ant Design-aligned 4/8/12/16/20/24/32 spacing steps to create hierarchy.
- Do keep one owner for every inset, scroll axis, overlay, and shared meaning.
- Do keep header, sidebar, logo, menus, search, and content ownership stable across all
  authenticated routes through the shared AppShell.
- Do verify canonical design-source PNGs are independent 1920x1080 files; a collage is
  an optional index, never the primary delivery.
- Do keep color-independent status labels, keyboard focus, accessible icon names,
  reduced-motion behavior, bilingual wrapping, and zoom readability.
- Do not add parent and child selection backgrounds at the same time.
- Do not add submenu bullets, oversized navigation rows, multicolor icon tiles, wrapped
  table headers, text-heavy action columns, clipped actions, or detached pagination.
- Do not add permanent sparse detail panes, detached helper panels, giant filler regions,
  or marketing-style authenticated-console content.
- Do not copy these shared semantics into prompts, feature specs, YAML/JSON sidecars, or
  route-local themes. Reference this file instead.
- Do not treat lint, build, screenshots, browser runtime, or generated output as human
  approval or complete accessibility acceptance.
