# UI Feature Index

Each UI feature below is independently loadable and references its product
basis. `DESIGN.md` is the only current shared visual-semantics authority;
feature specs record only route-local composition, states, and evidence.

## Feature inventory

| Product area | UI slice | Product basis | Status |
| --- | --- | --- | --- |
| Monitoring | [Monitoring surfaces](./features/monitoring.md) | [Product spec](../product/features/monitoring/spec.md) | Current route contract; representative local UI checks; complete state matrix pending |
| Analytics | [Collection safety](./features/analytics-collection-safety.md) | [Product spec](../product/features/analytics-collection-safety/spec.md) | Current contract; see scoped local verification |
| Automation | [Scheduled automation](./features/scheduled-report-automation.md) | [Product spec](../product/features/scheduled-report-automation/spec.md) | Implemented; source-resolved; runtime Not verified |
| Admin / runtime | [Module log diagnostics](./features/module-log-diagnostics.md) | [Product spec](../product/features/module-log-diagnostics/spec.md) | Implemented; source-resolved; runtime Not verified |
| Admin | [Metric-card route alignment](./features/metric-card-visual-consistency.md) | [Product spec](../product/features/metric-card-visual-consistency/spec.md) | Current route contract; representative light/dark checks |
| Admin | [Dashboard navigation](./features/dashboard-navigation-simplification.md) | [Product foundation](../product/product.md) | Current dashboard composition; representative local browser checks |
| Admin | [Role definition management](./features/role-definition-management.md) | [Product spec](../product/features/role-definition-management/spec.md) | Implemented; pre-merge evidence retained |
| Admin | [User role assignment readiness](./features/user-role-assignment-readiness.md) | [Product spec](../product/features/user-role-assignment-readiness/spec.md) | Implemented; pre-merge evidence retained |

## Current console route contract

Display names: **自动化 / Automation** for the `reports` module and
**告警事件 / Alert incidents** for `/monitoring/incidents`. Navigation, search,
module status and page states share these names; internal identifiers stay unchanged.

Product basis: [console interaction requirements](../product/product.md#console-interaction-requirements).
Shared placement, colors and responsive rules: root [DESIGN.md](../../DESIGN.md).
Existing list filters occupy the content upper right, with no Search/Reset submit
buttons; automatic input behavior follows the product contract. Loading/error states
retain the controls and focus. CRUD submission, password reset, node-policy reset,
refresh and destructive confirmations are distinct actions and remain available.

| Route | Current filters / local composition | Acceptance |
| --- | --- | --- |
| / | Four account cards; permission-gated resource panel and textual module health | CON-06; shared tone and panel rules |
| /system/user | Username and account status only; new-user action alongside filters | CON-01–03 |
| /system/role | Role name, role code and status; role-dialog controls remain separate | CON-01–02 |
| /system/menu | Menu name, permission code and status; read-only/core and editable/module rows retained | CON-01; only table body scrolls vertically |
| /manage/log | Action type and user/IP; export uses applied filters | CON-01–02 |
| /system/status | Module and date in the diagnostics heading; refresh/backup/cleanup retained; limits below heading | CON-01; short log list has no inner vertical scroll |
| /monitoring/incidents | State and type; filling body with bottom pagination | CON-01–02 |
| /monitoring/nodes | Add node onboarding and Global settings drawers; four global controls and one Save | CON-04 |
| /monitoring/summaries | No search control; existing report pagination | CON-03 |
| /analytics/overview | Activity metrics; no collection-policy status card | CON-05 |
| /analytics/details | Type and page/API path; other reports clear/disable path; bottom pagination | CON-01–02, CON-05 |

No new filters are implied for routes without search. Requirements and observed
runtime coverage are separate: see [local verification](../guides/local-verification.md)
for checks and remaining gaps. This index does not declare complete visual acceptance.

## Shared dependencies and exclusions

- Every slice depends on the product index, root `DESIGN.md`, and the source/API
  owner named by that slice. The slice document is not a replacement for those
  authorities.
- `DESIGN.md` owns shared component semantics, state vocabulary, theme, layout,
  responsive, and accessibility baselines. Do not copy those rules into this
  index or create a second component/token authority.
- The index does not define API schemas, generated clients, permissions, or
  runtime/browser/deployment evidence. Those remain in the product slice,
  source contracts, or validation records and are `Not verified` until tested.

## Consumer read order

Read repository guidance, the product index and target product spec, root
`DESIGN.md`, this UI index, then only the target UI slice. Source, route
registration, DTOs, and generated clients remain implementation authority.
Runtime, browser, deployment, and external-consumer claims are `Not verified`
until the owning validation task produces evidence.
