# Full distribution implementation plan

Status: active.

The first release has one implementation path and one acceptance boundary:

```text
build complete bundle -> copy to server -> install -> start -> four services healthy
```

## Optional examples stages

Status: planned; no stage is implemented by this document edit. Existing full
closure below is baseline evidence only. Follow [target contract](spec.md#optional-examples-target-approved-not-implemented)
and [OE acceptance](validation.md#optional-examples-acceptance). Reuse the current
branch and local 2bbd0c6; preserve later independent work and stop on overlap.

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

The complete bundle contains the Web application, `rz`, Admin, Monitor, Insights,
Reports, production configuration templates, the six systemd units and the installer.
It has one version, one Ed25519 signature, one update transaction and one rollback
boundary. Frontend and backend digests are inspection fields inside that one signed
identity.

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
  `apps/web/src/api/index.ts:5-8` barrel; the dashboard module panel is
  `apps/web/src/routes/index.tsx:141-234` over
  `GET /api/dashboard/modules`.
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
the archive contents and mismatches must be rejected before any write; the
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
no Reports env and no controller identity) and rejects, before any write:
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
