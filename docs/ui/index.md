# UI Feature Index

Each UI feature below is independently loadable and references its product
basis. `DESIGN.md` is the only current shared visual-semantics authority;
feature specs record only route-local composition, states, and evidence.

## Feature inventory

| Product area | UI slice | Product basis | Status |
| --- | --- | --- | --- |
| Monitoring | [Incident diagnostics](./features/monitor-incident-diagnostics.md) | [Product spec](../product/features/monitor-incident-diagnostics/spec.md) | Implemented; source-resolved; runtime Not verified |
| Analytics | [Collection safety](./features/analytics-collection-safety.md) | [Product spec](../product/features/analytics-collection-safety/spec.md) | Implemented; source-resolved; runtime Not verified |
| Reports | [Scheduled automation](./features/scheduled-report-automation.md) | [Product spec](../product/features/scheduled-report-automation/spec.md) | Implemented; source-resolved; runtime Not verified |
| Admin / runtime | [Module log diagnostics](./features/module-log-diagnostics.md) | [Product spec](../product/features/module-log-diagnostics/spec.md) | Implemented; source-resolved; runtime Not verified |
| Admin | [Metric-card consistency](./features/metric-card-visual-consistency.md) | [Product spec](../product/features/metric-card-visual-consistency/spec.md) | Implemented; retained runtime evidence is historical to that slice |
| Admin | [Dashboard navigation](./features/dashboard-navigation-simplification.md) | [Product foundation](../product/product.md) | Implemented; retained runtime evidence is historical to that slice |
| Admin | [Role definition management](./features/role-definition-management.md) | [Product spec](../product/features/role-definition-management/spec.md) | Implemented; pre-merge evidence retained |
| Admin | [User role assignment readiness](./features/user-role-assignment-readiness.md) | [Product spec](../product/features/user-role-assignment-readiness/spec.md) | Implemented; pre-merge evidence retained |

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
