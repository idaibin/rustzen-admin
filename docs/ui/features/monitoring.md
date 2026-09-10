# Monitoring UI

## Authority and scope

Current Feature UI contract for Overview, Nodes (including Global Settings), Incidents, and
Daily Summaries. Product basis: [Monitoring](../../product/features/monitoring/spec.md).
The user's current console requirements and repository-owned surfaces define the
scope; root [DESIGN.md](../../../DESIGN.md) alone defines shared visual semantics.
The repository-owned AppShell supplies the separated sidebar, header, and content
panels. Ant Design supplies controls; ProLayout does not own application geometry.
There is no checks page or desktop sidebar-collapse control.

This contract replaces outdated V3/V3.2/V3.3 candidate instructions for these routes.
It does not introduce generated assets, Agent configuration, custom checks, or
weekly/monthly/yearly reports. Component facts are source-extracted; runtime
coverage belongs to [local verification](../../guides/local-verification.md).

The Linux runtime browser gate opens the rendered Global Settings and Node
policy Drawers through a container-only route-exact proxy. It separately injects
disconnect and HTTP failures for global save and node save/reset. Disconnects
must show the actionable retained-draft Drawer alert; HTTP rejections continue
through the existing single global message and must not duplicate that alert.
Save cases change a threshold before submission and verify the edited value is
still present after rejection. Reset cases verify the node remains marked as a
custom policy and that its reset action remains available after rejection.
The schema-2 browser manifest records a run and PNG hash/dimensions per case.

The two-Agent Linux runtime gate is an API/data acceptance seam for the Nodes UI: it
proves the gateway can expose two independently registered nodes with current boot IDs
and raw metric history after central recovery. It does not render the Nodes page, so it
does not complete this document's visual or responsive acceptance matrix.

## Route composition and behavior

The Incidents surface is labeled **告警事件 / Alert incidents** in navigation,
page search, its heading, accessible table name and loading/error/empty messages.
The route remains `/monitoring/incidents`.

| Surface | Composition | Interaction and pagination |
| --- | --- | --- |
| Overview | One page title, four shared MetricCards, latest-resource panels, DataState and refresh feedback | Registered, online, offline, active-incident counts in that order; missing resources are empty data, not healthy substitute values. It refreshes in the background every 30 seconds. |
| Nodes | PageCard, ProTable, status tags, resource progress, node-detail Drawer | Inventory has no pagination; on background failure it retains the last table and shows BackgroundRefreshNotice with Retry and explicit refresh. Drawer owns history, per-mount series, effective policy, override/reset and bounded scrolling. |
| Incidents | PageCard, two upper-right Select filters, filling table and separate bottom Pagination | Status/type changes query immediately and return to page one. The fixed-size page refreshes in the background every 30 seconds. Empty success retains total zero and disabled pagination. Details preserve list context. At 390px, Event, Status, and Details remain visible in that priority order; Kind and Last observed are hidden until `sm`. Event title and node/target metadata have a bounded readable column: they never wrap one character per line, and long values use a one-line accessible ellipsis. |
| Nodes / Global Settings drawer | One configuration Card with CPU, memory, disk and offline controls; one Save and last-update footer | Four controls share one Form; outlined numeric inputs remain visible on the panel. Management permission controls editing and Save. |
| Daily Summaries | PageCard, DataTableShell, ProTable, separate bottom Pagination and DataState | No search input. Browse per-node daily summaries using the existing fixed-size pagination, without fabricated zero-valued ranges. It refreshes in the background every 30 seconds. At the 390px narrow layout, Date, Node, and Coverage remain visible; Samples and the resource, offline, and incident detail columns are hidden until the `sm` breakpoint so rows do not collapse into vertical text or crop at the right edge. |

The reusable notification-delivery card appears above Incidents only in the
`full` and `monitor-notify` selections. It displays pending and quarantine
counts with their charged bytes, the sum of all five irreversible gap counters,
and formatted nullable delivery timestamps. Pure `monitor` has no card,
notification endpoint, or card test identifier in its generated or emitted Web
artifact.

At 390px the delivery card remains above the readable Incident table. The
filters wrap without overlap, the table and bottom pagination stay within the
content viewport, and the table retains its own bounded scroll behavior if a
future localized value needs more room. The visible Event header and first row
stay within normal compact table height, so the screenshot demonstrates readable
content rather than document-overflow absence alone.

## States and accessibility

- Loading, empty, populated, error/retry and permission behavior stay distinct on
  Overview, Nodes, Incidents and Daily Summaries. An initial read failure blocks
  the route with Retry because no data exists yet. A normal background failure
  preserves the last successful route data and shows BackgroundRefreshNotice with
  Retry; a 403 instead takes precedence over cached data and shows the permission
  state, so protected rows, metrics and totals are no longer rendered. Filters
  remain mounted and editable during loading and non-permission errors. Route
  reads have automatic retries disabled; the visible Reload or Retry control
  explicitly starts the next request after an initial or background failure.
- Incident filters sit beside the title at the content upper right. At constrained
  widths they wrap and remain right-aligned without overlapping the title. Shared
  heading, description, empty-state colors and focus rules come from DESIGN.
- The incident table fills the available content height; only its body scrolls
  vertically. Pagination stays below that body. The empty-state content stays in the
  visible table viewport even when columns can scroll horizontally.
