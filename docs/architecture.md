# Architecture

`rustzen-admin` is the source authority for the Rustzen Admin, Monitor,
Insights, and Reports runtime. It is a Web/Rust A-class monorepo that produces
four independent server binaries in one signed release bundle, with one
version and one rollback boundary.

The same bundle also contains `rz`, a non-resident operations CLI.
It is a fifth executable entry point, not a fifth service: it owns no database,
has no systemd unit, and does not change the four server failure domains.

## Ownership

Admin persists the unique capability catalog in `menus` and module navigation in
`module_navigation`. Navigation identity is `(module_id, module_menu_code)`;
its `code` references a capability name without defining another capability.
Manifest reconciliation updates both projections atomically. This permits
Monitoring Overview/Settings and Nodes/Summaries to share read permissions while
retaining independent navigation, visibility, and presentation overrides.

- `apps/admin/` owns the Admin API, authentication and RBAC persistence, the
  in-memory module registry and gateway, release management, Admin migrations,
  and embedded Web assets.
- `apps/monitor/` owns Monitor routes, behavior, migrations, and the optional
  managed-node Agent mode.
- `apps/insights/` owns Insights routes, behavior, and migrations.
- `apps/reports/` owns Reports routes, behavior, migrations, and output files.
- `apps/web/` owns the React UI and typed API clients.
- `crates/ipc/` owns shared module response and pagination contracts, health
  responses, Manifest types, module route registration, and HMAC delegation
  signing and verification.
- `crates/auth/` owns shared authentication types and capability policy.
- `crates/config/` owns focused per-application `RUSTZEN_*` parsing and runtime
  path defaults.
- `crates/storage/` owns shared SQLite connection and maintenance primitives.
- `crates/runtime/` owns stable runtime-layout helpers and the shared
  daily-file logging mechanism used by all four server applications. It does
  not own application lifecycle or process registration.
- `deploy/` owns the installer, target, recovery unit, four server units, and
  the separately installed Monitor Agent unit.

There is no runtime dependency on `rustzen-core` or `rz-core`, no registry or
service discovery, and no dynamic module or independently published module
version.

Each server keeps a thin logger adapter at its own source boundary and delegates
file rotation and retention to `crates/runtime/`: `apps/admin/src/infra/logger.rs`,
`apps/monitor/src/infra/logger.rs`, `apps/insights/src/infra/logger.rs`, and
`apps/reports/src/infra/logger.rs`. These four adapters own only the service
prefix, log directory, and cleanup message; they do not share application log
content or persistence.

## Product and module evolution

`docs/product/product.md` is the current product-decision authority within its
scope; source, this architecture, and current guides remain implementation
truth. Automation is an internal Reports feature namespace; it is not a
separately shipped module. Report Center is deferred and has no process,
database, route, or permission owner.

Shared-code promotion follows `docs/guides/shared-capabilities.md`. Stable
technical mechanisms may be shared after matching real consumers and tests
exist. Module models, calculations, statuses, SQL, migrations, schedules,
retention selection, and business failure semantics remain application-owned.
Adding a fifth server is a release-topology change, not an ordinary feature
addition.

## Runtime topology

| Application | Command | Default bind | Database |
| --- | --- | --- | --- |
| Admin | `rz-admin serve` | `0.0.0.0:9801` | `data/db/admin.db` |
| Monitor | `rz-monitor controller` | `127.0.0.1:9802` | `data/db/monitor.db` |
| Insights | `rz-insights serve` | `127.0.0.1:9803` | `data/db/insights.db` |
| Reports | `rz-reports serve` | `127.0.0.1:9804` | `data/reports/db/reports.db` |

`rz-monitor-agent` is an optional managed-node process. It reports to the
Monitor Controller and is intentionally not part of the server `rz-full.service`.
It collects a fixed CPU, memory, and per-mount disk payload every 30 seconds and
submits it to the Controller. It has no configurable check/task runtime, policy
engine, incident store, report engine, historical query owner, Outbox, or
configuration synchronization channel.

