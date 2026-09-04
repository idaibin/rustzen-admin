# Composable distribution implementation plan

Status: execution started; complete distributions are not yet implemented.
Authority: the user's implementation request and the current root AGENTS.md.
The reviewed design supplies the target composition; this plan supplies the
implementation order, present evidence and current-rule corrections.

## Execution rules

Work in the existing checkout. One implementation owner writes a bounded slice;
the coordinator owns integration and one independent reviewer reviews fixed
results. Do not create worktrees, erase unrelated changes or modify live databases.
Commit, push, installation on an operator host and release delivery are separate
actions. Test builds and disposable test fixtures are implementation validation.

The current repository rule requires one final fresh initialization baseline.
It supersedes the earlier design's historical-upgrade/rollback-compatibility
suggestions: no old-data readers, conversion, sequential upgrade migration or
rollbackCompatibleBuildIds will be implemented. Same-build interrupted fresh
installation may resume from its own verified journal. A different build or
composition uses a new root and fresh databases, preserving the previous root.
Restarting the same installed build is not a historical migration. Existing
legacy migration/rollback tests are baseline observations, not authorization to
reproduce those behaviors in the new distribution pipeline.

## Milestones and responsibility

| Stage | Concrete implementation | Exit evidence | State |
| --- | --- | --- | --- |
| P0 | Refresh current source/dirty basis and behavior assertions | Existing five application test suites pass; current coupling documented | Baseline captured; backend suites passed |
| P1 | Finite catalog, preset/custom resolver, composition identity, test-only fixture isolation, release refusal until real producers exist | Deterministic CLI tests; monitor excludes notifications; invalid selections fail | Implemented; 17 tests pass on pinned Bun; independent review fixes verified |
| P2 | Minimal Admin composition, selected registry/config, essential account/role UI contract and final selected schema | Full regression plus minimal auth/API/DB tests; absent owner never initializes | Implemented; source/build gate passes |
| P3 | Controller/Agent binary separation; positive Cargo closure; code-derived selected contracts; Web/schema generation | Per-binary feature evidence, selected queries, full/monitor Web/API/schema negative tests | Controller/Agent prerequisite implemented and independently reviewed. Monitor selected-Web producer is implemented; selected server/schema producers remain open. |
| P4 | Monitor native producer and fresh install | Composition-qualified Admin/Monitor/Agent binaries with selected Web, selected API/schema/config/protocol descriptors, signed exact-member tar, installer layout verification | In implementation. Canonical manifest producer/validator is implemented separately; archive/signature/installer evidence still keeps the gate closed. Monitor recovery is only same-tuple fresh-root journal continuation, not a service, DeployService extraction, rollback, or DB restore. |
| P5 | Optional Admin inbox, current recipient authorization, sequence-bounded reads and retention budgets | Fresh SQLite transaction/concurrency/retention tests | Queued after P2/P3 |
| P6 | Monitor and Reports optional outbox/relay, trusted initiator, fenced claims and bounded ambiguity | Lifecycle/crash/dedupe/recipient tests | Queued after P5 |
| P7 | Direct Admin fetch-SSE and shell integration | Stream/parser/cancellation tests and real browser/proxy reconciliation | Queued after P5/P6 |
| P8 | Certify each shipped selection and current product journeys | Measured artifact, native runtime, browser, load and absence report for exact build | Queued after applicable stages |

The first current-full-regression artifact is an internal test fixture. It must
not be labeled or published as target full, which includes notifications. A
resolved selection is only a plan: until actual Rust/Web/schema/packaging
producers pass, release readiness is false and release requests fail closed.

## Source grounding refreshed for execution

The current source baseline contains 497 tracked/untracked paths excluding the
new delegated catalog/resolver paths. Its content hashes are in the ignored
execution ledger. This is separate from the older ten-round design basis.

