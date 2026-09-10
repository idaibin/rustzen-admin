# Deployment Guide

This repository publishes Admin, Monitor, Insights, and Reports as one signed
version and one rollback boundary. It does not support service-specific
versions, module-only releases, dynamic activation, mixed releases, or an old
single-binary fallback.

## Local commands

```bash
cargo test --workspace -- --test-threads=1
just check
just build-native
just build
just verify-services
just verify-monitor-admin
just verify-reports-linux
git diff --check
```

`just verify-monitor-admin` certifies the local minimal Admin source/build
boundary. It does not assemble or install a selected distribution; selected Web,
systemd inventory and signed native packaging are separate later gates.

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
    ├── config/rz-reports.env
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
├── config/{rz.env,rz-reports.env}
├── data/db/{admin,monitor,insights}.db
├── data/releases/rz-<version>-<arch>.tar
└── data/reports/db/reports.db
```

`setup-layout.sh` is initial-install only. If `current` already exists it fails
closed; every upgrade must use the Admin release worker so database backups,
health gates, the rollback journal, and the single-release boundary cannot be
bypassed. The installer links six units into systemd, reloads the daemon, and
enables `rz.target` without starting placeholder production secrets. After
replacing every remaining placeholder in `config/rz.env` and
`config/rz-reports.env`, start the server set with:

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
`rz-monitor-agent`, and is not part of `rz.target`. Build it separately with
`cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent`.
The server bundle does not include this collector binary.

`just verify-monitor-agent-multi-node-linux` supplies a disposable Linux runtime gate
for two independently-run Agent identities and one central Admin/Monitor pair. It
proves startup remains unready while that central pair is absent, then verifies first
delivery and recovery after a central-process restart through the Admin API. It uses
real processes, Unix readiness sockets, separate Agent runtime/log directories and
service UIDs, but it does not boot systemd PID 1 or establish remote-host or production
TLS behavior. Those remain native deployment acceptance work.

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

The release environment template contains eight non-empty production values:
environment, runtime root, JWT secret, IPC token, Monitor Agent token, Monitor
node ID, bundle
signature enforcement, and the public verification key. Ports, database paths,
pool limits, logging, timezone, retention, and task timeout use code defaults
unless explicitly overridden. Do not add blank optional values; an absent
numeric override remains `None`, while explicit zero is parsed as zero.

The shared database pool defaults to one minimum and eight maximum connections.
The eight-connection maximum is the certified default for the four-CPU,
512MiB pure-Monitor runtime profile; operators may override it for a measured
deployment profile with `RUSTZEN_DB_MAX_CONN`.

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
  the local Admin agent-report endpoint

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

Manual deletion and expired-release cleanup remove the stored bundle before
soft-deleting its database row. A missing bundle is treated as already removed;
any other file-removal failure leaves the row visible so the operation can be
diagnosed and retried instead of creating an untracked orphan file.

## Runtime and performance evidence

`just verify-services` uses release controller/service binaries plus an
independently built `rz-monitor-agent` binary and covers Admin-only login with all
modules down, persisted Monitor Agent submission through Admin, all 24 service
startup orders, independent process termination, disabled/unavailable gateway
envelopes with surviving module requests, direct delegation rejection, all four
database corruption/restore boundaries, and the Manifest
service-restart/route-change/incompatible HTTP contract.

Before the 24 four-service startup orders, the verifier prepares its disposable
Monitor database with `init-db`, `bind-database`, and `validate-database` in
that order. The Monitor controller remains fail-closed and does not migrate the
database at process start. The public Just target runs its pinned Bun static
harness-contract test before its Rust checks, builds, and dynamic verifier.

The worker verifier also exercises Monitoring shared-capability navigation,
owner/viewer policy access, report fencing, alert/recovery transitions, pagination,
and daily/weekly Reports schedule lifecycle. `verify-services` defaults to release
binaries and accepts only `release` or `debug` in `RUSTZEN_VERIFY_BUILD_PROFILE`.
It pins Bun 1.3.14. Latency evidence records `buildProfile`, fixed `p95BudgetMs`,
`budgetEnforced`, and `budgetPassed`: release enforces the 2 ms p95 overhead
budget, while debug records the same measurement without turning that release
budget into a debug failure. Its default debug evidence path is
`target/rz/gateway-latency-debug.json`; release uses
`target/rz/gateway-latency.json`, and either may be overridden explicitly.
All functional assertions remain fail-closed. Neither profile is a
production-wide benchmark.

`just verify-automation-browser <browser-path>` verifies real form submission,
step audit, screenshot/live-frame output, cancellation, overlapping runs, overall
run timeout, and owned-profile cleanup using a fresh disposable database. It
requires the browser executable and `sqlite3` for the bounded timeout fixture.

`just verify-reports-linux` runs the x86_64 Reports binary in a Colima/Docker
Debian container as `rz-reports`. It verifies signed delegated requests, a real
Chromium screenshot, user namespaces, WAL files, recovery blocking, log
ownership, and cleanup. On an Apple Silicon host the amd64 container requires
an unconfined outer Docker profile for emulation, so this check reports browser
seccomp and real systemd as **Not verified**; validate both on a confined native
Linux system before production acceptance. The public Just target first runs its
pinned Bun static three-key configuration-contract test, then starts this
dynamic container gate. Its disposable Debian image installs CA roots because
the Reports production HTTP client fails closed unless the system trust store is
available.

The verifier injects an independent Reports notification event key of at least
32 bytes, with its own key identifier. It is distinct from both the IPC token
and Reports credential key so production configuration validation exercises the
three-key boundary. This gate does not verify notification delivery itself.

The 2026-07-15 same-host benchmark used protected
`GET /api/monitor/nodes`, concurrency 32, 128 warm-up requests and 320 measured
requests per path. Results in milliseconds:

| Path | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| Direct Monitor | 0.825 | 1.302 | 1.507 |
| Through Admin | 1.151 | 1.722 | 1.835 |
| Gateway overhead | 0.327 | 0.419 | 0.328 |

The p95 overhead is below the 2 ms investigation gate. With pinned Bun 1.3.14,
`just verify-modules-mvp` exited 0 and recorded its debug diagnostic at
`target/rz/gateway-latency-debug.json`: measured `2026-09-10T19:19:24.605Z`,
SHA-256 `5f0ecade6b3f870505eafd352457d40c1d5d1a465464317bb952825e506a387d`,
direct/gateway/overhead p95 0.500/3.387/2.886 ms,
`buildProfile=debug`, `p95BudgetMs=2`, `budgetEnforced=false`, and
`budgetPassed=false`. `just verify-services` exited 0 and recorded release at
`target/rz/gateway-latency.json`: measured `2026-09-10T19:23:19.587Z`, SHA-256
`fa760ff0612887c1db1b66560fdb3a09f3b24c78a2bd502c1f7bc45f3555252f`,
p95 0.403/2.317/1.914 ms, `buildProfile=release`, `p95BudgetMs=2`,
`budgetEnforced=true`, and `budgetPassed=true`. The release run overwrote the
latest machine-local JSON. These same-host values are machine-local evidence
only, not production-wide latency claims.

Reports runs as the dedicated unprivileged `rz-reports` user. The installer
creates that account and grants it only `data/reports` (including the database
and browser directories) plus `logs/reports`; `rz-reports.service` keeps Chromium sandboxing enabled and uses systemd
privilege restrictions. Do not run Reports browser execution as root or add
Chromium `--no-sandbox` to production configuration.
Linux hosts must permit unprivileged user namespaces for Chromium's user
namespace sandbox. A host that disallows them rejects the browser run; it must
not be relaxed by adding `--no-sandbox`.

Set `RUSTZEN_TIMEZONE` in `config/rz-reports.env` to the installation timezone;
Reports scheduling treats that value as authoritative. Keep it consistent with
`config/rz.env`, and keep the Reports `RUSTZEN_IPC_TOKEN` equal to the shared
service token so delegated requests remain verifiable.