The Monitor Controller is the monitoring-data, policy, incident, and report
authority. It stores the latest state and up to 30 days of raw resource samples,
evaluates resource policies as samples arrive, and independently evaluates node
offline state when reports are missing. The terminal product behavior and
retention boundary are fixed in
`docs/product/features/monitoring/spec.md`; the Monitoring Rust protocol and
schemas implement that contract.
The terminal implementation structure, public and protected API, and executable
test matrix are defined in `docs/guides/monitoring-architecture.md`,
`docs/guides/monitoring-api.md`, and `docs/guides/monitoring-testing.md`.

Each server owns only its database and migrations. A module failure leaves
Admin login and the other module processes available. systemd restarts each
service independently.

Reports uses one final fresh-install migration baseline. Schedule tables belong
in that baseline with the rest of the Reports schema; Reports does not retain
sequential upgrade migrations or alternate-schema paths.

## Module contract and gateway

Monitor, Insights, and Reports each keep only module metadata and default menu
presentation in `module.toml`. Their Rust `ModuleRouter` calls are the single
source for HTTP method, route path, access mode, handler, and required
capability. The same registration builds the in-memory Axum router and the
runtime Manifest exposed at `GET /internal/v1/manifest`.

Admin uses fixed loopback endpoints and synchronizes enabled module Manifests
in the background. A valid change is reconciled transactionally into module
menu/capability rows, then swapped into an immutable in-memory registry. An
invalid or incompatible Manifest never partially updates database or runtime
state.

The request path performs one in-memory route lookup and one in-memory user
capability decision. Admin then streams the body through one reused
`reqwest::Client` and signs a request-scoped HMAC context bound to contract
version, timestamp, request ID, user ID, module, method, path, and the one
required capability. The module verifies that context and its local route
requirement before calling the handler. Admin does not query SQLite, read TOML,
fetch a Manifest, perform discovery, rebuild a client, send a full permission
set, or parse and re-serialize JSON on this hot path.

Public module routes still require Admin delegation at the service boundary.
Direct unsigned, expired, cross-module, or wrong-capability calls are rejected.

Insights accepts public tracking events only inside the configured retention
window and a five-minute future clock-skew allowance. HTTP status codes and
durations are bounded before they can affect overview or percentile metrics.
Its 30-request/300-event per-project/source admission windows live in the
running Insights process and reset on process restart; they are runtime
protection, not durable cross-restart hard maxima. Body, batch, storage-budget,
and free-disk caps remain hard fail-closed boundaries.

Reports rejects recognized secret fields in templates and run input, persists
only accepted non-sensitive input, and omits run input from API responses. A
running cancellation remains non-terminal while the browser step is being
aborted; the API exposes `cancelling` until the browser closes and Reports
durably records `cancelled`.

## Permissions and menus

The product navigation presents the stable internal services as grouped modules:

- Monitoring: overview, self-registering nodes, and service monitoring below `/monitoring`;
- Analytics: an instance-wide overview and raw details below `/analytics`, without a project
  selector or project-scoped queries;
- Reports: target-backed templates plus filling runs and live browser frames below `/reports`.

The internal service IDs, binaries, databases, and API prefixes remain
`monitor`, `insights`, and `reports`. Each page menu owns a concrete read
capability so Manifest reconciliation, Viewer grants, route guards, and frontend
authorization use the same boundary.

- `owner` receives `*` and is the only built-in role allowed to mutate
  releases.
- `admin` receives concrete Monitor, Insights, and Reports capabilities, but no
  owner-only system or management capabilities.
- `viewer` receives concrete read-only capabilities.

Admin persists mutable grants, module enabled state, menu overrides, and
reconciled module menu rows in SQLite. The permission cache is refreshed on
login and permission mutations; the gateway reads the cache only. Manual menu
overrides are preserved when a Manifest refreshes, and disabling a module
removes it from runtime navigation without deleting its stored overrides.
For enabled modules, the last reconciled active, visible menu rows remain in
navigation during service outages and incompatible runtime states; health and
contract compatibility affect request availability, not menu visibility.
Manual menu-visibility overrides remain effective.

## Release topology

All four server applications and the operations CLI use the workspace version.
`just build` creates and signs one uncompressed tar bundle:

```text
target/rz/rz-<version>-<arch>.tar
└── rz-<version>-<arch>/
    ├── bin/{rz,rz-admin,rz-monitor,rz-insights,rz-reports}
    ├── systemd/{rz-full.service,rz-recovery.service,rz-admin.service,
    │            rz-monitor.service,rz-insights.service,rz-reports.service}
    ├── config/{rz.env,rz-reports.env}
    └── setup-layout.sh
```

