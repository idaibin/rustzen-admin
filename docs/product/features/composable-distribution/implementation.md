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
