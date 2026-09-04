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

## Route composition and behavior

The Incidents surface is labeled **告警事件 / Alert incidents** in navigation,
page search, its heading, accessible table name and loading/error/empty messages.
The route remains `/monitoring/incidents`.

| Surface | Composition | Interaction and pagination |
| --- | --- | --- |
| Overview | One page title, four shared MetricCards, latest-resource panels, DataState and refresh feedback | Registered, online, offline, active-incident counts in that order; missing resources are empty data, not healthy substitute values. |
| Nodes | PageCard, ProTable, status tags, resource progress, node-detail Drawer | Inventory has no pagination; Drawer owns history, per-mount series, effective policy, override/reset and bounded scrolling. |
| Incidents | PageCard, two upper-right Select filters, filling table and separate bottom Pagination | Status/type changes query immediately and return to page one. Empty success retains total zero and disabled pagination. Details preserve list context. |
| Nodes / Global Settings drawer | One configuration Card with CPU, memory, disk and offline controls; one Save and last-update footer | Four controls share one Form; outlined numeric inputs remain visible on the panel. Management permission controls editing and Save. |
| Daily Summaries | PageCard, DataTableShell, ProTable and DataState | No search input. Browse per-node daily summaries using the existing fixed-size pagination, without fabricated zero-valued ranges. |

## States and accessibility

- Loading, empty, populated, error/retry and permission behavior stay distinct.
  Filters remain mounted and editable during loading and errors. Background refresh
  errors preserve the last successful Overview and Incident data when available.
- Incident filters sit beside the title at the content upper right. At constrained
  widths they wrap and remain right-aligned without overlapping the title. Shared
  heading, description, empty-state colors and focus rules come from DESIGN.
- The incident table fills the available content height; only its body scrolls
  vertically. Pagination stays below that body. The empty-state content stays in the
  visible table viewport even when columns can scroll horizontally.
- Settings preserve the product's control order, value ranges and required validation.
  Missing values show readable field names. Read-only users see disabled controls and
  no Save. Saving does not submit duplicate requests while pending.
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
discards unsaved form state. Drawers fit mobile width and preserve keyboard focus.
## Node onboarding availability

The Nodes Drawer is informational until the Web product has a secure delivery path for
signed release files and a root-only secret provisioning boundary. It presents ordered
Ant Design steps for archive, manifest, envelope, trusted key, key ID, root-only config
format, and CLI phases. Its primary execution action is disabled. It contains no copied
shell block, token field, token value, or direct Agent command.
