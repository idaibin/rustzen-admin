# Full distribution implementation plan

Status: active.

The first release has one implementation path and one acceptance boundary:

```text
build complete bundle -> copy to server -> install -> start -> four services healthy
```

## Optional examples stages

Current stage status is owned by [OE tasks](tasks.md); the dated receipts below
retain their original evidence boundaries. Follow [target contract](spec.md#optional-examples-target)
and [OE acceptance](validation.md#optional-examples-acceptance). Resume from the
current branch and verified task basis, preserving independent work and owner boundaries.

| Stage | Bounded task / owner | Exit required before next stage |
| --- | --- | --- |
| OE-0 | ZCode adopts these documents and records baseline/queue receipt | Confirm HEAD, clean/owned diff, prior commit retained, current release artifact availability; receipt lists adopted files and unresolved decisions; no repeat of language or security PR work |
| OE-1 | ZCode resolves two-composition dependency and compatibility contract | Source map for Cargo/Web/config/schema/CLI/package/sign/install/update; keep/adapt/retire existing selected paths; actual previous full artifact/worker identified; signed inventory encoding and legacy transition fixed; requirement/UI/code/test mapping complete |
| OE-2 | Core source and local runtime vertical slice | Core builds and starts alone with fresh Admin DB, real login/RBAC/audit UI; no module network calls, routes, assets, configs or workers; explicit full still builds/runs; English/Chinese preferences retained; OE-A01–05 pass |
| OE-3 | One build/sign/package/install chain for both choices | Real target binaries, exact signed member inventory, separate output paths, host prerequisites and generated config; clean disposable Linux installations; CLI/health follow installed members; OE-A06–08 pass |
| OE-4 | Compatibility, update and crash recovery | Actual preceding full artifact upgrades through proven compatible path; core-to-core and full-to-full update/rollback; interruption recovery preserves identities/data; composition mismatch rejected before writes; OE-A09–12 pass |
| OE-5 | Integrated closure and independent review | Fixed commit + immutable artifacts, complete OE evidence table, both docs/commands aligned, no unresolved release blocker; default flips to core only after core installation/update gates; build/release-ready verdict with unverified target rows explicit |

Do not couple unrelated fixes, rename existing deployment roots, rewrite all feature
layers or add arbitrary combinations. If OE-2 needs a default switch before packaging
closes, use an explicit development/test command; keep shipped `just build` behavior
unchanged until OE-5. Validate at each stage, rather than deferring real install and
legacy compatibility risks until the final stage.

### Single-writer handoff and change queue

After the coordinator hands over this initial documentation diff, ZCode is the sole
writer of the authoritative plan/tasks and implementation. Dot writes only a temporary
change queue at a path named in the handoff (suggested
`/tmp/rustzen-optional-examples-changes.md`). The queue is not project truth and must
not become a second specification. Each entry has an ID, request, acceptance impact,
priority and status. At every stage boundary ZCode reads it, records applied/deferred/
blocked plus rationale and resulting evidence in this plan, and acknowledges the IDs.
Only a destructive incompatibility, conflicting writer or invalid trust boundary
requires immediate interruption; ordinary refinements wait for the boundary.

Hourly coordinator checks inspect the current stage, fixed SHA/diff, tests, blocker
and queue receipts; continue the next bounded stage rather than restarting completed
work. Do not infer success from elapsed time or an idle task. Stop scheduling work
once OE-5 is independently closed; report a genuine external blocker when present.
Scheduling and sending the initial task belong to the coordinator, not this document.

Allowed execution: source edits, appropriate tests, local builds and signed packages,
disposable owned test installations, local commits and independent review. No remote
push, real-host deployment, public release, production migration, version bump or
main merge is implied. Signing material must stay in existing approved local trust
handling; missing keys/toolchain/old artifacts are reported rather than fabricated.

### First ZCode task (ready for coordinator dispatch)

Read AGENTS.md, docs/product/product.md and this OE plan, then spec.md, the root
architecture OE mapping, validation.md and the existing dashboard/navigation UI
slice. Inspect the current branch/diff before editing. Preserve local commit 2bbd0c6
and adopt only the coordinator-listed planning files. Complete OE-0/OE-1 first:
record the actual dependency inventory, current selected-code drift, previous full
artifact and verifier constraints, and the smallest compatible core/full contract.
Read and acknowledge the named temporary queue. Return exact touched files, fixed
baseline, proposed next bounded OE-2 task, checks and blockers. Do not redesign auth,
repeat security/language work, create a duplicate branch, push, merge or deploy.

## Scope

The full bundle contains the Web application, `rz`, Admin, Monitor, Insights,
Reports, production configuration templates, the full systemd units and the installer.
The explicit core bundle contains core Web assets, `rz`, Admin, Admin-only runtime
configuration, core systemd units and the installer. Both use one version, one Ed25519
signature, one update transaction and one rollback boundary per archive. Frontend and
backend digests are inspection fields inside that signed identity.

The production build entry points are `just build-release` (full, unchanged default)
and `just build-release-core` (explicit core). Docker uses a validated `COMPOSITION`
build argument to select the matching Web and Rust feature sets; an unknown value is
rejected. The packaged CLI composition is compile-time fixed: status, start/restart
health, doctor service probes, expected health count and binary inventory share that
single authority. Core archives contain only `rz` and `rz-admin` plus the core unit and
configuration inventory; task-only signing material remains outside the source/build
context.

The installation command accepts only the signed bundle path. It does not accept an
administrator password, signing key, verification key or database command. The build
places the public verification key into the packaged installer and runtime
configuration; the private signing key remains in the build environment. On a fresh
root, the installer generates internal service secrets, installs the immutable release,
links and enables the units, and leaves ordinary runtime overrides editable.

Admin, Monitor, Insights and Reports each create and validate their own fresh SQLite
database when they start. Monitor's migration and schema validation remain internal
implementation steps of `rz-monitor controller`; they are not separate operator work.

Updates are initiated by an authorized owner through the Admin release API. The worker
verifies one complete candidate, snapshots all four databases, installs the candidate,
switches the single `current` link, restarts the four services and checks their exact
release health. Any failed gate restores the previous link and database snapshots.
Interrupted work is handled by the existing recovery unit.

## Active work

| Order | Work | Completion condition |
| --- | --- | --- |
| 1 | Package trust and configuration | A freshly built installer verifies the matching bundle without external key input and rejects a bundle signed by another key; generated runtime configuration contains no placeholder secrets. |
| 2 | Fresh database startup | Starting each service against an empty runtime root creates and validates its database; Monitor requires no preceding CLI command. |
| 3 | Fresh system installation | In a disposable Linux systemd environment, one install command followed by `rz start` produces four healthy services on the same release version. |
| 4 | Restart recovery | Restarting the target and the container/host preserves the installed release and returns all four services to healthy state without operator repair. |
| 5 | API update and rollback | An owner can upload and apply one complete release; a deliberately failing candidate restores the previous release and all affected databases, with the outcome visible in deployment state. |
| 6 | Documentation closure | README, product contract, architecture, deployment guide and validation commands describe the same path and contain no manual key or Monitor database instructions. |

## Ownership and execution

One implementation owner changes the installer, service startup and focused tests.
The coordinator integrates the result, builds one immutable candidate and runs the
target-like installation, restart and rollback gates. One independent reviewer checks
the fixed result. Existing unrelated working-tree changes are preserved. Commit, push
and deployment to an operator-owned host remain separate actions.

## Ready verdict

Ready for the implementation and validation slices above. Release readiness is reached
only when every row passes against one immutable signed candidate. External production
deployment remains a separate environment-owner action.

## OE execution log

Authoritative per-stage record. Evidence anchors use full commit SHAs plus the
dirty-diff SHA-256 when the tree is not clean.

### OE-0 receipt (2026-10-09)

- Baseline fixed: `2bbd0c62f7743522b209fd60fd3f1ad0010b9faf` on
  `feat/theme-verification-20261005`; PR #7 security work already merged at
  `7124543`; template-first/English-default slice already complete at `2bbd0c6`.
  Neither is repeated.
- Adopted planning diff (six files, dirty SHA-256
  `6da2ad0846061076444faba296463cd0ba53247840abd1e254014ef0a766aa78`):
  `docs/product/product.md`, `docs/architecture.md`,
  `docs/product/features/composable-distribution/{spec,implementation,validation}.md`,
  `docs/ui/features/dashboard-navigation-simplification.md`. This document and the
  sibling files are committed together with this record as the adopted planning
  baseline; ZCode is the sole writer of authoritative plans and implementation
  from this point.
- Queue receipt: `/tmp/rustzen-optional-examples-changes.md` does not exist at
  OE-0. No queue entries to acknowledge; the boundary check repeats at every
  stage boundary.
- Parallel occupation: one git worktree (the repository root); no other
  worktrees, no running dev servers on 9801-9805.
- Current release artifacts available locally (see OE-1 legacy evidence).

### OE-1 dependency and compatibility record (2026-10-09)

All facts verified against the fixed baseline by direct source/artifact
inspection.

**Cargo composition surface.**

- `apps/admin/Cargo.toml`: `default = ["full"]`;
  `full = ["reports-notifications", "rustzen-config/admin", dep:croner,
  dep:ed25519-dalek, dep:hex, dep:image, dep:libc, dep:mime_guess,
  dep:rust-embed, dep:sysinfo, dep:tar, tower-http/fs]`;
  `notifications = [dep:aes-gcm, dep:base64, dep:fs2, dep:hex,
  rustzen-config/notifications]`;
  `reports-notifications = ["notifications", rustzen-config/reports-notifications]`.
- `crates/config/Cargo.toml` features: `admin`, `admin-monitor`,
  `admin-insights`, `notifications`, `reports-notifications`, `insights`,
  `monitor-controller`, `monitor-agent`, `reports`. Admin module endpoints
  default to loopback 9802-9804 (`crates/config/src/admin.rs:191,196,332`).
- Existing cfg gates are from the removed selected-build era and are
  inconsistent for a core build: notification ingress, maintenance and the
  realtime hub are already `#[cfg(feature = "notifications")]`
  (`apps/admin/src/infra/app/runtime.rs:1,26,45-66,100,157-161`,
  `infra/app/routes.rs:1,58`, `features/mod.rs:6`), but audit-capable login and
  several user/account surfaces are gated on `full`
  (`features/auth/service.rs:1,8,16,23,227,258`,
  `features/system/user/{types,repo,service}.rs`, `features/account/handler.rs`,
  `middleware/mod.rs:1`) while module synchronization runs unconditionally
  (`infra/app/runtime.rs:97,131` — `ModuleControlState::initialize` and
  `ModuleService::spawn_synchronizer`). `ModuleSpec::fixed()` hardcodes the
  three modules (`features/modules/types.rs:19-25`) and is consumed by the
  registry, module service and system status
  (`features/modules/registry.rs:88`, `features/modules/service.rs:41`,
  `features/system/status/service.rs:45`).
- `cargo check -p rustzen-admin --no-default-features` at the baseline fails
  with 60 errors (27 `libc`, missing `tar`/`hex`/`sysinfo`/`ServeDir`, missing
  `WebAssets::get`, `ServiceError::PayloadTooLarge`, `LoginAuditCommand`,
  `rustzen_config::AdminConfig`): the selected-era gating is incomplete and
  cannot serve as the core build today.

**Web composition surface** (see also the dashboard/navigation slice):

- Route tree: TanStack file-based routing with committed
  `apps/web/src/routeTree.gen.ts`; the single navigation seam is
  `apps/web/src/components/layout/routes.tsx` (`AppRoutePath`,
  `moduleGroupPaths`, `registeredModuleRoutePaths` whitelist at lines 181-190,
  `getMenuData`/`getSearchRouteItems`), with the module navigation polled from
  `systemAPI.module.navigation` in `components/layout/index.tsx:58-79`.
- Notifications: one import site in `apps/web/src/routes/__root.tsx:11,81`
  through `-notifications-shell.tsx`; ~12 files under `src/notifications/`;
  module deep links in `notifications/subject-navigation.ts:10-28`.
- Insights tracker: owned entirely by the Insights service
  (`apps/insights/src/features/tracking/`); zero injection points in
  `apps/web` (`index.html`/`main.tsx` carry no tracker; Admin serves the
  embedded dist verbatim, `apps/admin/src/infra/web.rs`).
- Module API clients are module-route-local plus the
  `apps/web/src/api/index.ts:5-8` barrel; the Dashboard renders
  `apps/web/src/routes/-module-health-panel.tsx`, mounted by `DashboardPage`,
  over `GET /api/dashboard/modules`.
- Embedding: `apps/admin/build.rs` hardcodes `../web/dist`, a full-composition
  id (`rustzen-full-composition-v1`) and asserts a non-empty dist; rust-embed
  supports environment-variable folder interpolation for a core web root.
- Selected-web precedent: `scripts/distribution-build-web.ts` +
  `apps/web/vite.config.ts:12-21,56-120` + `src/distribution/` thin portals +
  `scripts/distribution-web-inventory-policy.ts` + `distribution/catalog.json`
  implement module-portal presets only (`monitor`, `monitor-notify`,
  `analytics`, `reports`); no core preset exists. A build-time notification
  strip with residue assertions already exists
  (`scripts/distribution-build-web.ts:85-133`).
- Tests pinning the full inventory: ~10 seam tests under `apps/web/tests/`
  read module route files by path (message-center, analytics-ui,
  monitor-settings-permission, reports-*, layout-navigation) plus colocated
  module tests and API contract tests.

**Schema.** Admin owns one init migration
(`apps/admin/src/infra/db.rs:9` → `migrations/sqlite/0001_init.sql`); core and
full keep it byte-identical, so module/menu seed rows stay dormant in core and
are filtered at runtime boundaries. `apps/admin/migrations/{insights,monitor,
reports,sqlite-monitor,sqlite-analytics,sqlite-notifications}` have no
consumers in `apps/`, `crates/`, `scripts/` or the justfile (selected-era
orphans; retire candidates for a focused composition-scope cleanup commit).

**Packaging, signing, installation, update.**

- `scripts/package-release-bundle.sh` hardcodes the five-binary member check
  (ELF magic, machine byte order, `RUSTZEN_RELEASE_MARKER` identity),
  `config/rz-reports.env`, `controller-protocol.json`, the
  `identity/controller.json` full-controller identity, eight systemd units and
  ustar output.
- `scripts/deploy-sign.mjs`: marker `schemaVersion 2`, payload
  `rustzen-release-v2` with `content/frontend/backend` digests, Ed25519 over a
  canonical string; fixed `BINARIES` list of five; key resolution from
  `.rustzen-admin/config/deploy-sign-private.pem` (present on this machine,
  119 bytes) with an iCloud fallback directory.
- `deploy/setup-layout.sh`: generates all runtime secrets locally (JWT, agent
  token, node id, notification key, reports credential key, bootstrap owner
  password — lines 120-203), verifies the bundle signature itself via a DER
  Ed25519 public key and `openssl pkeyutl` (lines 301-326), refuses an occupied
  `current` link (line 602), refuses replacing unit links/command paths
  (lines 615-626), and switches `current` atomically through
  `.current.new.$$` (line 762).
- Admin update worker (`apps/admin/src/features/manage/deploy/bundle.rs`):
  `BINARIES: [&str; 5]`, exact expected unit bytes including the
  `rz-full.service` `Wants=` aggregate, per-service `ExecStart`, config
  members and the controller identity; the worker re-verifies the signed
  marker in Rust. Backup set is `database_paths() -> [monitor, insights,
  reports, admin]` (`service/database.rs:168-175`). CLI status joins the four
  services (`apps/cli/src/operations.rs:144-150`). A selected-era CLI
  `install` subcommand remains wired (`apps/cli/src/main.rs:126` →
  `install::apply`, 280 lines): it must not become a second production
  installer for core/full; final disposition is fixed in OE-3.

**Legacy full artifacts (OE-A09 inputs).**

- `target/rz/rz-0.5.1-x86_64.tar` (2026-09-23 11:24, 49,005,594 bytes) and
  `target/rz/rz-0.5.0-x86_64.tar` (2026-09-22 19:53) both verify against the
  existing local key with `bun scripts/deploy-sign.mjs verify-bundle`
  (exit 0). Member list confirmed: 5 binaries, 8 units, 2 configs,
  `identity/controller.json`. `target/rz/rz-install` (2026-09-23 14:52) and
  the release build tree `target/rz/build/x86_64/bin/` are present.
- The 0.5.1 worker source is the current
  `apps/admin/src/features/manage/deploy/bundle.rs`: its last change
  (`1a36869`, 2026-09-23 11:09) predates the 0.5.1 bundle build (11:24), and
  no later commit touches it. Its marker struct is a plain serde struct
  (unknown JSON fields ignored, no `deny_unknown_fields`), so appending a new
  marker field is expected to keep new full candidates verifiable by the real
  0.5.1 worker; this is a design expectation only until OE-4 proves it with
  the actual 0.5.1 `rz-admin` in a disposable installation.

**Fixed core/full contract (implementation target for OE-2/OE-3).**

- Cargo: new `core` feature = the current `full` dependency set minus
  notifications (`rustzen-config/admin`, croner, ed25519-dalek, hex, image,
  libc, mime_guess, rust-embed, sysinfo, tar, tower-http/fs); new `modules`
  feature owns module synchronization/registry/gateway/projections;
  `full = ["core", "modules", "reports-notifications"]`. Existing `full` cfg
  gates on Admin-authenticated audit/user/account surfaces move to `core`.
  Core builds via `--no-default-features --features core`.
- Runtime: core runs the complete Admin shell (login with audit, accounts,
  roles, instance RBAC, diagnostics, release/recovery machinery) with an empty
  installed-module inventory; no module endpoints are probed, no notification
  ingress/SSE starts, no module credentials are read. Full keeps today's
  behavior byte-for-byte.
- Web: one shared shell with build-owned route selection: a core staging
  producer keeps the real layout/auth/account/audit surfaces, omits module
  route directories, module API clients and notifications, and strips the
  notification import using the existing verified strip mechanism; core and
  full embed through a parameterized web root with distinct composition ids.
- Signed inventory: the tar gains `identity/composition.json`
  (`{"schemaVersion":1,"composition":"core"|"full",...}`); the signed marker
  keeps every existing field and appends `composition`. The new verifier
  admits a composition-less marker only as exact legacy full (five binaries,
  controller identity, full unit set, valid signature) and never infers core
  from absence. Unknown/tampered compositions are rejected before any write.
  Installed identity + candidate composition must match exactly
  (core→core, full→full); mismatch fails with a supported-path message.
- Members: core bundle = `bin/{rz,rz-admin}`,
  `systemd/{rz-full.service (Wants=rz-recovery.service rz-admin.service),
  rz-recovery.service, rz-admin.service, rz-update.service, rz-update.path}`,
  `config/rz.env`, `setup-layout.sh`, `identity/composition.json`. Full bundle
  keeps today's exact member set plus `identity/composition.json`. Service
  health, CLI status and backup sets derive from the installed inventory.
- Outputs: `target/rz/core/` and `target/rz/full/` (full archive basename
  preserved). Until OE-5 gates close, `just build` keeps writing the legacy
  full path `target/rz/rz-<version>-<arch>.tar`; explicit development commands
  carry the core/full slices. The default flips to core only after the OE-5
  installation/update gates pass.

**Selected-path disposition.**

- Keep/adapt: `scripts/distribution-build-web.ts`, vite selected mode,
  `src/distribution/` portals, `scripts/distribution-web-inventory-policy.ts`,
  `distribution/catalog.json`, `scripts/stage-message-center-web.sh` — the
  core web producer extends this family rather than adding a parallel
  mechanism.
- Retire candidates (composition-scope cleanup, one focused commit, no
  unrelated coupling): `apps/admin/migrations/{insights,monitor,reports,
  sqlite-monitor,sqlite-analytics,sqlite-notifications}`,
  `apps/admin/build_support/selected_api.rs` (dead: no `mod`/`include`
  reference), `apps/web/.selected-web/` leftovers.
- Quarantine decision: the CLI `install` subcommand and `install_*` modules
  stay unwired from the core/full release chain; OE-3 records their exact
  disposition with evidence so no parallel selected-release installer is
  revived.

**Checks and blockers.**

- No blocker for OE-2 (source, tests, local core run). Signing key and legacy
  artifacts are present.
- OE-3 depends on the existing Docker/Buildx Linux build chain and disposable
  systemd containers (established pattern under `target/rz/`); unavailability
  would block only the install/upgrade gates, not source work.
- OE-4/A09 depends on the real 0.5.1 artifacts above; no synthetic versions
  are permitted.

**Next bounded task (OE-2 slice 1).**

1. Cargo feature graph (`core`, `modules`) with
   `cargo check -p rustzen-admin --no-default-features --features core`
   green and full unchanged.
2. Runtime gating: module machinery behind `modules`; core boots with an empty
   inventory; notifications already gated.
3. Local vertical: core `rz-admin` against a fresh Admin DB with real
   login/RBAC/audit API receipts and negative evidence of module probing
   (no connections to 9802-9804).
4. Focused tests for the inventory boundary (dashboard modules endpoint,
   module control routes, gateway behavior in core).

### OE-2 slice 1 record (2026-10-09, Rust composition vertical)

Executed exactly the bounded task above; working tree committed per slice.

**Source changes.**

- `apps/admin/Cargo.toml`: new `core` feature (full dependency set minus
  notifications), new `modules` feature (installed example-module
  inventory), `full = ["core", "modules", "reports-notifications"]`. The
  default feature set and therefore the full build graph are unchanged.
- All 75 former `feature = "full"` cfg gates renamed to `core`
  (complete-Admin semantics: audit-capable login, user/account surfaces,
  permission/menu reconciliation, log middleware). Module-behavior test
  modules (`modules/service/tests.rs`, `gateway_tests.rs`,
  `system/status module_storage_tests`) now gate on `modules`; the
  notifications-contract OpenAPI test and `TraceWriter` gained their missing
  `notifications` gates.
- `features/modules/types.rs`: `ModuleSpec::installed()` returns the fixed
  inventory only in a `modules` build and an empty vector otherwise;
  `ModuleSpec::fixed()` and the `CONFIG` import are `modules`-gated. The
  registry, module service and system-status projections now consume
  `installed()`; `sync_once` iterates the registry snapshot, so an empty
  inventory performs no network probe by construction.
- `features/modules/service.rs`: new `assert_installed` pre-write guard on
  both `set_enabled` variants — enabling anything outside the installed
  inventory is rejected **before** the database write (previously the dormant
  seed row was updated first and the request then failed, leaving a
  write-then-fail inconsistency).
- Update machinery derived from composition: `SYSTEMD_UNITS`
  (`service/mod.rs`), `service_probes()` and `database_paths()` /
  `restore_databases_for_units_at` (`deploy/service/{recovery,database}.rs`)
  are `modules`-gated to the four-service set and collapse to the Admin-only
  set in core.
- `apps/admin/build.rs`: composition identity is feature-derived
  (`rustzen-full-composition-v1` for `modules` builds — byte-identical to
  the previous constant — and `rustzen-core-composition-v1` otherwise) and
  exposed as `RUSTZEN_PACKAGED_COMPOSITION`. The embedded web root remains
  the shared full build until the core Web producer lands (next slice).
- New `features/modules/inventory_tests.rs` (four tests, run under every
  composition with derived expectations): installed inventory matches the
  composition; statuses/dashboard follow the installed inventory; enabling
  outside the inventory is rejected without modifying the dormant seed row;
  synchronizing an empty inventory is a no-op. The contract-inventory test
  now derives its operation count (48 without / 54 with notifications).

**Verification (all real exit codes).**

- `cargo check -p rustzen-admin --no-default-features --features core` —
  pass, no warnings.
- `cargo clippy -p rustzen-admin --all-targets -- -D warnings` and the same
  with `--no-default-features --features core` — both pass.
- `cargo test -p rustzen-admin -- --test-threads=1` — 220 passed / 0 failed
  (full unchanged).
- `cargo test -p rustzen-admin --no-default-features --features core --
  --test-threads=1` — 151 passed / 0 failed (core).

**Local core vertical (composition `core`, target = this host, debug
binary, fresh runtime root `/tmp/rz-core-vertical/root`, port 19801).**

- Build: `cargo build -p rustzen-admin --no-default-features --features
  core` (exit 0). Startup log contains no module/manifest/notification/
  ingress lines; `lsof` shows the process's only socket is its own
  listener — zero outbound connections, i.e. no probing of 9802-9804.
- Real API receipts (originally captured under the volatile
  `/tmp/rz-core-vertical/`; persisted and sanitized to
  `target/rz/oe2-core-vertical-20261009/` on 2026-10-10 per DOT-OE-1900-04 —
  see `PROVENANCE.md` there; the original pointer in the slice-1 record
  claimed the durable path before the copy actually happened and is hereby
  corrected):
  `POST /api/auth/login` (owner) → code 0 with a 339-byte token;
  `GET /api/auth/me` → owner identity with permissions;
  `GET /api/manage/logs` → `AUTH_LOGIN 用户登录成功` plus request audit rows
  (audit capture/list work in core);
  `GET /api/system/roles` → owner/admin/viewer;
  `GET /api/dashboard/stats` → account totals;
  `GET /api/dashboard/modules`, `GET /api/system/modules`,
  `GET /api/system/modules/navigation` → 200 with empty arrays;
  `PUT /api/system/modules/monitor/enabled` → 404 "Module monitor not
  found." (pre-write rejection);
  `GET /api/monitor/nodes` and `GET /api/reports/systems` → 404 (gateway
  fail-closed with an empty registry);
  `GET /api/notifications/unread-count` → 404 (notification surface absent).
- Runtime root after the run contains only `admin.db` (+wal/shm) — no module
  databases are created.

**Boundary queue receipt:** checked after OE-0/OE-1 and again at this
boundary — `/tmp/rustzen-optional-examples-changes.md` still does not exist;
no entries to acknowledge.

**Next bounded task (OE-2 slice 2): the core Web build.** One shared shell
with build-owned route selection: a staging producer that keeps the real
layout/auth/account/audit surfaces, omits the module route directories,
module API clients and the notification shell, strips the `__root.tsx`
notification import using the existing verified strip mechanism, builds into
a dedicated output directory, and is embedded by a core Admin build through a
parameterized web root with the core composition identity. Exit: core Admin
serves the core Web build (login page + authenticated core surfaces over the
real API), full Web builds/tests stay green, and the embedded core asset set
contains no module route chunks or notification code (asserted by inventory
checks, not by navigation hiding).

### Coordinator queue receipts — 2026-10-10 boundary

Queue `/tmp/rustzen-optional-examples-changes.md` appeared at 19:07 on
2026-10-09 (absent at the OE-0, OE-1, and slice-1 boundary checks recorded
above) with four entries. Read and dispositioned this boundary; the queue is
not project truth.

**DOT-OE-1900-01 — applied.** Core configuration isolation. Reproduced on
the slice-1 binary (sha256 `e053e616…`): `RUSTZEN_REPORTS_PORT=not-a-number`
with a cleared environment exits 101 during extraction, before database
startup. Fix: `crates/config` gains a `modules` feature; the module
endpoint/database fields, their defaults, accessors, and the module-path
validation now gate on the composition (`any(admin-monitor, modules)` for
monitor, `any(admin-insights, modules)` for insights, `modules` for reports)
instead of the Admin-wide `admin` feature, and `apps/admin`'s `modules`
feature enables `rustzen-config/modules`. The IPC production-secret
requirement now applies only in the modules composition (core has no module
endpoint to authenticate; full unchanged — negative control re-verified).
`infra/app/runtime.rs` no longer spawns the module synchronizer in core
(compile-time, not just empty-inventory), and the sync machinery
(`spawn_synchronizer`/`sync_once`/`sync_module`, manifest-hash bookkeeping,
module permission/navigation reconciliation) is `modules`-gated throughout.
Verified: core serves `/health` 200 under the previously fatal environment;
full still exits 101 there; receipts and SHAs in
`target/rz/oe-queue-1900-20261010/EXIT-RECEIPTS.md`.

**DOT-OE-1900-02 — applied.** The inventory regression now attempts a real
state change: seed rows start `enabled=1` and the test attempts
`set_enabled(…, false)`, asserting rejection plus an unchanged stored value;
the modules composition exercises the same invariant by installing a subset
registry (the `modules` table CHECK constraint limits ids to the three
examples, so a foreign row cannot be seeded). The positive authorized
mutations remain covered by `features/modules/service/tests.rs`. The
now-vacuous `synchronizing_an_empty_inventory_is_a_no_op` test was removed:
core no longer compiles a synchronizer at all, which is the stronger
guarantee. Four module-reconciliation permission tests moved under the
`modules` gate with their subject.

**DOT-OE-1900-03 — deferred to OE-3/4 with a binding constraint.** The
proposal to add `identity/composition.json` to the full archive conflicts
with the old worker's exact-inventory member check
(`features/manage/deploy/bundle/validation.rs` rejects any extra archive
member; `deploy/setup-layout.sh:456-466` rejects unknown paths); marker JSON
tolerance does not bypass either. Admissibility for the full archive
therefore requires proving a direct upgrade with the actual old signed
artifacts (`target/rz/rz-0.5.1-x86_64.tar`, worker = the 0.5.1 `rz-admin`)
or a compatible full bridge that adds no unrecognized member — JSON-field
tolerance alone is not acceptable evidence. This does not block the core Web
work. The backup/probe set derivation (compiled-feature-derived today)
moving to authenticated installed/candidate inventory plus journal remains
OE-3/4 scope.

**DOT-OE-1900-04 — applied.** The slice-1 evidence pointer has been
corrected in place above (the durable copy only happened during this
boundary). The full vertical evidence is persisted, sanitized (login JWT
redacted; original and redacted-token SHA-256 recorded), with per-file
limitations documented in
`target/rz/oe2-core-vertical-20261009/PROVENANCE.md`: bodies alone do not
encode HTTP statuses or process identity, and the one-time `lsof` snapshot
does not prove the absence of transient connections over the whole run.
Final suite stdout pointers for slice 1: `/tmp/rz-ft3.log` (220/0/1) and
`/tmp/rz-ct3.log` (151/0/1); earlier `/tmp/rz-full-test.log` and
`/tmp/rz-core-test.log` runs contained failures and are superseded.

**Post-receipt suite state (2026-10-10).** Core suite 146/0/1 (−6/+1: the
renamed inventory test, the removed vacuous sync test, four reconciliation
tests now modules-only); full suite 219/0/1; workspace
`cargo clippy --workspace --all-targets -- -D warnings` exit 0;
`cargo test --workspace -- --test-threads=1` exit 0 (478/0 summed);
`rustzen-config` combos: default 27/0, admin-only 10/0, +modules 26/0,
legacy `admin-monitor` 7/0. New config tests:
`core_admin_ignores_absent_module_settings`,
`core_admin_production_does_not_require_module_secrets`.

**Next bounded task is unchanged: OE-2 slice 2, the core Web build** (exit
criteria above).

### OE-2 slice 2 receipt — the core Web build (2026-10-10, commit after `b532439`)

Executed to the exit criteria above; baseline `b532439`, tree clean at start.

**Producer.** `scripts/distribution-build-web.ts` gained a `core` preset
branch and `distribution/catalog.json` a `core` preset
(`access`, `admin-console`, `release-ui`, `task-admin`, `log-console`;
fixture `distribution/fixtures/core.json`). The core branch stages the real
Admin route set from the selection's web roots plus every non-test
co-located helper under `routes/{system,manage}` (globbed, not
hand-listed), strips the `__root.tsx` notification import with the existing
verified strip (now with a residue assertion), mirrors the real
`index.html` head (theme/locale bootstrap) instead of the portal stub, and
keeps the four shell seams resolving to their real modules — only portal
presets swap them. Vite accepts `RUSTZEN_WEB_PRESET=core` with no seam
aliases. Output: `apps/web/dist-core` (`just build-web-core` /
`bun run build:core`); no portal binding machinery is applied.

**Boundary assertions (in the producer, fail-closed).** Module-graph absence
is asserted for module route directories and the notification shell
(graph-level); module API clients and notification text are denied at the
emitted-asset level — every emitted `.js/.css/.html` is scanned for
`/api/monitor/`, `/api/insights/`, `/api/reports/`, `/api/notifications/`,
and `notification-delivery` — plus required-route positives (all twelve
real Admin routes must be in the graph) and a notification-asset name
check. The asset-level scan exists because the real `src/api` barrel
statically re-exports every module client, so client modules legitimately
appear in the module graph while being tree-shaken out of the bundle; the
assertion therefore targets what the core Admin actually embeds.

**Embed.** `apps/admin/build.rs` `web_root()` is composition-parameterized
(core → `../web/dist-core`, full → `../web/dist`) and `infra/web.rs`
defines the `WebAssets` embed per composition with the same split; a
missing core build fails the build with a non-empty inventory assertion.
The composition identity remains the feature-derived marker from slice 1.

**Verification.** Core Web build exit 0 with all assertions passing; full
Web build exit 0; core Admin (re)built and served over a cleared
environment on port 19804: `/` → 200 `Rustzen Admin` with the core build's
hashed entry chunk served 200 (272739 bytes), owner login → `me` and
`dashboard/stats` 200 over the real API, and the served `index.html` +
entry chunk contain zero module endpoint or notification markers. Receipts
and SHA-256s in `target/rz/oe2-slice2-core-web-20261010/PROVENANCE.md`.
Web gates: `bun test` 174/0, `vp fmt --check` clean, `vp lint` 0 errors,
`tsc --noEmit` clean. Rust gates after the embed change: core suite
146/0/1, full suite 219/0/1, clippy `-D warnings` clean in both
compositions.

**Found defect (pre-existing, documented — portal chain, not core).**
The four selected-Web distribution tests were red at the `b532439`
baseline (verified by stashing this slice's changes): `733c8b1` ("unify
signed 0.5.1 server distribution") added the shared route-keyed icon map
to all three portal layouts and removed the notification delivery card
from the Monitor incidents surface without updating the selected-Web
policy chain. Repaired in this slice (tightly scoped):
`lucide-react` added to `distribution-web-allowed-packages.ts`; the three
`src/distribution/*-layout.tsx` now carry their own lucide icons (same
glyphs and rendering parameters as the shared map) so the shared map's
cross-composition route strings stay out of portal bundles. Three of the
four tests are green again. The fourth
(`distribution-notification-web.test.ts`, monitor-notify delivery closure)
remains red: the delivery UI now lives only in `reports/runs.tsx` while
the Monitor web client (`src/api/monitor/api.ts` +
`notification-contract.ts`), the Rust `selected_contract` route, and the
ops evidence scripts still carry `/api/monitor/notification-delivery`.
Completing that migration is a cross-layer bounded task of its own (web
client + web contract + monitor `selected_contract` + parity/evidence
scripts); it is recorded here and left for a dedicated slice — no repo
gate runs these four tests (`just check`/`contract-verify` do not invoke
them), so the baseline gate stays green.

**Next bounded task: OE-2 slice 3 — core packaging integration** (extend
the package script and signed marker with the composition field per the
adopted contract; the DOT-OE-1900-03 real-artifact proof rule applies
before any full-archive admission).

### Coordinator queue receipts — 2026-10-10 second boundary (DOT-OE-1010)

Queue digest at this boundary: sha256
`f1525abd5f011d63742dc43b6ca657b81d8a938850ceed9f33baf7199f0d7784`.
Stage acceptance recorded: `b532439` (config isolation fixes) and
`9c9ab2f` (core Web producer) accepted as stage progress — not overall
delivery. The prior-queue disposition block also accepts 1900-01/02/04
fixes from current source and keeps 1900-03 as the OE-3/4 release blocker.

**DOT-OE-1010-01 — applied.** The Monitor notification delivery surface is
restored as the promised contract: `733c8b1` had dropped the
`NotificationDeliveryCard` from `routes/monitoring/incidents.tsx` (an
incidental casualty of its table-preset refactor) while the client method,
web contract, Rust `/api/monitor/notification-delivery` route, and the
Linux evidence scripts' DOM/API expectations all stayed — the closure
contract on disk remained the promise, so per the queue's preference the
UI was restored to the exact pre-`733c8b1` shape (byte-identical
`deliveryCard` block and the three `actions={deliveryCard}` placements, so
the pure-Monitor strip regexes keep matching). All four selected-Web
distribution tests pass (4/0) — monitor-notify's delivery closure is
complete again and no assertion was relaxed or deleted. The composition
checks are now wired into an invoked gate: `just verify-web-compositions`
runs the four selected-Web tests plus `bun run build:core`, and `just
check` invokes it. Contract alignment cross-checked against
`monitoring-ui-state-delivery-lib.sh` (expects
`[data-testid=notification-delivery-card]`, provided by the restored
card) — script execution itself is Linux-stack verification and stays
unexecuted on this host.

**DOT-OE-1010-02 — applied.** Final-source evidence index persisted at
`target/rz/oe-1010-evidence-20261010/INDEX.md`: base HEAD
`9c9ab2f2804ade8f58272978cd5e8aaa95757c5c`, dirty-diff digest
`e3101dfe87eae4c7469c74f4dd8f10b06aebf2c196fe09cf66b1bb488cfba944`, with
per-file SHA-256 for the Web 174-test log, full Web build log, the four
selected-Web tests log, both Rust suite logs (146/0/1 core, 219/0/1 full),
and both clippy logs. Pre-commit versus post-commit evidence is
distinguished explicitly: all logs were captured on the dirty pre-commit
tree whose bytes equal the follow-up commit's tree, and a post-commit
anchor (clean tree SHA + fast-gate rerun) is recorded in the index. The
earlier `/tmp/rustzen-template-tests.log` was not reused (it belongs to
earlier template work, per the queue).

**Blocked / not claimed at this boundary** (unchanged): browser DOM
acceptance of the core shell and the restored card; Docker/Linux runtime
verification; signed packaging and the DOT-OE-1900-03 real-artifact
upgrade/rollback proof (OE-3/4); the queue also notes build.rs's
non-empty-root check is not yet authenticated correct-root binding — that
belongs to packaging acceptance (slice 3+).

**Next bounded task: OE-2 slice 3 — core packaging integration** (per the
adopted contract, with DOT-OE-1010's constraints folded in: composition
identity must be anti-misplacement — the marker's composition must match
the archive contents and mismatches must be rejected before candidate
install, backup, or switch; the
full archive must not gain members the old worker rejects, and admission
of any full-archive change requires the real 0.5.1 signed-artifact
upgrade/rollback proof, never JSON-tolerance inference).

### OE-2 slice 3a receipt — composition-aware update chain and packaging (2026-10-10)

Executed after the DOT-OE-1010 boundary commit `36e1c4a`; evidence in
`target/rz/oe2-slice3a-packaging-20261010/PROVENANCE.md` (base HEAD, dirty
digest, per-log SHA-256, exit receipts).

**Worker.** The update worker now derives an archive's composition from
its exact member inventory (`bundle/validation.rs` `ArchiveShape`: full =
the legacy set, core = `rz` + `rz-admin`, five core units, shared config,
no Reports env and no controller identity) and rejects before candidate
install, backup, or switch:
a bundle whose composition differs from the compiled build
(anti-misplacement, `COMPILED_COMPOSITION`), and a signed marker whose
composition (absent = legacy full) differs from the archive. The
signature payload remains `rustzen-release-v2` bytes so legacy workers
keep verifying full bundles — composition travels as marker metadata plus
member-set derivation, not as a payload change. Install-path
normalization tolerates the core shape's absent members. Tests: foreign
archive and marker-mismatch rejection in both compositions, legacy-marker
admission (full), plus the real-artifact test below.

**Real legacy artifact (DOT-OE-1900-03, new-worker direction).**
`accepts_the_real_legacy_signed_artifact` validates the real
`rz-0.5.1-x86_64.tar` (sha256 `01acde7f…`) with full signature
verification against the real deployment key parsed from
`.rustzen-admin/config/deploy-sign-private.pem` — 1/0, exit 0. The
signer chain verifies the same artifact (exit 0) and rejects it when
claimed as core (exit 1, "members derive full"). **The old-worker
direction stays blocked**: the 0.5.1 worker is an x86_64 Linux ELF, this
host is macOS/arm64 with Docker down and no x86_64 musl cross target —
no Linux runtime and no real new x86_64 artifact can be produced here.
That direction requires the Linux environment; JSON-marker tolerance of
the old worker remains a source-backed expectation, never admission
evidence.

**Signer and packager.** `deploy-sign.mjs` takes
`--composition <full|core>` (default full), derives the composition from
the member inventory, cross-checks the claim, and writes `composition`
into the marker (verification treats a missing field as legacy full).
`package-release-bundle.sh` takes an optional fifth `composition`
argument: core bundles stage the two binaries, the five units from the
new `deploy/core/` (aggregate/recovery/admin/update units without module
references and with Admin-only `ReadWritePaths`), and omit the Reports
config and controller identity; the full path is unchanged. The core
fresh-install installer variant of `setup-layout.sh` is deliberately not
in this slice — it is the next bounded packaging task with its own test
harness.

**Suites.** Core 148/0/1, full 223/0/1 (both +new tests), both clippy
`-D warnings` clean, `cargo fmt --all -- --check` clean (the fmt pass
also normalized drift from earlier slice commits), workspace check exit 0.

**Next bounded task: OE-2 slice 3b — core fresh-install installer**
(composition-aware `setup-layout.sh` + `rz-install` generation + local
harness tests), with the Linux runtime proof (old-worker upgrade/
rollback, real core package build) remaining environment-blocked and
tracked in the task ledger (OE-T10/OE-T12).

### Boundary supplement receipt — 2026-10-10 (post slice 3a)

Ordinary boundary, no new writer; slice 3 continues (3b is Ready).

- Stage acceptance recorded from the coordinator: `36e1c4a`'s
  monitor-notify delivery closure and the composition gate are
  independently accepted as a stage.
- Evidence indexes corrected in place without rerunning anything: both
  `target/rz/oe-1010-evidence-20261010/INDEX.md` and
  `target/rz/oe2-slice3a-packaging-20261010/PROVENANCE.md` now carry a
  version header, the post-commit log SHA-256
  (`c46ef646…` and `e11651d0…` respectively), and an explicit
  dirty-digest-versus-final-commit explanation: each dirty digest was
  captured before the authoritative-documentation edits, so the commit
  equals the digested tree plus those documentation files; the logs
  verify code states that are byte-identical across that boundary.
- Exit-code binding policy corrected: a `cargo … | head; echo $?` pattern
  was observed printing the pipe tail's status (one `fmt=0` while the
  check had actually failed; the true check caught it and `cargo fmt
  --all` was applied before the commit). All exit codes bound as evidence
  were captured with redirection only (raw process status); future
  bindings use redirection, `pipefail`, or `PIPESTATUS`.
- Packaging constraints reaffirmed for slice 3b (folded into OE-T12
  acceptance): the installer and packager must bind signature identity,
  the binaries, and the Web composition together; wrong package, stale
  assets, or tampering must be rejected before any overwrite and before
  any database or service change; the legacy full member compatibility is
  preserved.
- The old-worker direction of DOT-OE-1900-03 remains unmet and is not
  substitutable: the new verifier accepting the old artifact does not
  prove the old worker accepts a new one; that requires the real
  signed-artifact upgrade/rollback on a Linux environment (OE-T10).
- Logs stay persisted outside git tracking (under `target/rz/`, which is
  gitignored); the indexes bind source, hashes, and commands, so
  gitignored-ness alone is not treated as evidence failure.


### Signer closure receipt — 2026-10-10 (minimal review package)

Independent review of `3f59559`/`1729777` found the signer chain
endorsing an uninstallable package: the composition derivation in
`deploy-sign.mjs` was heuristic (Monitor-binary presence plus Reports/
identity absence), so a core archive with a foreign `bin/rz-insights`
passed both `sign-bundle` and `verify-bundle --composition core` with
exit 0 (reviewer artifact reproduced: both exit 0 before the fix). The
Rust worker would have rejected the archive, but runtime isolation does
not excuse the signing chain vouching for it.

Fix (minimal unification): the signer and verifier now derive the
composition by **exact member-inventory equality** against the same
shapes the Rust worker enforces (`ARCHIVE_SHAPES`, in lockstep with
`bundle/validation.rs`): an archive with an extra module binary, a
missing member, or an unknown member matches neither shape and is
neither signed nor verified. Focused tests
(`scripts/deploy-sign.test.ts`, 7 pass / 0 fail): exact core and full
positives; the review counterexample (sign rejection plus a
signed-then-spliced verify rejection); missing-member, unknown-member,
and full-missing-Reports-config negatives; and the real legacy artifact
(verifies as full, rejected as core — runtime `skipIf` carries an
explicit NOT-VERIFIED name, absence is never a pass). The reviewer's
artifact and result are persisted sanitized with hashes in
`target/rz/oe3-signer-closure-20261010/PROVENANCE.md`; their test-only
key was deliberately not copied. Already-green suites were not rerun;
the Rust worker is untouched.

Wording corrections: "rejected before any write" claims are narrowed to
"before candidate install, backup, or switch". The old-worker direction
of DOT-OE-1900-03 remains unmet and unverified (real Linux upgrade/
rollback), not substitutable by any result here. OE-T12 stays gated on
this package's independent review.


### Writer handoff checkpoint — 2026-10-10

Division of labor changed at this boundary: the writer role returns to
the coordinator (dot). The previously instructed OE-T12 implementation
package is **not started**; this checkpoint only preserves state and
hands over.

- Signer closure `b2de445` **independently re-reviewed and accepted**:
  counterexample reproduced as rejected (core + rz-insights, sign and
  verify both exit 1), focused suite 7/0 with no skips including the
  real legacy full positive and the claim-as-core rejection. Re-review
  log preserved sanitized at
  `target/rz/oe-checkpoint-handoff-20261010/signer-rereview-focused-tests.log`
  (sha256 `ed12dc66…`).
- Acknowledged open limitation (not masked): the Rust
  `accepts_the_real_legacy_signed_artifact` test returns silently when
  the artifact is absent — skip-shaped, never a pass.
- Repository state at handoff: branch `feat/theme-verification-20261005`
  (non-main, reused — no new branch, no empty commits), HEAD `b2de445`,
  tree clean including untracked, single worktree, two pre-existing
  foreign stashes untouched, no parallel writers. Full checkpoint
  record: `target/rz/oe-checkpoint-handoff-20261010/CHECKPOINT.md`.
- Status ledger: Done = OE-0/1, slice 1, DOT-OE-1900-01/02/04, slice 2,
  DOT-OE-1010-01/02, slice 3a + signer closure (review-accepted);
  Not executed = OE-T12 installer; Not verified = real Linux
  deployment, old-worker upgrade/rollback, interrupted recovery on a
  real host, browser DOM acceptance.
- Next executable package: OE-T12 per its ledger acceptance (exact
  member shapes, rejection before any overwrite and before database or
  service changes, legacy full compatibility).


### OE-T12 receipt — composition-aware installer (2026-10-10)

Executed as the sole writer after the 14:33 division change (ZCode
implements, dot reviews). Baseline `b6af0ee` verified clean with no
parallel writers before start.

**Found and fixed regression.** At the baseline, the local installer
harness failed with "signed release bundle marker metadata is invalid":
the accepted signer package (`b2de445`) always writes the marker
`composition` field while the installer's strict marker reconstruction
did not include it — full installs were broken. The installer now parses
the optional composition, reconstructs both marker shapes exactly
(legacy without the field included), and binds the marker composition
(absent = legacy full) to the member-derived composition.

**Composition awareness.** The installer derives the composition from
the exact member inventory (full = legacy 17 files, core = 10 files;
directories excluded) before signature work and before any overwrite or
database/service change: a foreign module member, a missing member, or
an unknown member matches neither shape and is rejected outright.
Binary identity, the Admin frontend-digest binding, backend inventory
digest, and signature checks run per composition unchanged; the core
install creates only Admin-owned units, directories, and config (no
module service accounts, databases, logs, or the Reports config), and
`rz-install` generation is unchanged — one installer serves both
compositions.

**Verification.** Harness `sh scripts/test-setup-layout.sh` — 21 groups,
exit 0: the original 16 full groups plus the core positive and four
negatives (core+foreign binary, marker-composition mismatch, missing
member, unknown member), with rejected runs asserting no release
directory was created. The real signed `rz-0.5.1-x86_64.tar` installs
end-to-end through the new installer with the real deployment key
(exit 0, full composition, five binaries, zero placeholders, exact
systemctl sequence; secret-bearing outputs not persisted). The accepted
signer tests remain 7/0 unchanged. Evidence and SHA-256 receipts:
`target/rz/oet12-installer-20261010/PROVENANCE.md`.

**Not verified (no local-harness extrapolation):** real Linux deployment
(systemd/users/paths), old-worker upgrade/rollback on a real host,
interrupted-update recovery, browser DOM. The Rust real-artifact test's
silent-return-on-absence limitation stays open, never masked as a pass.


### OE-T12 acceptance-boundary receipt — 2026-10-10 (dynamic negatives + OE-T10 preparation)

- Ledger status wording unified: OE-T12 is "implemented with local
  evidence, independent acceptance pending" (the coordinator's
  source-level review of code and log identities is recorded as such,
  not as dynamic acceptance).
- Dynamic negative verification by the writer, current installer from
  `79f1206`, fresh test key, per-case assertions of non-zero exit,
  expected rejection class, empty install root, untouched systemd
  directory, and zero systemctl invocations: 6/6 passed — core+foreign
  member, marker-composition mismatch in both directions, tampered
  content (signed then flipped byte), wrong signing key (valid signature
  by another key), and a wrong per-binary identity marker. Driver, logs,
  and per-case SHA-256s:
  `target/rz/oet12-negative-verification-20261010/PROVENANCE.md`.
- OE-T10 environment preparation: the isolated x86_64 colima VM
  `rz-p8e-amd64` exists with a working `rust:1.95` image and the repo's
  systemd image (probed live, restored to stopped); the real 0.5.1
  artifact (worker re-confirmed x86_64 ELF) and the real signing key are
  on the host. Recovery plan and the prepared upgrade/rollback protocol
  are recorded; Mac + stub-systemctl runs and new-verifier-accepts-old
  remain explicitly excluded from that acceptance.
  `target/rz/oet10-env-prep-20261010/ASSESSMENT.md`. The in-VM build
  window and update-driving script are the remaining work items.
- Unchanged: the Rust real-artifact test's silent-return-on-absence
  limitation stays open, never masked as a pass.


### Real-artifact test NOT-RUN gate — 2026-10-12

Minimal closure of the open limitation recorded at the 2026-10-10
handoff: the Rust `accepts_the_real_legacy_signed_artifact` test no
longer returns silently when the real legacy artifact or the real
deployment key is absent. A tests-module-only
`require_real_legacy_inputs` gate reports exactly which inputs are
missing with an explicit `NOT-RUN:` message and the test panics on it,
so absence is a loud fail-closed failure — never a skip and never a
skip-shaped pass. This is deliberately stricter than the TS signer
suite's named NOT-VERIFIED skip convention: a Rust skip (silent
return or ignored test) could still be counted as green, so here
absence must fail the test and make cargo itself exit non-zero.
Production code is untouched; the change lives entirely in the
bundle.rs test module
(`apps/admin/src/features/manage/deploy/bundle.rs`, sha256
`27f74776b90729c72d555a4be5a5846810845182f7c827ea253af33f9f75e1af`
after the change). The gate helper's four input states — both inputs
missing, only the key file present, only the artifact present, both
present — each have a focused assertion in
`missing_real_legacy_inputs_are_an_explicit_not_run_never_a_pass`.

