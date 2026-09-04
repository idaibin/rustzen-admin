# Project Map

This is the authoritative repository navigation index. Source code, manifests,
the root `justfile`, and the nearest `AGENTS.md` remain proof.

- Scope class: versioned Web/Rust monorepo.
- Map and Git root: the repository root.
- `DESIGN.md` is the single shared visual-semantics entry point. Current
  route-local UI contracts are indexed by `docs/ui/index.md`; legacy YAML
  sidecars in that directory are historical pointers only.

## Authority and reading paths

| Question | Read in order |
| --- | --- |
| Product positioning, direction, or module purpose | `docs/product/product.md` → delivered source behavior |
| Runtime topology or repository ownership | source code → `docs/architecture.md` → the nearest guide |
| Frontend page or interaction change | target route → owning `apps/web/src/api/` module → shared component owner → matching backend route |
| Shared UI semantics or visual tokens | `DESIGN.md` → live definitions and consumers → `apps/web/src/styles/theme.css` |
| Build, check, or runtime verification | the matching root `justfile` recipe → scripts invoked by that recipe |

## Root

| Path | Value | Inspect when |
| --- | --- | --- |
| `README.md` | Human entrypoint and project summary. | You need a quick repository overview. |
| `AGENTS.md` | Repository-wide contribution constraints. | You start any task. |
| `Cargo.toml` | Workspace members and authoritative release version. | You change packages, dependencies, or versions. |
| `justfile` | Command source of truth. | You run, check, build, package, or verify. |
| `.env.example` | Minimal production configuration template. | You change runtime configuration or deployment. |

## Commands

| Task | Authoritative command |
| --- | --- |
| Start Admin, Monitor, Insights, Reports, or Web | `just dev-server`, `just dev-monitor`, `just dev-insights`, `just dev-reports`, `just dev-web` |
| Format, lint, typecheck, build, clippy, and test | `just check` |
| Exercise the four-service module contract | `just verify-modules-mvp` |
| Exercise real Reports browser filling | `just verify-automation-browser <browser-path>` |
| Build the native release (four servers plus `rz`) | `just build-native` |
| Verify the read-only operations CLI | `just verify-cli` |
| Build the signed Linux release bundle | `just build` |

## Shared crates

| Path | Value | Inspect when |
| --- | --- | --- |
| `crates/auth/` | JWT/auth context, permission checks, and capability constants. | You change authentication or authorization policy. |
| `crates/ipc/` | Shared module response and pagination, health responses, Manifest, method-aware module router, and HMAC delegation contract. | You change a compatible module HTTP envelope, service health, module routes, or the Admin-to-module boundary. |
| `crates/config/` | Focused Admin, Monitor, Insights, and Reports configuration. | You change `RUSTZEN_*` parsing, defaults, or runtime paths. |
| `crates/storage/` | SQLite pool and maintenance primitives. | You change shared SQLite behavior. |
| `crates/runtime/` | Stable runtime-layout and compatible process-logging helpers. | You change runtime-root topology or shared daily-file logging mechanics. |

## Admin

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/admin/AGENTS.md` | Admin-specific backend rules. | You work below `apps/admin/`. |
| `apps/admin/src/infra/` | Admin startup, DB, auth cache, logging, config, and embedded Web serving. | You change Admin runtime wiring. |
| `apps/admin/src/features/modules/` | Fixed module state, Manifest sync, immutable registry, gateway, and module APIs. | You change module availability, sync, navigation, or forwarding. |
| `apps/admin/src/features/manage/deploy/` | Signed bundle validation, update worker, rollback, and recovery. | You change release behavior. |
| `apps/admin/src/features/auth/` | Login, logout, and current-session bootstrap. | You change sessions or token flows. |
| `apps/admin/src/features/account/` | Current-account profile, avatar, and password flows. | You change self-service account behavior. |
| `apps/admin/src/features/dashboard/` | Account totals and module-health landing data. | You change dashboard cards or module health display. |
| `apps/admin/src/features/manage/` | Logs, scheduled tasks, and deploy versions. | You change Admin management features. |
| `apps/admin/src/features/system/` | Menu, role, user, and status management. | You change RBAC or system administration. |
| `apps/admin/migrations/sqlite/` | Admin, RBAC, module-state, and release migrations. | You change Admin schema. |

## Operations CLI

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/cli/` | Non-resident `rz` command, stable JSON envelopes, allowlisted loopback health reads, and CLI tests. | You change operator-facing read commands or output contracts. |

