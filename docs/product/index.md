# Product Feature Index

Read [product.md](./product.md) for product positioning, shared principles,
module boundaries, and direction. Read only the target feature specification
below for slice-local behavior and acceptance. The independent
`project-delivery-system` repository owns cross-project templates and review
records; this repository keeps only the current product contract and its
implementation handoff.

## Shared implementation contract

### Actors and capabilities

The role model remains `owner`, `admin`, `viewer`, and custom roles. The
capability catalog is generated from Admin route registration and module
Manifests; no page or menu creates a capability by itself.

| Actor or boundary | Read behavior | Write or privileged behavior |
| --- | --- | --- |
| Owner | All four module and Admin read surfaces. | All target management actions, including module-log backup and cleanup. |
| Admin | Existing concrete Monitor, Insights, and Reports read capabilities; no module-log view, backup, or cleanup. | Slice-specific management only when the corresponding `*:manage` capability is granted; all module-log operations remain owner-only. |
| Viewer | Concrete read capabilities only; no module-log view, backup, or cleanup. | No acknowledgement, schedule mutation, collection-policy mutation, log export, or cleanup. |
| Custom role | Only explicitly granted concrete capabilities; no implicit module-log access. | Module-log view, backup, and cleanup are not grantable to custom roles in this slice. |
| Public tracker | No authenticated Admin capability. | Host bootstrap enforces visitor opt-in before injection; HTTP ingestion validates only the installation policy: `collection_enabled`, `project`, and normalized `origin`. |

Target capability boundaries are deliberately narrow:

- Monitor incident diagnostics: `monitor:incident:view`.
- Analytics query: existing `insights:overview:view` and
  `insights:event:view`; collection policy remains `insights:manage`.
- Scheduled Reports: `reports:schedule:view` and
  `reports:schedule:manage`.
- Module logs: `system:module:log:view`,
  `system:module:log:backup`, and `system:module:log:cleanup`; view, backup,
  and cleanup are owner-only.

These names are a product permission boundary. Their runtime registration and
Manifest output remain implementation responsibilities of the owning service;
Monitor, Insights, and Reports use their handwritten contracts plus worker
verification, while the Admin module-log route uses OpenAPI and Orval.

### Shared state vocabulary

Every query-backed surface distinguishes `loading`, `populated`, `empty`,
`error`, and `permission`. Long-running actions add `processing` and then an
explicit `success` or `failure` result. A successful zero-row response is the
only valid `empty` state; request failure must never be rendered as empty.

`partial` is reserved for a bounded operation where some named items succeed
and others fail. The UI keeps successful items, identifies failed items and
their reason, and offers review or retry. It is not a generic success state and
must not hide an incomplete backup, cleanup, or multi-target report result.

The owner-only module-log slice is a route/menu/API exception to the generic
permission-state vocabulary: non-owners never enter its diagnostics surface,
and no local permission state is rendered.

### Ownership and data boundaries

| Product slice | Business/data owner | Control-plane/UI owner | Contract authority |
| --- | --- | --- | --- |
| Incident diagnostics | Monitor incidents and Monitor SQLite | Web Monitoring routes | Rust `ModuleRouter/Manifest -> handwritten apps/web/src/api/monitor/contract.ts -> scripts/verify-worker-contracts.mjs` |
| Collection safety | Insights tracking, settings, and Insights SQLite | Web Analytics overview/details; public tracker is not an Admin page | Rust `ModuleRouter/Manifest -> handwritten apps/web/src/api/insights/contract.ts -> scripts/verify-worker-contracts.mjs` |
| Scheduled Reports | Reports schedules, flows, runs, and Reports SQLite | Web Reports templates/runs | Rust `ModuleRouter/Manifest -> handwritten apps/web/src/api/reports/contract.ts -> scripts/verify-worker-contracts.mjs` |
| Module logs | Each service emits its own daily file; Admin authorizes and audits access | Web System Status | Admin `ContractRouter -> OpenAPI -> Orval` plus the fixed runtime log allowlist |

No module reads another module's database. The shared runtime logger owns file
format and retention mechanics; each service remains responsible for its own
log content and lifecycle.

## Slice inventory and status

| Product area | Feature slice | Product specification | UI specification | Status |
| --- | --- | --- | --- | --- |
| Monitoring | Incident diagnostics | [monitor-incident-diagnostics](./features/monitor-incident-diagnostics/spec.md) | [monitor-incident-diagnostics UI](../ui/features/monitor-incident-diagnostics.md) | Implemented; source-resolved; runtime Not verified |
| Analytics | Collection safety | [analytics-collection-safety](./features/analytics-collection-safety/spec.md) | [analytics-collection-safety UI](../ui/features/analytics-collection-safety.md) | Implemented; source-resolved; runtime Not verified |
| Reports | Scheduled automation | [scheduled-report-automation](./features/scheduled-report-automation/spec.md) | [scheduled-report-automation UI](../ui/features/scheduled-report-automation.md) | Implemented; source-resolved; runtime Not verified |
| Admin / runtime | Module log diagnostics, backup, and cleanup | [module-log-diagnostics](./features/module-log-diagnostics/spec.md) | [module-log-diagnostics UI](../ui/features/module-log-diagnostics.md) | Implemented; source-resolved; runtime Not verified |
| Admin | Role definition management | [role-definition-management](./features/role-definition-management/spec.md) | [role-definition-management UI](../ui/features/role-definition-management.md) | Ready |
| Admin | User role assignment readiness | [user-role-assignment-readiness](./features/user-role-assignment-readiness/spec.md) | [user-role-assignment-readiness UI](../ui/features/user-role-assignment-readiness.md) | Ready |

Each row is independently loadable. Implementing one slice does not require
loading a sibling specification. The root `DESIGN.md` and the shared state and
permission contract above are the only shared prerequisites.

## Consumer read contract

An implementation consumer reads, in order: repository guidance; this index
and the target product slice; the root `DESIGN.md`; the UI index; and the target
UI slice. Source, route registration, DTOs, and generated clients remain the
current implementation authority. Runtime, browser, deployment, and external
consumer evidence stay `Not verified` until exercised by the validation owner.