Implementation versus acceptance on this host (baseline `34fb13b`,
branch `feat/theme-verification-20261005`, working tree carrying only
this gate's candidate changes; real artifact and real key confirmed
present by read-only existence check only — no key content read in
this package; full commands, direct exit codes, logs, and SHA-256s in
`target/rz/oe-legacy-gate-20261012/PROVENANCE.md`):

- Implementation complete: the NOT-RUN gate plus the four-state
  helper test live in the bundle.rs tests module.
- Dynamic acceptance complete — the missing-input counter-example ran
  against the real entry, not just the helper: in an isolated
  public-source copy exported from git-tracked files only (no
  `target/`, no `.rustzen-admin/`, no real key or fixture — verified
  by a post-export find), `cargo test --locked --offline -p
  rustzen-admin --features modules
  accepts_the_real_legacy_signed_artifact` compiled cleanly with
  `modules` explicitly enabled, ran exactly 1 test (the real entry),
  FAILED at the gate panic with `NOT-RUN: missing real legacy inputs`
  naming both copy-local paths, and cargo exited 101 directly
  (redirection only — no pipeline exit codes). A control run of the
  four-state helper test in the same copy passed 1/0, so the failure
  is precisely the missing-input gate, not a broken build, zero
  tests, or an unmatched filter.
