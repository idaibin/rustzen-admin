# Dashboard And Admin Navigation Simplification

Status: current route contract synchronized with the user's console requirements.
Observed local coverage and remaining gaps belong to [local verification](../../guides/local-verification.md).

## Scope and product facts

This is one control-plane entry slice covering:

- `/`: account totals, permission-gated Admin-host CPU/memory/disk summary, and
  the availability of Monitor, Insights, and Reports;
- the persistent sidebar and page search: current product destinations only;
- removal of Dictionary and demo navigation from the current surface.

Detailed host resources and storage diagnostics belong to `/system/status`;
the Dashboard may project only CPU, memory, and disk usage for users who can
already view System Status. Monitoring and Analytics own their metrics and
trends. Dictionary has no current in-repository consumer.
The historical SQLite table is dormant upgrade data, not a UI or API contract.
The `403` and `404` pages remain reachable for real routing and authorization
failures even though they are not navigation examples.

## Visual-source mapping

The project-owned product source is `docs/product/product.md`. Shared visual
semantics and component ownership are defined only by root `DESIGN.md`;
`apps/web` source is its current implementation adapter, and
`docs/ui/profile.yaml` is historical task evidence rather than an accepted
visual source. This slice only owns Dashboard and navigation composition.

## Layout, components, and tokens

- `PageHeader` owns the single page heading and description.
- The Dashboard starts directly with four account metric cards, without a redundant
  account-overview title or enclosing Card. A permission-gated resource panel and
  module panel follow side by side where space allows, then stack in that DOM order.
- Resource and module panels use the ordinary bordered surface from DESIGN. Module
  entries use neutral compact cards, an icon, and an explicit state indicator plus
  text; an all-green tile is not the only indication of availability.
- Reuse `Card`, `MetricCard`, `DataState`, `Badge`, and `Button`. Do not
  introduce charts, progress rings, tabs, a quick-action framework, a nested
  dashboard, or a new metric-card variant for this slice.
- Reuse the adopted root `DESIGN.md` semantics through the existing theme
  adapter. No shared semantic change is authorized by this slice.
- Sidebar groups remain System and Management plus enabled module-provided
  navigation. Search consumes the same route inventory as the sidebar.

## States, transitions, and feedback

Dashboard account totals, system resources, and module health are independent
asynchronous regions. Each owns its loading, error with retry, and populated
state through `DataState`; failure in one must not hide the others. The system
resource region is absent when the user lacks `system:status:view`. A module
marked unavailable is valid populated data, not a page error. Empty account
totals render numeric zero values.

Sidebar/search entries are derived from current fixed Admin routes, granted
permissions, the configured module-enabled state, and active, visible,
reconciled module menu metadata. An enabled module's otherwise visible entries
stay visible when its service is unavailable or its current runtime contract is
incompatible; opening such an entry may render the existing unavailable/error
response, but the UI must not silently remove the entry. Disabling a module or
manually hiding a menu removes the affected entries. Removing Dictionary and
demo entries must remove them from both navigation and search. No UI transition
may reactivate an inactive core permission. Language and theme switches
preserve the current route and data state.

## Responsive and accessibility rules

- Target sizes: 1920x1080 and 1440x900; the page and representative routes must
  have no document-level horizontal overflow.
- Also check the current console at 1705×1039, 1024×768 and 390×844; narrower
  grids must preserve order, readable metrics and reachable module controls.
- Preserve account overview, system running summary, and module availability in
  that DOM and keyboard order when the layout stacks.
- Keep one semantic `h1`, visible focus behavior from existing primitives,
  textual availability labels in addition to color, localized control names,
  and live status semantics supplied by `DataState`.
- Theme changes must preserve readable foreground/surface contrast. This slice
  adds no animation and therefore introduces no reduced-motion exception.

## Executable acceptance

1. `GET /api/dashboard/stats` remains the Dashboard-owned account endpoint;
   module health continues through `GET /api/dashboard/modules`; the resource
   summary reuses `GET /api/system/status` without changing its permission.
2. Dashboard renders its page heading, four account metrics, three
   permission-gated Admin-host resource metrics, and module availability. It
   adds no storage diagnostics, request metric, trend, or duplicate Analytics
   panel.
3. Sidebar and search contain no `/manage/dict`, `/403`, or `/404` entry, while
   direct `403` and `404` routing still renders.
4. An enabled module's active, visible, previously reconciled menu metadata
   remains present in both sidebar and search while its service is unavailable
   or incompatible; disabling it, hiding the menu, or removing the RBAC grant
   removes the affected entries.
5. Dictionary route/API/capability source is absent, stale built-in Dictionary
   permissions become inactive on sync, current manual overrides and custom
   menus remain active, and the historical table is not dropped.
6. Menu management shows the same navigation inventory as the sidebar and page
   search. Core Admin entries are read-only, module-owned presentation is
   editable, no permission-definition switch or create-permission action is
   exposed, and role management remains the permission-assignment surface.
7. Frontend format, lint, typecheck, and production build pass; all current
   authenticated routes render at 1920x1080 without horizontal overflow;
   representative routes pass at 1440x900; Dashboard preserves readable light
   and dark surfaces; the authenticated console has no warning or error.

The accepted global profile does not authorize additional visual or product
scope. Any new surface still requires its own scoped contract.
