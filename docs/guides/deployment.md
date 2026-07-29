# Deployment Guide

This repository publishes Admin, Monitor, Insights, and Reports as one signed
version and one rollback boundary. It does not support service-specific
versions, module-only releases, dynamic activation, mixed releases, or an old
single-binary fallback.

## Local commands

```bash
cargo test --workspace
just check
just build-native
just build
just verify-services
git diff --check
```

`just build-native` builds the Web application, four optimized server binaries,
and the non-resident `rz` operations CLI for the current machine. `just build`
uses Docker to build the same five Linux musl executables, assembles one bundle,
signs the complete tar, and verifies the signature. The Web package manager is
Bun 1.3.14 and `apps/web/bun.lock` is the only frontend lockfile.

## Bundle contract

The artifact is an uncompressed signed tar:

```text
target/rz/rz-<version>-<x86_64|aarch64>.tar
└── rz-<version>-<arch>/
    ├── bin/
    │   ├── rz
    │   ├── rz-admin
    │   ├── rz-monitor
    │   ├── rz-insights
    │   └── rz-reports
    ├── systemd/
    │   ├── rz.target
    │   ├── rz-recovery.service
    │   ├── rz-admin.service
    │   ├── rz-monitor.service
    │   ├── rz-insights.service
    │   └── rz-reports.service
    ├── config/rz.env
    └── setup-layout.sh
```

All five ELF executables must match the declared architecture and workspace
version. Each embeds `RUSTZEN_RELEASE_MARKER` with
`artifact=rz-bundle-member` and its exact binary name. The appended Ed25519
signature covers the complete tar payload as component `bundle`.

```bash
bun scripts/deploy-sign.mjs sign-bundle \
  --file target/rz/rz-<version>-<arch>.tar \
  --version <version> \
  --arch <arch>

bun scripts/deploy-sign.mjs verify-bundle \
  --file target/rz/rz-<version>-<arch>.tar \
  --version <version> \
  --arch <arch>
```

The signing command uses the established private key sources and fails closed
when no key is available. It does not create or rotate a signing key.

## Installation and systemd

Obtain the release verification key through the trusted release channel, then
run `setup-layout.sh` with that key and the signed bundle:

```bash
RUSTZEN_DEPLOY_VERIFY_KEY=<trusted-ed25519-public-key> \
  ./setup-layout.sh rz-<version>-<arch>.tar
```

The installer validates the complete Ed25519 signature and exact safe member
set before installing executable content. It stores the signed bundle
byte-for-byte, installs an immutable release directory, and atomically creates
one relative link:

```text
/opt/rz/
├── current -> releases/<version>
├── releases/<version>/bin/{rz,rz-admin,rz-monitor,rz-insights,rz-reports}
├── config/rz.env
├── data/db/{admin,monitor,insights,reports}.db
├── data/releases/rz-<version>-<arch>.tar
└── data/reports/
```

`setup-layout.sh` is initial-install only. If `current` already exists it fails
closed; every upgrade must use the Admin release worker so database backups,
health gates, the rollback journal, and the single-release boundary cannot be
bypassed. The installer links six units into systemd, reloads the daemon, and
enables `rz.target` without starting placeholder production secrets. After
setting the seven required production values in `config/rz.env`, start the
server set with:

```bash
systemctl enable --now rz.target
systemctl status rz.target
systemctl restart rz-monitor.service
```

`rz.target` uses `Wants=` for recovery and all four services. Every server unit
uses `PartOf=rz.target`, `Restart=on-failure`, and an independent start-limit
policy. There is no `Requires=` coupling. `rz-recovery.service` runs before the
four services and leaves `data/recovery-blocked` in place if interrupted-update
recovery fails.

`rz-monitor-agent.service` is installed only on managed nodes, runs
`rz-monitor agent`, and is not part of `rz.target`.

`rz` has no systemd unit and is not a service alias. It is upgraded and rolled
back only with the same signed release and `current` link as the four servers.
Operators can invoke `/opt/rz/current/bin/rz` directly or add that directory to
their managed shell `PATH`. Its current contract is read-only:

```bash
rz --json doctor
rz --json version
rz --json status all
rz --json status admin
```

`status` performs only bounded loopback `/health` reads. `doctor` reports
installation paths, binary presence, and the same health summary. Its config
reader accepts only the internal host and four port keys; credential values
are neither retained nor emitted.

## Configuration

The release environment template contains seven non-empty production values:
environment, runtime root, JWT secret, IPC token, Monitor Agent token, bundle
signature enforcement, and the public verification key. Ports, database paths,
pool limits, logging, timezone, retention, and task timeout use code defaults
unless explicitly overridden. Do not add blank optional values; an absent
numeric override remains `None`, while explicit zero is parsed as zero.

Supported optional overrides are:

- `RUSTZEN_ADMIN_HOST`, `RUSTZEN_ADMIN_PORT`
- `RUSTZEN_INTERNAL_HOST`, `RUSTZEN_MONITOR_PORT`, `RUSTZEN_INSIGHTS_PORT`,
  `RUSTZEN_REPORTS_PORT`
- `RUSTZEN_ADMIN_SQLITE_PATH`, `RUSTZEN_MONITOR_SQLITE_PATH`,
  `RUSTZEN_INSIGHTS_SQLITE_PATH`, `RUSTZEN_REPORTS_SQLITE_PATH`
- `RUSTZEN_DB_MAX_CONN`, `RUSTZEN_DB_MIN_CONN`, `RUSTZEN_DB_CONN_TIMEOUT`,
  `RUSTZEN_DB_IDLE_TIMEOUT`
- `RUSTZEN_JWT_EXPIRATION`, `RUSTZEN_TIMEZONE`,
  `RUSTZEN_TASK_RUN_TIMEOUT_SECONDS`
- `RUSTZEN_MONITOR_CONTROLLER_URL` for a remote Monitor Agent; its default is
  the local Admin heartbeat endpoint

## Apply, recovery, and rollback

Only `owner` may view or mutate releases. Admin, Viewer, and custom roles cannot
receive deployment capabilities.

Apply runs the fixed transient `rz-update.service` outside the Admin service
cgroup. The worker:

1. revalidates the candidate signed bundle and current installed rollback
   bundle;
2. creates consistent online `VACUUM INTO` backups plus a hash manifest for all
   four databases;
3. installs or byte-validates one immutable release directory;
4. durably records the old link, candidate, backup, staging, and restarted
   units;
5. atomically switches `/opt/rz/current` once;
6. restarts Monitor, Insights, Reports, then Admin, requiring both systemd
   active state and an exact release-version health response;
7. marks the database release current only after all gates pass.

A normal gate failure restores the previous link and only the databases whose
services entered the restart sequence, then verifies those old-version
services. A reused historical release directory is retained; a failed directory
created by the current update is removed.

On host restart, `rz-recovery.service` validates the old signed installed
release, restores the journaled link and databases without starting services
inside the recovery transaction, removes the journal, then requeues the four
services. A durable sentinel prevents them from starting if recovery or
requeue fails.

## Runtime and performance evidence

`just verify-services` uses release binaries and covers Admin-only login with all
modules down, persisted Monitor Agent submission through Admin, all 24 service
startup orders, independent process termination, disabled/unavailable gateway
envelopes with surviving module requests, direct delegation rejection, all four
database corruption/restore boundaries, and the Manifest
service-restart/route-change/incompatible HTTP contract.

The 2026-07-15 same-host benchmark used protected
`GET /api/monitor/nodes`, concurrency 32, 128 warm-up requests and 320 measured
requests per path. Results in milliseconds:

| Path | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| Direct Monitor | 0.825 | 1.302 | 1.507 |
| Through Admin | 1.151 | 1.722 | 1.835 |
| Gateway overhead | 0.327 | 0.419 | 0.328 |

The p95 overhead is below the 2 ms investigation gate. The latest machine-local
JSON evidence is written to `target/rz/gateway-latency.json` by the verification
target.