- Settings preserve the product's control order, value ranges and required validation.
  Missing values show readable field names. Read-only users see disabled controls and
  no Save. Saving does not submit duplicate requests while pending. A network
  failure during global save, node-policy save, or reset is visible inside the
  open Drawer and retains the draft/current policy state. HTTP and business
  failures use the request layer's one existing toast and do not produce a
  duplicate drawer toast. If management permission is withdrawn while a Drawer
  is open, its failure/retry state is cleared and no save or reset handler may
  submit a mutation.
- Drawers retain Ant Design focus trapping, restoration and close behavior. Status
  includes text; keyboard actions, filters, retry and Save have accessible names.
- Parent grids reflow at narrow widths. No document-level horizontal overflow or
  duplicated page title is allowed; table overflow remains bounded.

## Acceptance mapping

| ID | Required outcome | Verification seam |
| --- | --- | --- |
| MON-UI-001 | Exactly four current routes; AppShell and Ant Design ownership preserved | Route inventory and navigation |
| MON-UI-002 | Incident filters at upper right, immediate changes, page-one request, stable controls | Filter interaction and loading/error state |
| MON-UI-003 | Filling incident body and bottom pagination including empty data | Desktop and narrow geometry |
| MON-UI-004 | One global alert configuration panel with four controls and one Save | Form composition, validation and permission states |
| MON-UI-005 | Daily Summaries has no search and retains paging/retention meaning | Route controls and query scope |
| MON-UI-006 | Shared tone cards and visible empty state in both themes | Theme and viewport comparison |
| MON-UI-007 | At 390px, Incidents retains Event, Status and Details; title and metadata remain readable without character-by-character wrapping | Mobile geometry: visible-column count, bounded primary header/row, table and pagination right edges |

## Validation scope

| Viewport | Required check |
| --- | --- |
| 1705×1039, light Chinese | Overview, incident filters/empty/footer, unified settings and search-free summaries |
| 1024×768, light Chinese | Title/filter wrapping without overlap; usable content region |
| 390×844, light and representative dark | Settings reflow, bounded tables, visible empty state and pagination |
| 1440×900, dark English | Representative Drawer, localization and read-only settings regression |

The requirements are specified for implementation. Representative local browser
checks do not imply completion of this full matrix, all populated pagination cases,
all permission combinations, or deployment acceptance. See the verification owner
for observed coverage; this document is not a visual approval record.

The minimum local closure has exactly 23 route-exact fixture cases: twenty route
state cases (four routes times initial loading, initial permission, initial
server error, post-success 403, and post-success 500), then Incident page two,
Incident status/kind filtering at page one, and populated Daily Summaries page
two. A post-success 403 hides Overview metrics, Node rows, Incident rows and
Summary rows, while a post-success 500 retains each route's data and shows Retry.
One 1440×900 dark English and one 390×844 light Chinese capture must
also pass the no-document-overflow assertion.

## Nodes actions and drawers

At the upper right, show Global settings (secondary) and Add node (primary).
Keep actions mounted during node-list loading and error. Add node is manager-only;
Global settings is readable with monitor:node:view and editable with monitor:manage.
Remove the independent Alert Settings route, sidebar entry and page-search result.

Add node opens an informational Ant Design Drawer with ordered signed-install
prerequisites. Until the console can securely supply the release files and provision
the target-host root-only secret, it exposes no node/URL/token form, generated command,
copy action or connection action; its primary execution action remains disabled.

Global settings opens a Drawer with one four-control configuration form, one Save
and the last-update time. Closing either drawer returns to the node list and
discards unsaved form state. A failed network save keeps this Drawer and its
draft open with an inline retryable error. Drawers fit mobile width and preserve
keyboard focus.

Node details shows the current boot ID and the 5-minute CPU/memory history. When
history points exist, the chart uses a fixed-height inner plot area within the Card;
the responsive chart must not collapse because the Card body has content-driven height.
A one-bucket history shows visible markers for both series; longer histories may hide
point markers and use their lines.
## Node onboarding availability

The Nodes Drawer is informational until the Web product has a secure delivery path for
signed release files and a root-only secret provisioning boundary. It presents ordered
Ant Design steps for archive, manifest, envelope, trusted key, key ID, root-only config
format, and CLI phases. Its primary execution action is disabled. It contains no copied
shell block, token field, token value, or direct Agent command.

## Notification delivery health

In `full` and `monitor-notify`, the Incidents page places a compact delivery
card above its table. It shows pending and quarantine counts, the irreversible total
(omitted+expired+unconfirmed+quarantined+evicted), first/last gap and last
success. Zero gaps is healthy; a nonzero total is explicit. Loading, 403 and
500 are distinct and each recoverable state has Retry. No payload is rendered.

The Monitor Linux Chromium delivery-card extension has a local, checkout-bound
closure: it passes only when
`target/rz/monitoring-ui-state/current/manifest.json` matches the current
checkout and has `status: "passed"`. It records 23 canonical route runs, 18
owner and 26 viewer delivery steps, four screenshots, real Monitor SQLite and
authorized API receipts, permission behavior, and either zero retries or a
recorded single retry receipt. Pure Monitor absence remains covered by the
selected-Web composition gate; runtime error and forbidden card rendering
remains the component-test seam unless a bounded Monitor harness seam is added.