| Boundary | Present implementation | Required change |
| --- | --- | --- |
| Build | Root justfile and Docker build all five binaries and full Web | Resolve selected products and verify actual outputs |
| Admin entry | infra/app.rs initializes task/deploy, all module state, logs and public file serving | Separate access from optional owners |
| Agent (original basis) | Monitor main.rs combined controller and agent; collector shared monitoring.rs with SQL handlers | Separate targets implemented in the first execution batch; native certification remains pending |
| Routes | Admin ContractRouter; module ModuleRouter; all Web files discovered by TanStack | Keep Rust route authority and select Web discovery input |
| Admin schema | 12 initial tables, including dicts/log/task/deploy/module presentation | Feature-owned selected final baseline; explain unused-object removal |
| Monitor schema | 9 monitoring tables | Preserve accepted-report/incident behavior; optional outbox only when selected |
| Insights/Reports | Initial SQL plus existing sequential collection/schedule SQL | Fold final fresh schema when that owner is touched; do not preserve historical upgrade paths |
| Native delivery | Existing fixed four-service units, installer and release worker | Selected inventory, fresh-root admission and bounded interrupted-install recovery |

The independent current behavior oracle starts from existing tests, not the new
catalog. It includes login/account/grants, module gateway rejection and route
contracts, Monitor report fencing/incidents, Insights tracking rejection/query,
Reports creation/cancellation/scheduling, CLI JSON and signed bundle validation.
UI and target-host behavior still need their own later evidence; backend tests
do not certify those layers. Catalog coverage cannot replace these assertions.

## Initial verification

Executed before application refactoring:

```sh
cargo test -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports -p rustzen-cli -- --test-threads=1
```

Result: exit 0. These are current application unit/in-process integration tests,
not selected-artifact, live browser, host installation or capacity acceptance.
P1 commands:

```sh
just verify-distribution-selection
just distribution-validate
just distribution-plan
just distribution-release-gate
```

The first command passed 17 tests with 45 assertions on Bun 1.3.14. Validation
and resolution default to the monitor fixture. The release gate deliberately
returns a JSON error and a nonzero exit until real producers are certified.
Selection accepts the current native target names `x86_64-unknown-linux-musl`
and `aarch64-unknown-linux-gnu`; name validation does not certify a native build.
The independent review found invalid target acceptance and lost blocker reasons
when capabilities share a binary. Both were corrected and covered by focused
tests. The root also added the missing named node-agent preset and verified that
it resolves with no Web roots or schema owners.

P2 adds the mutually exclusive Admin compositions `full` and
`monitor-distribution`. The latter compiles only access/account/user/role,
selected navigation and the Monitor gateway. It does not compile the complete
embedded Web shell or its static-serving dependencies, does not register
dashboard, menu/status/module-management, deploy, task or operation-log routes,
and unmatched paths return JSON 404 instead of the SPA shell. Its embedded fresh
initialization owns only the access tables/views plus the selected
`monitor` module row. Full-only configuration fields and the direct task,
release and system-status dependencies are absent from the minimal composition.

```sh
just verify-monitor-admin
cargo test -p rustzen-admin -- --test-threads=1
```

Role management retains the read-only `/api/system/menus/options` permission
catalogue. Menu listing and mutation routes remain absent from the monitor
distribution.

The P2 gate passes five focused config tests, 37 minimal Admin tests,
warnings-denied Clippy, a minimal binary build and a direct dependency-boundary
check. Fresh-database behavior includes permission synchronization and an owner
login. The full Admin regression passes 126 tests with one benchmark ignored.
This verifies the backend composition and the declared essential Web root
contract; P3 still owns generation and physical verification of the selected
Web bundle and selected API artifact.

The Monitor native producer embeds only the verified composition-qualified copy at `apps/admin/selected-web/<compositionId>/dist`; the generic `apps/web/dist` is used only by the full composition. The Admin build script rejects a missing selected inventory, a composition mismatch, or an absent selected `index.html`. Docker validates `DISTRIBUTION` as exactly `full` or `monitor`. Monitor physically emits server `rz-admin`/`rz-monitor` to `/out/server/bin` and Agent `rz-monitor-agent` to `/out/agent/bin`; full retains the five-binary `/out/bin` output. The Monitor branch builds and verifies selected Web before copying it to that embedded directory and compiling the Admin binary.