- Re-verified in the real repository with direct cargo exit codes
  (default features, which resolve `modules` through `full`): the
  three filtered tests —
  `missing_real_legacy_inputs_are_an_explicit_not_run_never_a_pass`,
  `legacy_marker_without_composition_is_admitted_as_full`, and
  `verifies_one_signature_over_the_complete_bundle_and_rejects_tampering`
  — each 1 passed / 0 failed, exit 0.
- `cargo fmt --all -- --check` — pass (pre-existing nightly-only
  rustfmt config warnings unchanged).

Not executed in this package (no fabricated acceptance): the
real-artifact positive admission itself — executing it reads the real
private key and belongs to the OE-T10 evidence window; the full `just
check` suite. Unverified boundaries unchanged: Linux real candidate
packages; real old-worker upgrade/rollback.


### Selection split — ordinary CI vs the real-material gate (OE-LEGACY-SELECT-01) — 2026-10-12

Independent review of the NOT-RUN gate surfaced a CI-shape gap:
`default = ["full"]` includes `modules`, so the ordinary CI
`cargo test --workspace` run selects
`accepts_the_real_legacy_signed_artifact` and would depend on this
host's private inputs. Minimal split (task OE-LEGACY-SELECT-01, v1):

- Implementation: the test carries `#[ignore = "...run via just
  test-real-legacy"]` in addition to `#[cfg(feature = "modules")]`;
  the NOT-RUN panic is unchanged, nothing else is ignored, and no
  silent return was restored. `just test-real-legacy` is the single
  dedicated entry; it runs `scripts/test-real-legacy-gate.sh`, which
  first verifies with the exact production selection (`-p
  rustzen-admin --bin rz-admin --features modules` + the full test
  name + `-- --ignored --exact --list`) that exactly one target test
  is selected — otherwise non-zero — then executes `--ignored
  --exact --test-threads=1`, verifies the target ran exactly once,
  and exits with cargo's raw exit code. An empty or non-matching
  selection can never fake green.