## Module applications

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/monitor/src/features/` | Agent reports, node resources, metrics, alerts, incidents, settings, and daily summaries. | You change Monitoring behavior. |
| `apps/monitor/migrations/` | Monitor-owned schema. | You change Monitor persistence. |
| `apps/monitor/module.toml` | Monitor metadata and default menu only. | You change module presentation metadata. |
| `apps/insights/src/features/` | Single-project tracking, instance-wide overview/details, and retention settings. | You change Analytics behavior. |
| `apps/insights/migrations/` | Insights-owned schema. | You change Insights persistence. |
| `apps/insights/module.toml` | Insights metadata and default menu only. | You change module presentation metadata. |
| `apps/reports/src/features/automation/` | Report targets, templates, browser filling runs, artifacts, live frames, and retention. | You change Reports behavior. |
| `apps/reports/migrations/` | Reports-owned schema. | You change Reports persistence. |
| `apps/reports/module.toml` | Reports metadata and page menus. | You change module presentation metadata. |

Module route method, path, access mode, capability, and handler are declared in
Rust route registration, not in `module.toml`.

## Frontend

| Path | Value | Inspect when |
| --- | --- | --- |
| `apps/web/AGENTS.md` | Frontend-specific rules. | You work below `apps/web/`. |
| `apps/web/package.json` and `apps/web/bun.lock` | Bun package and dependency lock. | You change frontend dependencies or scripts. |
| `apps/web/src/routes/` | File-based pages and route guards. | You add or change pages. |
| `apps/web/src/api/` | Typed API modules and request wrapper. | You change HTTP contracts or data access. |
| `apps/web/src/components/layout/` | Admin shell and runtime navigation. | You change menus, header, or layout. |
| `apps/web/src/store/` | Shared frontend state. | You change auth or persisted cross-page state. |

The frontend uses React 19, TanStack Router, React Query, Zustand, Tailwind CSS,
and Ant Design (antd) + @ant-design/pro-components as the only UI library.
Tailwind is reserved for layout spacing and composition utilities,
not component behavior.
`apps/web/package.json` pins Bun 1.3.14, Vite 8.1.3, and Vite+ 0.2.4;
`apps/web/bun.lock` is its only lockfile.

### Reuse entry points

| Product or design term | Canonical owner | Representative consumers | Reuse boundary |
| --- | --- | --- | --- |
| Authenticated application shell | `apps/web/src/components/layout/index.tsx` | `apps/web/src/routes/__root.tsx` | Adapt the existing route, permission, locale, theme, and account owner; do not create another shell. |
| Page heading and actions | `apps/web/src/components/page/page-header.tsx` (`PageHeader`) | dashboard, profile, system status | Reuse for overview/detail surfaces; list and management pages use `PageCard`. |
| List and management page surface | `apps/web/src/components/page/page-card.tsx` (`PageCard`) | Monitoring, Analytics, Reports, Admin lists | Reuse title, toolbar, action, and content hierarchy without nesting a second page title. |
| Query state vocabulary | `apps/web/src/components/feedback/data-state.tsx` (`DataState`) | Monitoring, Analytics, Reports, Admin tables | Reuse loading, empty, error, permission, and processing states; retry calls the owning query. |
| Operational metric | `apps/web/src/components/page/metric-card.tsx` (`MetricCard`) | Dashboard, Monitoring, and Analytics overviews | Reuse for factual compact metrics; do not turn it into decorative KPI cards. |
| Table surface and pagination | `apps/web/src/components/table/`, `@ant-design/pro-components` | module and Admin list routes | Reuse `DataTableShell` and route-local `ProTable`; keep columns and page-local actions with the route and Pro components pagination behavior. |

For shared visual work, start with `DESIGN.md`, then revalidate the live definition
and at least one current consumer. `docs/ui/features/` contains subordinate current
route-local contracts; legacy `docs/ui/*.yaml` sidecars are historical pointers only.

## Common task routes

| Task | Shortest evidence chain | Verification entry |
| --- | --- | --- |
| Change a frontend page | route → domain API module → shared UI owner → backend route when the contract changes | `just build-web`, then the focused runtime or browser check |
| Change a module HTTP contract | Rust `ModuleRouter` registration → handler/types → `apps/web/src/api/<module>/contract.ts` → route consumer | `just verify-services` |
| Change authentication or permission behavior | `crates/auth/` → Admin auth/system feature → `apps/web/src/routes/__root.tsx` and auth store | focused tests, then `just check` |
| Change release or deployment behavior | Admin deploy feature → bundle/signing script → installer/systemd assets → architecture contract | focused script tests, then `just check` |
| Change product scope or module purpose | `docs/product/product.md` → affected source and acceptance evidence | documentation checks plus the implementation slice's own gate |
| Add or expand a module capability | `docs/product/product.md` → `docs/reference/legacy-module-comparison.md` → `docs/guides/shared-capabilities.md` → owning feature and route registration | focused tests, then `just verify-modules-mvp` |
| Add a shared Rust capability | `docs/guides/shared-capabilities.md` → closest named crate → exports → two representative consumers | focused tests, then `just check` |
| Add or change shared UI | target UI feature contract → `DESIGN.md` anchor → current component/token owner → representative routes | frontend checks and browser verification |

OpenAPI is not a universal task gate. Use the repository-native route and client
authority unless an existing generated pipeline or an explicit task introduces one.

## Verified reuse index

| Capability or product job | Canonical definition and access | Representative consumers | Boundary and evidence |
| --- | --- | --- | --- |
| Module route and Manifest | `ModuleRouter`, `Require` in `crates/ipc/src/router.rs`, exported by `crates/ipc/src/lib.rs` | Monitor nodes/metrics/incidents; Insights overview/query; Reports automation | Reuse for module routes; Rust registration is the only method/path/access/capability source. Tested in `crates/ipc`. |
| Service health | `HealthResponse` in `crates/ipc/src/health.rs` | Four service health producers; Admin deploy health gate | Reuse for the fixed release health contract; module-specific diagnostics stay local. |
| Authentication and capabilities | `crates/auth/src/` exports | Admin permission checks and three module routers | Reuse policy and constants; business authorization decisions remain with the owner. |
| SQLite connection and maintenance | `crates/storage/src/{sqlite,maintenance}.rs`, exported by `crates/storage/src/lib.rs` | Four databases and retention jobs | Reuse mechanics; schemas, SQL, and retention selection stay application-owned. |
| HTTP transport | `apiRequest`, `apiUpload`, `apiDownload` in `apps/web/src/api/request.ts` | Admin and module API packages | Reuse transport; keep URLs, DTOs, and shaping in the domain API module. |
| Page hierarchy | `PageHeader`, `PageCard` in `apps/web/src/components/page/` | Dashboard/status and list/management routes | Reuse the semantics in `DESIGN.md#layout-and-density` and verify live consumers. |
| Query feedback and table surface | `DataState`, `DataTableShell` | System, management, and module lists | Reuse states and surface; keep filters and columns route-local until semantics repeat. |
| Operational metrics | `MetricCard` in `apps/web/src/components/page/metric-card.tsx` | Dashboard, Monitoring, and Analytics overviews | Reuse factual metrics; do not create module copies. |
| Nullable locale date-time | `formatDateTime` in `apps/web/src/lib/format-date-time.ts` | Management and system tables | Reuse only for the same `null`/empty to `-` and locale-display contract. |
| Module API envelope and page | `ApiResponse<T>`, `Page<T>`, and bounded pagination in `crates/ipc/src/response.rs` | Monitor, Insights, and Reports HTTP boundaries | Reuse only for the shared `{code,message,data}` and `{data,total,success}` wire contracts; Admin's historical top-level `total` stays local. |

Before a new declaration, search the named owner, its export or registration,
and representative consumers. Decide `reuse`, `extend`, `wrap`, justified
`new`, or `Not verified`; a map miss is not proof that no implementation exists.

## Former product references

`docs/reference/legacy-module-comparison.md` fixes the current comparison basis
for `rustzen-inspect`, `rustzen-analytics`, and `rustzen-report`. Those
repositories are behavior references only. They are not current owners or
package dependencies. New module work must select capabilities row by row and
must not copy their duplicate Admin, auth, RBAC, deployment, or Web-shell code.

## Deployment and verification

| Path | Value | Inspect when |
| --- | --- | --- |
| `Dockerfile` | Five-executable Linux release build: four servers plus `rz`. | You change release compilation. |
| `scripts/package-release-bundle.sh` | Exact bundle assembly and marker checks. | You change bundle membership. |
| `scripts/deploy-sign.mjs` | Complete-bundle signing and verification. | You change signature behavior. |
| `deploy/setup-layout.sh` | Signature-verifying initial installer and first atomic `current` link. | You change installation. |
| `deploy/rz.target` and `deploy/rz-*.service` | Recovery, four server units, and separate Monitor Agent unit. | You change systemd topology. |
| `scripts/verify-services.sh` | Four-service runtime, isolation, DB restore, and latency verification. | You change runtime contracts or acceptance gates. |
| `scripts/test-setup-layout.sh` | Signature, initial-install boundary, and unit-topology integration tests. | You change bundle or install layout. |

## Documentation

| Path | Value | Inspect when |
| --- | --- | --- |
| `docs/README.md` | Documentation index. | You choose current guidance. |
| `docs/architecture.md` | Current repository and runtime facts. | You need architecture or data flow. |
| `docs/product/product.md` | Product positioning, current boundary, module purposes, non-goals, and deferred slices. | You make or verify a product-boundary decision. |
| `docs/guides/` | Current development rules. | You edit backend, frontend, permission, or deployment behavior. |
| `docs/guides/shared-capabilities.md` | Shared-code ownership and new-module intake gate. | You consider copying, extracting, or creating a shared declaration. |
| `docs/reference/` | Optional deeper current context. | Current facts and guides are not enough. |
| `docs/reference/legacy-module-comparison.md` | Fixed live-source comparison with former standalone products. | You decide which former behaviors to retain, reproduce, defer, or drop. |
| `docs/history/` | Non-current plans and records. | You need historical rationale. |
| `DESIGN.md` | Shared visual semantics and implementation naming anchors. | You change product-wide visual semantics, shared components, or themes. |
| `docs/ui/index.md` and `docs/ui/features/` | Current subordinate route-local UI contracts; not a shared visual authority. | You need page composition, applicable states, or feature acceptance. |
| `docs/ui/*.yaml` | Historical superseded pointers; not current contracts. | You need to identify the retired UI-standardization task. |

## Commands

Run recipes from the repository root and inspect their current bodies in the
`justfile` before use.

| Need | Command |
| --- | --- |
| Start Admin, module services, or Web | `just dev-server`, `just dev-monitor`, `just dev-insights`, `just dev-reports`, `just dev-web` |
| Build only the Web application | `just build-web` |
| Run the complete repository check | `just check` |
| Verify module topology, Manifests, gateway, and client route contracts | `just verify-services` |
| Verify the debug module MVP runtime | `just verify-modules-mvp` |

## High-risk and not-verified boundaries

- Adding a fifth server changes the signed bundle, target units, health gates,
  database backup/restore transaction, and rollback boundary.
- Multi-project Analytics changes event identity, permissions, navigation, and
  migration semantics.
- Reports credentials, datasets, uploads, DSL expansion, suspend/resume, and
  scheduling each require a separate product and failure-state specification.
- Former repository runtime behavior is not verified by this map; re-run its
  current tests and the selected `rustzen-admin` acceptance path before reuse.