The P3 Monitor Web producer resolves `distribution/fixtures/monitor.json`, copies
only the access and Monitor route allowlist into
`apps/web/.selected-web/<compositionId>/routes`, and gives that directory—not
`apps/web/src/routes`—to the TanStack generator. It writes its route tree,
temporary generator data and selected public directory below the same
composition-qualified workspace; Vite writes the artifact to
`target/distributions/<compositionId>/web/dist`. The generated entry uses the
access/Monitor API surface and a static Monitor/access shell; it has no full
API barrel or module-navigation query. `scripts/distribution-verify-web.ts`
records Vite's actual module IDs and emitted entries, then strictly compares
the generated inventory with the resolver's monitor plan. It rejects extra,
missing, stale, escaped or symlinked output paths; and any module ID which is
not exactly one of the selected generated source, an explicit repository-source
allowlist, an allowlisted dependency package, or Vite's fixed preload helper.
Before reading inventory or traversing output it rejects symbolic links at every
ancestor of the distribution root, Web root, dist root, generated root and API
adapter source. It also rejects Analytics, Reports, Admin-console and
management route/API sentinels. The
selected role page receives an options-only menu adapter, so menu list,
inventory mutation and delete APIs never enter the artifact. The verifier also
requires the access and Monitor API/path sentinels and the selected
`rustzen.png` asset. `apps/web/dist` remains the full-build output and is never
accepted as selected-artifact evidence. This producer closes only
the selected-Web blocker; selected Admin API export, selected schema, native
bundle, installer and runtime gates remain release blockers.

```sh
just verify-distribution-web
```

The Controller/Agent separation is independent of minimal Admin and was executed
early. `rz-monitor controller` is the default controller
target. `rz-monitor-agent` uses `--no-default-features --features agent`; the old
combined Agent command is rejected. The Agent's original collector and six
behavior tests were moved together, including filesystem/container filtering,
local bind-mount deduplication, retained network mounts, request timeout,
sequence fencing and missed-tick behavior. A rewritten collector in the first
delegated result omitted existing behavior and was replaced during integration.
There is one production collector implementation.

Focused verification:

```sh
just verify-monitor-agent
cargo test -p rustzen-monitor --no-default-features --features controller --bin rz-monitor -- --test-threads=1
cargo test -p rustzen-config --no-default-features --features monitor-agent
cargo test -p rustzen-config --no-default-features --features monitor-controller
bash scripts/test-setup-layout.sh
```

Agent: 12 tests; Controller: 47 tests; focused configuration: 8 Agent and 5
Controller tests; setup layout: 12 groups. Agent Clippy with warnings denied and
the host normal/build dependency check passed. That graph excludes Axum, SQLx,
rustzen-auth, rustzen-ipc and rustzen-storage. Shared protocol tests run in each
binary, so these are per-target counts, not distinct-case totals. The systemd
Agent unit points to the independent binary and remains outside the server unit
group and bundle. No server schema, Controller router or database startup module
is imported by the Agent entry point.

After the shared configuration change, the five application suites plus
rustzen-config passed 237 tests with one pre-existing ignored test. The Agent
test target is verified separately because the default Monitor package selects
only the Controller. The local host Agent executable was also built with its
isolated feature command. Linux packaging and live collection were not tested.

Independent review of the corrected prerequisite found no actionable P1/P2.
It verified the extracted production functions against the frozen pre-extraction
source, the migrated behavior tests, separate entry modules even with feature
unification, Cargo target metadata and the 149-package host dependency closure.
This review does not certify the later distribution milestones.

Selected Web pruning, notifications, signed pairing profiles and native
distribution certification remain pending. A host dependency tree is not
proof of a Linux release artifact or target-host operation.

No release artifact, installation or externally delivered result is claimed.