Four-category status (commands, direct exit codes, and log hashes in
`target/rz/oe-legacy-select-01-20261012/PROVENANCE.md`; runs in an
isolated public-source copy without the real key or fixture):

- Implementation: complete — `#[ignore]`, the gate script, and the
  just recipe.
- Ordinary CI ignores the real-material test: verified — the exact
  ordinary invocation (no `--ignored`) exits 0 with
  `0 passed; 0 failed; 1 ignored`, the ignored line naming the
  dedicated recipe.
- Real-material gate negative path: verified — the dedicated
  invocation without inputs ran exactly the one target test, FAILED
  on the NOT-RUN panic, and cargo exited 101 raw; end-to-end `just
  test-real-legacy` propagated 101 through the selection gate. The
  selection-gate negative: a deliberately non-matching name listed
  zero tests and the same gate rejected it non-zero without
  executing (a raw wrong-name run would exit 0 with
  `running 0 tests` — the empty-selection fake green the gate
  forecloses).
- Positive unverified: the real-input positive admission (reads the
  real private key; OE-T10 window) was not executed; Linux real
  candidate packages and real old-worker upgrade/rollback remain
  unverified.

#### v2 hardening after independent review (P1) — 2026-10-12

Review of the v1 gate found (P1): the script's `${1:-canonical}`
argument allowed substituting another real ignored test (the
route-contract benchmark) and verifying a different test green; a
leading-dash argument would be parsed by cargo/grep as an option,
with `|| true` swallowing grep errors so an empty count could slip
through the integer comparison; and the ran-count failure exit could
mask cargo's original non-zero code. v2 fixes all three in
`scripts/test-real-legacy-gate.sh` only (the justfile recipe and the
Rust code are unchanged):

