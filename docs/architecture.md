# Architecture

`rustzen-admin` is the source authority for the RustZen Admin, Monitor,
Insights, and Reports runtime. It is a Web/Rust A-class monorepo that produces
four independent server binaries in one signed release bundle, with one
version and one rollback boundary.

The same bundle also contains `rz`, a non-resident, read-only operations CLI.
It is a fifth executable entry point, not a fifth service: it owns no database,
has no systemd unit, and does not change the four server failure domains.

## Ownership

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
- `crates/runtime/` owns stable runtime-layout helpers and the compatible
  daily-file logging mechanism used by Admin, Monitor, and Reports. It does not
  own application lifecycle or process registration.
- `deploy/` owns the installer, target, recovery unit, four server units, and
  the separately installed Monitor Agent unit.

There is no runtime dependency on `rustzen-core` or `rz-core`, no registry or
service discovery, and no dynamic module or independently published module
version.

## Product and module evolution

`docs/product/product.md` is the current product-decision authority within its
scope; source, this architecture, and current guides remain implementation
truth. Automation is an internal Reports feature namespace; it is not a
separately shipped module. Report Center is deferred and has no process,
database, route, or permission owner.

Former `rustzen-inspect`, `rustzen-analytics`, and `rustzen-report` repositories
are live behavior references, not current implementation owners. A module
expansion must use the fixed comparison in
`docs/reference/legacy-module-comparison.md`, select capabilities individually,
and implement them inside the current owner. Old authentication, RBAC, system
management, deployment, runtime layout, and Web shell code must not be copied
into a module because Admin and the current release bundle already own those
contracts.

Shared-code promotion follows `docs/guides/shared-capabilities.md`. Stable
technical mechanisms may be shared after compatible real consumers and tests
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
| Reports | `rz-reports serve` | `127.0.0.1:9804` | `data/db/reports.db` |

`rz-monitor agent` is an optional managed-node process. It reports to the
Monitor Controller and is intentionally not part of the server `rz.target`.

Each server owns only its database and migrations. A module failure leaves
Admin login and the other module processes available. systemd restarts each
service independently.

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

## Release topology

All four server applications and the operations CLI use the workspace version.
`just build` creates and signs one uncompressed tar bundle:

```text
target/rz/rz-<version>-<arch>.tar
└── rz-<version>-<arch>/
    ├── bin/{rz,rz-admin,rz-monitor,rz-insights,rz-reports}
    ├── systemd/{rz.target,rz-recovery.service,rz-admin.service,
    │            rz-monitor.service,rz-insights.service,rz-reports.service}
    ├── config/rz.env
    └── setup-layout.sh
```

The initial-only installer verifies the complete bundle signature with a
separately supplied trusted public key, preserves shared configuration and data,
and installs an immutable release directory:

```text
/opt/rz/
├── current -> releases/<version>
├── releases/<version>/bin/{rz,rz-admin,rz-monitor,rz-insights,rz-reports}
├── config/rz.env
├── data/db/{admin,monitor,insights,reports}.db
├── data/releases/rz-<version>-<arch>.tar
└── data/reports/
```

`rz.target` uses `Wants=` for recovery and the four server services. The four
services use `PartOf=rz.target`, independent restart/start-limit policies, and
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
