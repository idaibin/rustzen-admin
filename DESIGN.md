---
name: "RustZen Admin"
description: "Shared visual semantics for the RustZen self-hosted operations console"
---

# RustZen Admin Design System

## Overview

RustZen Admin uses a restrained, precise visual system for developer-operators and
small technical teams. The interface prioritizes operational status, readable data,
clear actions, and explicit failure evidence. Visual refinement comes from hierarchy,
alignment, typography, spacing, borders, and state clarity rather than decoration.

## Authority and scope

This file is the sole shared visual-semantic authority for the repository-root Web
design boundary. Product specifications own behavior, permissions, failure semantics,
and acceptance outcomes. Feature UI specifications own page- or flow-local composition,
applicable states, responsive behavior, and acceptance. Frontend source implements this
contract but does not silently approve a conflicting visual decision.

The acceptance record for a revision is a named non-implementer human approval bound
to the exact file content hash. A changed hash requires a new approval. Task captures,
screenshots, generated concepts, historical QA, CSS values, and structured sidecars are
evidence or implementation; none is a second shared authority.

## Visual direction

- Use quiet operational density: compact enough for administration work, with enough
  separation to scan status, tables, and actions reliably.
- Make the primary task and critical status evident without decorative competition.
- Use solid surfaces, restrained borders, and clear hierarchy. Do not use gradient,
  glass, glow, ambient imagery, hover lift, bento filler, or oversized hero treatment
  as default Admin language.
- Keep one primary accent role. Success, warning, danger, information, focus, and chart
  series remain distinct semantic roles.
- Preserve the product's information architecture, routes, behavior, and accessibility
  unless their owning contracts separately authorize a change.

## Themes and color semantics

Light and dark are the only shared themes; light is the default. Both themes map the
same semantic roles:

- canvas, surface, raised overlay, foreground, muted foreground, and border;
- primary action/accent and visible keyboard-focus ring;
- success for healthy/completed, warning for attention/degraded, danger for failed or
  destructive, and information for active neutral status;
- independent chart-series and factual metric-category tones.

Color never carries status alone. Text, icon, label, or another non-color signal must
communicate the meaning. Dark mode preserves readable surface separation, focus,
hover, disabled, and overlay states rather than merely inverting light colors.

## Typography and data

Use the repository's system-compatible sans-serif stack for interface text. Page titles,
section titles, body/table text, and supporting copy form one descending hierarchy.
Operational numbers use tabular figures. Labels and supporting copy may wrap; primary
actions and status meaning must not disappear when Simplified Chinese or English text
expands. Avoid display typography or marketing-scale headlines inside the authenticated
console.

## Layout and density

The application shell owns global chrome, viewport clipping, and the broad content
boundary. The authenticated main content container owns one desktop page inset. A route
owns its page composition and vertical scroll region. A panel or reusable component owns
only its internal spacing. An overlay owns its stacking, focus, collision, and dismissal.

Do not stack equivalent shell, page, panel, and component insets. Align page headings,
actions, panels, empty states, and table headers to a named content edge. The document
must not scroll horizontally. A bounded table or long-content region may scroll
horizontally only when that owner is explicit and accessible.

The shared desktop rhythm uses a 24 px page inset, 24 px major separation, 16 px section
separation, and 8-12 px inline separation. Panels use a restrained 10 px radius and
ordinary controls use an 8 px radius with a 36 px default height. These values are shared
targets; feature contracts own justified exceptions and source owns the implementation.

## Shared component semantics

### PageHeader

Use one `PageHeader` for overview and detail surfaces. It owns the page title, concise
description, and directly related actions. It does not create a panel or a second page
inset. A page does not repeat the same title in another shared shell.

### PageCard

Use one `PageCard` for list and management surfaces. It owns the bounded surface that
groups title, description, primary action, optional toolbar, and content. It does not
redefine table columns, business actions, query behavior, or page-level permissions.

### MetricCard

Use `MetricCard` only for compact factual operational metrics with a label, dominant
value, and optional supporting hint. Icon and category tone are supporting cues, not
status by themselves. Routes own the metric selection, ordering, grid, and business
meaning. Do not use MetricCard for arbitrary content, module availability, charts,
forms, progress, or decorative KPI filler, and do not create route-local visual variants.

### DataState

Use `DataState` for shared query or process feedback. Preserve these distinctions:

- loading: no successful result is available yet;
- populated: successful usable data is present;
- empty: a successful result contains no applicable data;
- error: the owning operation failed and exposes recovery when available;
- permission: access is unavailable and is not presented as an empty result;
- processing: a long-running operation is active and reports progress when known;
- background refresh: keep the last successful data visible, identify stale/failure
  state, and provide retry without converting the surface to empty.

A feature contract lists only the states its real behavior exposes and justifies any
excluded state. Form validation and business-result semantics remain with their owning
feature or product contract.

### DataTableShell

Use `DataTableShell` as the bounded horizontal-overflow owner around a route-local data
table. The route owns columns, filters, pagination, row selection, and business actions.
The shell does not add a second card, page inset, or generic table domain abstraction.

### Overlays and feedback

Use the existing dialog, drawer, form, alert, tag, confirmation, and feedback primitives
for their established meanings. The default accepted page state contains no open
overlay. An open overlay must define its trigger, initial focus, focus trap, dismissal,
and focus restoration in the applicable feature contract.

## Interaction and accessibility

- All actions are keyboard reachable and have a visible focus indicator in both themes.
- Icon-only controls have an accessible name; color-only and hover-only disclosure are
  not acceptable.
- Dialogs and drawers trap focus while open, support expected dismissal, and restore
  focus to a sensible trigger.
- Loading and processing announce status without unnecessary interruption; errors and
  permission failures use appropriate alert semantics.
- Text and controls remain readable at zoom and with bilingual wrapping. Long values
  wrap, truncate with an accessible reveal, or scroll inside an explicit owner.
- Motion communicates feedback or state change only. Reduced-motion preference removes
  nonessential transitions and animation.

## Viewport acceptance baseline

- Primary: 1920x1080 CSS pixels, 100% zoom.
- Compatibility: 1440x900 CSS pixels, 100% zoom.
- Required themes: light and dark according to each feature's acceptance matrix.
- Smaller viewports: best effort unless a feature contract explicitly promotes a named
  size and state into required acceptance.

Each feature contract names its applicable state, locale, theme, overflow, focus, and
evidence matrix at these viewports. Static source, lint, build, screenshots, browser
runtime, and assistive-technology checks remain separate evidence levels.

## Reuse and extension

Reuse an existing shared owner when its meaning matches. Extend it only when the new
meaning is shared, has at least two compatible real consumers, and can be independently
verified. Otherwise compose or wrap existing primitives inside the feature. Do not add
a parallel token system, component library, generic shared directory, or route-local
theme override to avoid the shared owner.

Changes to shared themes, tokens, component meanings, state vocabulary, or cross-surface
visual rules require an updated DESIGN candidate, official lint/diff gates, and a fresh
named non-implementer approval bound to the exact content hash before implementation.

## Do's and don'ts

- Do keep operational hierarchy, status, actions, and recovery clear.
- Do keep one effective owner for each inset, scroll axis, overlay, and shared meaning.
- Do use real product copy and data states from their owning contracts.
- Do not infer exact tokens, accessibility, behavior, or approval from pixels alone.
- Do not copy shared visual semantics into feature specs, guides, YAML/JSON projections,
  task reviews, or generated prompts.
- Do not treat a build, lint result, screenshot, or isolated browser pass as complete
  visual and assistive-technology acceptance.
