# Full distribution architecture

Status: current.

Rustzen ships one complete signed server distribution. The release contains Admin,
Monitor, Insights, Reports, the Web application and the non-resident `rz` operations
CLI. Runtime module enablement and authorization may hide or deny an installed
capability, but they do not change the signed file inventory.

## Runtime topology

`rz-full.service` is the aggregate systemd unit used by the installer and public
`rz start`, `rz stop`, `rz restart` and `rz status` commands. It groups four
independent resident services:

- `rz-admin.service` owns authentication, RBAC, the management API, the module
  gateway, notifications and the embedded Web application;
- `rz-monitor.service` owns monitoring state and Controller APIs;
- `rz-insights.service` owns collection and Analytics queries;
- `rz-reports.service` owns report definitions, runs and scheduling.

The aggregate unit does not merge process or failure boundaries. `rz status` checks
all four resident units and fails if any child is inactive. Starting or restarting
waits for all four units to become active before reporting success.

The managed-node Agent remains a separate non-Web artifact because it runs on a
monitored node rather than the server. It is not a module-only server release and
does not create a second Admin/Monitor distribution.

## Source and data ownership

Each resident service owns its routes, configuration, SQLite database, migrations,
logs and health endpoint. Services never read another service's database. Shared
wire contracts and delegated-request validation live in `crates/ipc`; shared
authentication policy lives in `crates/auth`.

Admin discovers enabled module manifests and exposes authorized gateway routes. An
installed module may be disabled at runtime without removing its binary, Web files or
database from the signed release. An enabled module with an unavailable service stays
visible to an authorized user and returns an explicit unavailable state.

## Notifications

Notifications are an Admin-owned durable inbox, not a fifth resident service.
Monitor and Reports commit module-owned outbox events with their business state, then
relay them to Admin over loopback-only authenticated ingress. Admin validates the
producer, deduplicates the event, resolves currently authorized recipients and stores
the message and recipient projection transactionally.

SSE is advisory invalidation. The browser reconnects and reloads authoritative inbox
state; URLs never contain JWTs. Session expiry and user revocation close existing
streams. A live signal is not proof of durable or human delivery.

## Build and release identity

`just build` builds the Web application, four Linux server executables and `rz`, then
assembles one tar bundle under `target/rz/`. One Ed25519 signature covers the complete
archive. The package records frontend and backend digests for inspection, while the
signature and release version remain unified.

The signing private key exists only in the build environment. The packaged installer
contains the public verification material required for the matching official bundle;
the operator does not supply a key during installation. Modified, unsigned or
differently signed bundles fail before installation writes.

## Installation and first start

The operator copies the generated bundle and `rz-install` to the server and runs:

```bash
sudo ./rz-install ./rz-<version>-<arch>.tar
sudo rz start
rz status
```

The installer accepts the bundle path, verifies it, creates the immutable release
directory, generates internal service secrets, installs the six systemd units and
switches the single `current` link. It does not request an administrator password,
signing key, verification key or database command.

Admin, Monitor, Insights and Reports create and validate their own SQLite databases
when started against an empty runtime root. Database initialization and identity
checks are internal startup work, not operator-facing lifecycle steps.

Admin, Monitor, Insights and Reports run under dedicated non-root service identities
with module-specific writable paths. Admin submits a fixed update request containing
only the release ID and actor; `rz-update.path` starts the root-only
`rz-update.service`, which revalidates that request and the signed bundle before any
privileged release operation. Installer state, release directories, unit links,
recovery state and the `current` link remain root-owned.

## Updates and recovery

An authorized owner uploads a complete signed candidate through the Admin release API.
The deployment worker verifies the candidate, snapshots all four databases, installs
the immutable release, switches the single `current` link, restarts the service set
and verifies exact-version health for all four services. A failed verification,
restart or health gate restores the previous link and database snapshots. The
recovery unit completes or rolls back interrupted transactions after a host restart.

There are no service-specific versions, mixed-release installations, physical
module-only server archives or alternate selected Web shells.

## Failure isolation and acceptance

A module failure must not corrupt another module's database or turn into silent
success from the aggregate CLI. Admin remains the entry host and reports unavailable
module state when a downstream service is offline. Recovery is bounded and retains
the last known healthy release until the candidate passes every gate.

Source tests do not prove the bundle; bundle verification does not prove systemd;
target-like systemd evidence does not prove an external production deployment. The
required layers and current status are defined in [validation](validation.md) and the
[local verification record](../../../guides/local-verification.md).