- The production entry takes NO arguments: any argument — another
  ignored name, a leading-dash option, an extra word — exits 2 before
  cargo is invoked. The target is a single readonly canonical
  constant.
- Counting is exact-line awk with always-numeric results (no grep
  option parsing, no error-swallowing fallback); zero, duplicate,
  wrong-target, and unparseable lists all fail closed.
- cargo's exit code is preserved verbatim for both the `--list` and
  the execution call; counting can only turn a cargo-success with
  zero/duplicate/wrong targets into a gate failure (exit 1), never
  mask cargo's original code.

Evidence in `target/rz/oe-legacy-select-01-v2-20261012/PROVENANCE.md`
(every run recorded by a wrapper persisting the exact argv with real
boundaries, run id, gate-script sha256 at start, full stdout/stderr,
and a separately captured decimal rc — no pipeline between execution
and rc): a fake-cargo deterministic harness drove the production
script through twelve scenarios, 19/19 assertions — argument
rejections with the cargo-call marker proving zero invocations; list
zero/duplicate/wrong/garbage rejections without execution; cargo list
rc 2 and execution rc 101 preserved verbatim; cargo-success zero-ran
and duplicate-ran fake greens rejected; healthy single-target
passthrough rc 0. The real ignored benchmark was never executed and
no replaceable production-target interface was reintroduced. In the
real-material-free isolated copy: the production gate exited 101
after listing exactly the one canonical target and failing it on
NOT-RUN (matching the raw cargo rc 101), and the ordinary exact
invocation exited 0 with `0 passed; 1 ignored`. Real legacy positive
(OE-T10), Linux real packages, and old-worker upgrade/rollback
remain unverified.