`just build-config` also emits `target/rz/rz-install`, a separately copied
installer with the trusted public verification key embedded by the build. The
initial-only installer accepts only the complete bundle path, preserves shared
configuration and data, generates local runtime secrets, and
installs an immutable release directory:

```text
/opt/rz/
├── current -> releases/<version>
├── releases/<version>/bin/{rz,rz-admin,rz-monitor,rz-insights,rz-reports}
├── config/rz.env
├── data/db/{admin,monitor,insights}.db
├── data/releases/rz-<version>-<arch>.tar
└── data/reports/db/reports.db
```

`rz-full.service` uses `Wants=` for recovery and the four server services. The four
services use `PartOf=rz-full.service`, independent restart/start-limit policies, and
no `Requires=` coupling. `rz-recovery.service` runs before them and blocks their
start if an interrupted update cannot be recovered.

Once `current` exists, the installer refuses to switch it. Upgrades run only
through the Admin update worker so no direct symlink change can create a mixed
release outside the backup, health-gate, journal, and rollback transaction.

The Admin update worker verifies the complete signed bundle and the installed
rollback release, creates consistent online backups of all four databases,
installs or verifies one release directory, atomically switches `current` once,
then restarts Monitor, Insights, Reports, and Admin through systemd-active and
release-version health gates. Rollback restores the previous link and only the
databases whose services entered the restart sequence. The durable journal and
boot recovery unit close process-crash and host-restart interruption windows.

Module-only activation, separate service versions, mixed releases, an old
single-binary fallback, and multiple active release links are not supported.

## Verification

The root `justfile` is the command authority. `just verify-services` uses
release binaries to test all 24 startup orders, independent termination, four
database corruption/restore boundaries, gateway and delegation contracts, and
the Manifest restart/change contract.

The same service verification reads each live module Manifest and checks every
method/path consumed by the Web API clients. A frontend route that is renamed,
removed, or assigned a different HTTP method therefore fails verification
without introducing another backend route source of truth.

The 2026-07-15 same-host gateway comparison used
`GET /api/monitor/nodes`, release binaries, concurrency 32, 128 warm-up
requests and 320 measured requests per path. Direct p50/p95/p99 were
0.825/1.302/1.507 ms; gateway p50/p95/p99 were 1.151/1.722/1.835 ms; the
corresponding overhead was 0.327/0.419/0.328 ms. The p95 overhead passed the
2 ms investigation gate.

## Admin route contract

The Admin contract is code-first and covers all 47 statically enumerable
Admin-owned operations: the public login operation, authenticated and
capability-gated Admin routes, and the four ModuleControlState routes. Module
service routes (Monitor, Insights, and Reports) remain excluded from the public
OpenAPI document: `ModuleRouter` currently emits only method, path, access, and
capability metadata, while the gateway resolves dynamic paths through its
in-memory registry. Adding operation IDs, request/response schemas, and grounded
business errors there requires extending that shared contract seam; a second
hand-maintained OpenAPI catalog would violate route authority.

`apps/admin/src/infra/contract.rs` registers the Axum method router and records
normalized method, final path, operation ID and access policy in one call. The
outer JWT middleware remains owned by `infra/app.rs`; the contract records this
boundary rather than duplicating JWT enforcement. `Public`, `Authenticated`,
`Require`, `Any`, and `All` map to OpenAPI security and, for capability policies,
`x-rustzen-authorization`.

`just contract-generate` derives `openapi/admin-contract.json`; Orval derives
the Web client. Generated JSON calls use the `generatedApiRequest` mutator,
while binary responses use `generatedBlobRequest`; both preserve the existing
token and error semantics, and feature APIs remain the only page-facing
callers. `just contract-verify`, `contract-client`,
`contract-baseline`, and `contract-bench` provide focused checks. The fixed
`contract-admin-current.json` baseline detects generated-contract drift. `contract-bench` is
an isolated release-mode microbenchmark over the 47-operation registration set;
it alternates baseline-first and contract-first samples and reports separate
prebuilt hot-router probes for Public, Authenticated, and Require paths. It is
not an end-to-end or production latency claim.
