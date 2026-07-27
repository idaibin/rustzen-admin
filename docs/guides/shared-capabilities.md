# Shared Capabilities and Module Intake

This guide defines when code stays local, moves to a module-owned shared layer,
or becomes a repository-wide contract. The governing rule is:

> Share stable technical mechanisms; keep business meaning with its owner.

## Ownership levels

| Level | Default owner | Promotion gate |
| --- | --- | --- |
| Feature-local | The route or feature directory. | Default for one behavior or one caller. |
| Module-shared | The owning application. | At least two compatible callers in the same module. |
| Repository-shared | A named `crates/` or frontend responsibility. | At least two real cross-boundary consumers, compatible semantics, and an independent test. |
| New shared crate | A stable business-neutral contract. | Multiple applications or crates, no application dependency, and a complete public test surface. |

Security, authentication, delegation, file safety, and protocol consistency may
cross the boundary with two consumers because divergent implementations create
an immediate correctness risk. Similar names or similar code shape are not an
extraction reason.

## Current canonical owners

### Rust

| Capability | Canonical owner | Representative consumers | Boundary |
| --- | --- | --- | --- |
| JWT, auth context, permission checks, capability constants | `crates/auth/` | Admin and all module route declarations | Business-neutral auth policy only. |
| Health response, Manifest, module route registration, HMAC delegation | `crates/ipc/` | Admin gateway plus Monitor, Insights, and Reports | Stable Admin-to-module protocol. |
| Focused process configuration | `crates/config/` | Four server applications | Each process parses only its own settings. |
| Runtime paths | `crates/runtime/` | Admin and deployment-aware code | Stable layout primitives, not application workflows. |
| SQLite pool and maintenance | `crates/storage/` | Four application databases | Connection and safe maintenance mechanics only. |

Application business types, SQL, handlers, services, migrations, schedules,
retention selection, and status lifecycles remain under the owning `apps/*`
tree.

### Frontend

| Product or visual job | Canonical owner | Representative consumers | Reuse rule |
| --- | --- | --- | --- |
| Page hierarchy | `PageHeader`, `PageCard` under `apps/web/src/components/page/` | Dashboard, status, list, and management routes | Reuse; extend the owner only for a product-wide semantic variant. |
| Operational metric | `MetricCard` in `apps/web/src/components/page/metric-card.tsx` | Monitoring and Analytics overviews | Reuse with factual values; do not add module-specific copies. |
| Loading, empty, error, permission, processing | `DataState` in `apps/web/src/components/feedback/data-state.tsx` | Query-backed pages and lists | Reuse; the owning query supplies retry behavior. |
| Table surface and paging | `DataTableShell` under `apps/web/src/components/table/` + route-local `ProTable` | System, management, and module lists | Reuse; keep columns and filters local until semantics repeat. |
| Confirmation | `ConfirmDialog` in `apps/web/src/components/feedback/confirm-dialog.tsx` | Destructive management actions | Reuse; business copy remains local. |
| Date-time display | `formatDateTime` in `apps/web/src/lib/format-date-time.ts` | Management and system tables with nullable values | Reuse only when empty values map to `-` and locale display is intended. |
| HTTP transport | `apiRequest`, `apiUpload`, `apiDownload` in `apps/web/src/api/request.ts` | All domain API modules | Reuse; URLs and transport shaping stay in the owning API module. |

`DESIGN.md` owns shared visual and component semantics. This guide owns
implementation placement and extraction gates. The historical `docs/ui/`
package must not be treated as a current accepted owner.

## New-module intake

Every new module or major expansion follows this sequence:

1. Fix the current `rustzen-admin` basis and the live default-branch revision of
   each former standalone repository used as evidence.
2. Compare user-visible behavior, business rules, failure states, permissions,
   persistence, and tests. Do not compare directory names alone.
3. Classify each capability as `retain`, `reproduce`, `reuse`, `extend`, `wrap`,
   `new`, `defer`, or `drop`.
4. Remove former repository platform duplication from the proposal. Admin owns
   auth, RBAC, release, module control, and the Web shell.
5. Search the canonical shared owners before declaring a helper, component,
   contract, crate, or API.
6. Implement one independently verifiable vertical slice in the owning
   application and database.
7. Promote a mechanism only when current consumers prove the same semantics.
8. Update source, tests, commands, architecture, project map, product facts,
   API clients, UI specification, and deployment assets in the same change when
   their boundaries move.

Former repositories are behavior references. Copying a source file is allowed
only after the capability is selected, its dependencies are removed or mapped,
its current owner is declared, and its tests are adapted to the current public
contract. Historical review documents never establish current reusability.

## Extraction decisions

- `reuse`: consume the current owner without changing its contract.
- `extend`: add a compatible variant to the current owner and preserve existing
  consumers.
- `wrap`: adapt a module-specific boundary without cloning the shared
  mechanism.
- `new`: create only after documenting the closest candidates and why they do
  not fit.
- `reproduce`: reimplement selected former-product behavior in the current
  owner; do not preserve its old deployment or application shell.
- `defer` or `drop`: keep the behavior out of the implementation slice.

## Rejected abstractions

- Catch-all `common`, `utils`, `helpers`, or `shared` business directories.
- One global enum for Monitor, Analytics, Reports, run, schedule, artifact, and
  alert status.
- A generic repository, CRUD service, schema-driven form builder, dashboard
  builder, or workflow engine created for possible future modules.
- A shared type filled with optional fields for unrelated module models.
- A module importing another module's service, repo, migration, or database.
- A second route catalog beside Rust `ModuleRouter` registration.

## Review checklist

Every new shared declaration must answer:

1. Who owns the contract and where is it exported or registered?
2. Which current consumers use it, and do they require the same semantics?
3. Does the shared layer remain independent of application business code?
4. Can the contract be tested without a specific application workflow?
5. Does extraction remove synchronized duplication rather than add generic
   configuration?
6. What is the closest current implementation, and why is the decision
   `reuse`, `extend`, `wrap`, or justified `new`?
7. Which docs, commands, API consumers, migrations, and UI mappings must move
   with the owner?