### OE-T12 local independent acceptance and OE-T11 partial DOM — 2026-10-13

Reviewer: current Codex coordinator (not the original installer implementer).
Fixed basis: parent b6af0ee2b8f03244322741428cf6acb1a77114aa to
79f1206c227a8fccff782289dbbae11761159e5e; raw parent metadata and objects
verified, repository is not shallow. Local source review covers installer
composition/config/account/unit branches, marker/signature/content/backend/Web
identity checks before installation writes, and focused fixture assertions.
The installer, harness, signer and core units have no diff from that commit
in the current checkout. Standards: no actionable finding in this bounded
change review. Spec: local core/full inventory, failure-before-install and
exact full compatibility scope supported; Linux lifecycle remains Not verified.

Evidence accepted by readback: original installer PROVENANCE and four matching
log hashes, 21-group green terminal (baseline failure retained), signer 7-pass
terminal; negative-verification SHA256SUMS 14/14 matching, six rejected cases
and 6/6 run.log terminal, with driver assertions for empty install roots and
zero systemctl calls. These are historical writer executions, not this
reviewer's fresh dynamic run. This review accepts OE-T12's local scope only;
no signing key read, installer execution, package generation, service start
or Linux acceptance occurred. Both shell syntax checks returned 0.

OE-T11: existing identity-pinned V2 core/full at ports 9821/9801 verified by
lsof executable/cwd, binary hashes and actual served index hashes matching
original records. Browser at 1920x1080 verified core [] without a module panel
and full's three unavailable modules/cards. Monitor delivery-card states
remain unverified. Runtime/DOM/network/screenshot details and remaining
A-04/OE dependencies: [A-04/A-06 receipt](../../../plans/product-iteration-tasks.md).
The original V2 binaries predate the dotenv/launcher isolation fix; none of
this evidence upgrades that new launcher's runtime acceptance.

### OE-T11 failure-surface acceptance — original ZCode client, 2026-10-13

The same original Admin session continued the A-04 package through the ZCode
client. Existing full V2 at 9801, 1920x1080/light, /monitoring/incidents:
DOM/screenshot shows Delivery status failed and Failed to load alert incidents
with Reload. Browser requests incidents and notification-delivery; authenticated
actual readback returns HTTP 503/code 40001 (Monitor worker unavailable).
This accepts the currently observable failure surface only; same-version
Monitor registration/availability and delivery data are still needed for
success, pending/quarantine/details and the remaining journey. No service was
started, no candidate built, no source changed. Original OE-T11 remains Blocked
for those dependencies. [Coordinator case receipt](../../../plans/product-iteration-tasks.md)
contains current model/session attribution, eight-case denominator, source
hashes, screenshot/response provenance, cleanup and restore conditions.


### OE-T11 bounded local acceptance — 2026-10-23

The current Codex coordinator and one implementation/acceptance owner completed
the explicitly authorized disposable Admin/Monitor window (9801/9802/9811,
fresh SQLite, synthetic reports, no external notifications). Full Web and
Admin/Monitor builds, the core Web/Admin build, focused card tests (2/32), and
unchanged composition tests (4/79) passed. Three code fixes close the missing
core dashboard helper, the non-core dialog-session staging/verifier omission,
and narrow delivery-popover clipping. Real reports produced four matching
Monitor-to-Admin stored events. Served core UI and layered full-card
ready/details, aggregate-only gap fixture, controlled 403/503/retry/recovery
checks passed. Fixtures, services, databases and source/build copies were cleaned.

ZCode independently reviewed frozen basis `51218e7a259aca7071244ea5d66fa68ac119d4518918d8ae317d5f2ab525e243`
(1,323 files), returning PASS with no P0–P2. Report:
ignored `.codex/reviews/admin-d03-core-result.md`; primary coverage and native
receipts: ignored `.codex/artifacts/core-delivery/`. The three nonblocking
traceability observations were closed by this final documentation transition;
`final-closure.json` binds its hashes and exact post-review delta. Earlier ledger
patches are intermediate evidence, not byte-exact final ledger reconstructions.
The initial preflight ledger preimage was not retained byte-for-byte; final
file hashes are authoritative for this uncommitted package.

This accepts local OE-T11 only. OE-T09/T10 Linux packaging, real preceding
worker upgrade/rollback/recovery and the Linux 23-case gate remain open; OE-T12
retains historical local acceptance. Reports live runtime was not run. Original
Chrome viewport restoration remains Not verified; excluded DevTools diagnostics
were replaced with IAB/native headless evidence. No commit, push or deployment.


### D-03 current-fixture Linux acceptance — 2026-10-24

Owner: current coordinator plus one Luna implementation/acceptance owner.
Consumers: release/packaging and acceptance owners. Revalidate affected claims
when source, fixture identity or trust inputs change; this receipt is scoped
to the fixed basis and is not a persistent environment-availability promise.
Basis: original HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` and the
preserved dirty source; frozen fixed fixture source
`d6446d808a249314dea1c87ae6b21b9ff675d34159d47301882d13370041e550`.
The user authorized a dedicated isolated x86_64 Linux build/service/update
window and excluded recovery of the missing genuine old 0.5.1 signed artifact.
All newly built versions are local fixtures. OE-A09/OE-T10 historical-worker
compatibility remains Not verified and still blocks an unconditional release.

Explicit core packaging, composition-aware CLI and fresh-install configuration
are implemented. Core/full signed packages built and installed on real systemd
in Colima `rz-admin-d03-amd64`. CLI lifecycle and ordinary reboot, update and
reverse application, health failure rollback, cross-composition refusal and
occupied-root refusal have native evidence. A real restart-limit failure left
the restored Admin unable to start; bounded `reset-failed` on journal-recorded,
composition-whitelisted units fixes that path, and native core/full faults now
restore healthy old services. Source recovery tests passed 4/4.

All 18 frozen A11 interruption points were actually observed with verified
worker identities, SIGKILL and changed boot IDs: core six, full twelve. Database
oracles follow the existing journal contract: restarted services match verified
backups; untouched services preserve captured pre-reboot state. Every table is
in scope. Full cases cover 53 tables, owner/mode/path and ten runtime files.
SQLite logical digests and physical file hashes are distinguished; normal
startup and pre-interruption manifest/audit writes are separately reported.
Packaged recovery child identity is inferred where journald lacks a child PID/
ELF. The core pre-journal phase and two-encoding bridge qualifications remain
explicit in the report. Failed instrumentation attempts are retained rather
than counted as passing cases. Final normal runtime has empty task DropInPaths,
correct signed binaries and four healthy endpoints.

Linux Monitoring's original 23-case gate and local evidence publication
passed on fixed source (outer exit 0, 2026-10-24). The two additional delivery
UI runs use synthetic gap-state data through real login/RBAC/API; they do not
repeat the historical Monitor-to-Admin relay acceptance. An earlier browser run completed assertions but
failed publication; it remains failed. The harness repair binds four loading
barrier artifacts and their hashes to the actual run. Final ZCode review passed on 2026-10-24 (no P0–P2; four nonblocking
P3 suggestions/observations). The coordinator verified actual CLI exit0,
provider SID/turn/trace, glm-5.3 model usage and final assistant stop, completion
marker, source identity and the returned findings. Native/request recovery
qualifications remain unchanged; this acceptance does not clear the old-worker
release gate. Necessary sanitized evidence, build/source overlays, per-case
coordinator readbacks and the final report are retained in ignored
`.codex/artifacts/core-delivery/linux/`; provider artifacts use ignored
`.codex/reviews/admin-d03-linux-*`. No commit, push, production access or goods
VM operation is authorized or claimed.

The dedicated Linux profile and data were deleted after evidence retention.
Both goods VMs and the global Docker context retain their prior state. The
review result and coordinator verification are in `.codex/reviews/admin-d03-linux-*`;
only factual closure-ledger changes follow its frozen source basis.
