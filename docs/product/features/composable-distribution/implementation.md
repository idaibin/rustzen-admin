# Composable distribution implementation plan

Status: execution started; complete distributions are not yet implemented.
The native producer reads stable non-link input bytes, emits units through
`nativeUnitBytes`, and never copies configuration values.
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

P8d is the source-build certificate publication slice. It consumes only the
validated Monitor container snapshot and a reread signed release triplet, then
atomically writes one canonical certificate. It does not change installation,
`current`, systemd, browser, load or deployment state.

P8e consumes the issued certificate capability only to admit one already
published certificate into a disposable Linux/amd64 Monitor runtime gate. It
does not extend the certificate or change its four literal later-layer flags.
The gate separately captures canonical runtime evidence after `rz verify`,
dry-run, fresh apply, status and server activation. A certificate is not an
installation authorization and no `current` pointer or operator host changes.
The verifier takes the current export root, release-result JSON, certificate,
public key, independently supplied expected source identity and a caller-selected fresh output directory as exact arguments. It
contains no P8d pointer filenames or reusable output path and never removes an
existing output. It runs the existing published-certificate verifier before
creating that directory or calling Docker, and rejects any input/output ancestry overlap.

`rz verify` reads every archive payload member needed by the contract verifier.
For server artifacts it parses canonical `contracts/web/binding.json`, checks
its composition and Web digest against the release manifest, validates the
selected API digest shape, replaces the sole
matching HTML digest stamp with the fixed slot in memory, then hashes the sorted
`{path,sha256}` Web table with the same algorithm as the TypeScript producer.
The earlier source-build/export admission owns the byte-level selected API check.

For one current export, use the existing producers in this order (all output
paths must be new):

```bash
read -r source_head source_state source_tree < <(scripts/admin-browser-source-identity.sh)
expected_source_identity="git:$source_head tree:$source_tree state:$source_state"
pnpm dlx bun@1.3.14 scripts/distribution-publish-monitor-container-release.ts --selection distribution/fixtures/monitor.json --export-root "$export_root" --expected-source-identity "$expected_source_identity" --output-base "$release_output" --private-key "$private_key" --public-key "$public_key" --key-id "$key_id" > "$release_result"
release_root=$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).root)' "$release_result")
pnpm dlx bun@1.3.14 scripts/distribution-issue-source-build-certificate.ts --selection distribution/fixtures/monitor.json --export-root "$export_root" --expected-source-identity "$expected_source_identity" --release-root "$release_root" --public-key "$public_key" --key-id "$key_id" > "$certificate_result"
certificate=$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).path)' "$certificate_result")
scripts/verify-monitor-native-runtime-linux-amd64.sh --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$expected_source_identity" --output "$runtime_output"
```

## Milestones and responsibility

| Stage | Concrete implementation | Exit evidence | State |
| --- | --- | --- | --- |
| P0 | Refresh current source/dirty basis and behavior assertions | Existing five application test suites pass; current coupling documented | Baseline captured; backend suites passed |
| P1 | Finite catalog, preset/custom resolver, composition identity, test-only fixture isolation, release refusal until real producers exist | Deterministic CLI tests; monitor excludes notifications; invalid selections fail | Implemented; 17 tests pass on pinned Bun; independent review fixes verified |
| P2 | Minimal Admin composition, selected registry/config, essential account/role UI contract and final selected schema | Full regression plus minimal auth/API/DB tests; absent owner never initializes | Implemented; source/build gate passes |
| P3 | Controller/Agent binary separation; positive Cargo closure; code-derived selected contracts; Web/schema generation | Per-binary feature evidence, selected queries, full/monitor Web/API/schema negative tests | Implemented for the Monitor selection and independently reviewed. The selected Web inventory includes every local import required by its access and Monitor routes. |
| P4 | Monitor native producer, publication and server activation | Composition-qualified Admin/Monitor/Agent binaries with selected Web, selected API/schema/config/protocol descriptors, signed exact-member tar, immutable publication, root-only server activation and PID1 owner-login proof | Implemented for the Monitor server selection. `rz apply` publishes the immutable payload; `rz activate-monitor-server` validates configuration and excluded-service residue before writes, initializes and journals two fresh identity-bound databases, installs only selected units, starts/enables `rz.target`, verifies owner, signed schema/data identities, exact migration/schema inventory, MainPID executables and bound health, and records readiness. The arm64 systemd PID1 gate covers nine recoverable activation and durability faults, exact retry, both service start orders, restart, unsafe/existing paths, completed-state mutation, same-schema foreign DB rejection, and omitted-service residue. |
| P5 | Optional Admin inbox, current recipient authorization, sequence-bounded reads and retention budgets | Fresh SQLite transaction/concurrency/retention tests | P5a and P5b implemented and verified locally: selected dual-ledger schema and authenticated reads plus durable accounting, bounded retention, atomic admission, startup/periodic maintenance, pressure refusal and true two-pool concurrency. Producer delivery and realtime invalidation remain P6/P7. |
| P6 | Monitor and Reports optional outbox/relay, trusted initiator, fenced claims and bounded ambiguity | Lifecycle/crash/dedupe/recipient tests | P6a Monitor incident delivery and P6b Reports terminal publication are **Closed locally / Passed** by their source-bound disposable `linux/arm64` Colima gates, including selected/pure process, schema, transaction, relay, authorization and negative-artifact evidence. |
| P7 | Direct Admin fetch-SSE and shell integration | Stream/parser/cancellation tests and real browser/proxy reconciliation | **Closed locally / Passed.** P7a auth/session authority and P7b backend SSE passed source, independent-review and disposable `linux/arm64` runtime gates. P7c adds the notifications-owned durable inbox shell and one authenticated fetch-SSE lifecycle. Its final browser manifest is `target/rz/message-center-browser/runs/20260908T034234Z-91415/manifest.json` (SHA-256 `abf2d85978f080c339bce8498205cdb650303009ea104da7dc68051426afdd77`), binding dirty source tree `c407429cf9cc88ddf340540f3f656566417521adc22fdb075bf4ab3de21eb9d6` to six Linux binaries, selected/pure Web inventories and 18 exact receipts. Eleven Chromium journeys prove desktop/mobile layout, durable loading/empty/detail/paging/read states, one browser-origin SSE lifecycle per journey, event-driven count/list reconciliation, stream-401 login recovery, 403 clearing and in-app Monitor incident deep linking. Direct and same-origin-proxy SSE return `200 text/event-stream` with Bearer headers and no URL secret. The pure Monitor composition has no notification-owned Web module, schema object, API route, ingress listener or service owner. Global-1,000 sustained load, native systemd, production reverse-proxy and production deployment remain **Not verified** for P8 or target acceptance. |
| P8 | Certify each shipped selection and current product journeys | Measured artifact, native runtime, browser, load and absence report for exact build | P8a readiness admission is implemented: every named catalog preset has a canonical fixture and an exhaustive source/build producer audit. It admits only `monitor` and `node-agent` to later certification commands; `monitor-notify`, target `full`, `analytics`, `reports`, test-only regression and every `custom` selection fail closed. P8b's Linux/amd64 Monitor container export and host-side stable-snapshot validator are **Closed locally / Passed** at `target/rz/p8b-container-export-verified`: the source identity was unchanged before and after BuildKit, the host validator accepted the exact export before and after a restricted Linux runtime pass, six binary outputs matched the exported API, config and protocol contracts, and the seventh Agent config witness passed its structural policy. P8c signs retained captured bytes into an immutable three-file release triplet and rereads it with independent trust material. P8d derives and atomically publishes the source-build certificate from the retained snapshot and that verified triplet; its synthetic Linux gate and the current Monitor export/release certificate issue pass locally. Installation, systemd, browser and load gates remain later P8 work. |

The first current-full-regression artifact is an internal test fixture. It must
not be labeled or published as target full, which includes notifications. A
resolved selection is only a plan: until actual Rust/Web/schema/packaging
producers pass, release readiness is false and release requests fail closed.

### P8 certification admission matrix

P8 starts with a deterministic admission check so an incomplete preset cannot
reach an expensive build and later be mislabeled as certified. The audit resolves
every preset in `distribution/catalog.json` and asks each real Cargo, Web, API,
schema, config, native-layout and protocol producer whether it supports the exact
capability closure and composition identity. `custom` always requires its own
exact assessment and is rejected by this first slice, even when its capability
closure happens to match a named preset.

| Selection | Current producer evidence | P8a result | Remaining certification boundary |
| --- | --- | --- | --- |
| `monitor` | Selected Admin/Controller Cargo, Web, API/schema/config, protocol and native-layout producers; signed apply and Linux activation gates exist | Admitted to source/build certification | One fixed clean build basis and final acceptance report |
| `monitor-notify` | Selected Admin/Controller Cargo, Web, API/schema/config and notification runtime/browser gates; the Monitor protocol is unchanged | Rejected | Merge the access and notifications config owners into one selected native Admin layout, then produce the composition-bound source/build report |
| `node-agent` | Agent Cargo, config, protocol, native layout, signed apply and controlled Linux pairing evidence | Admitted to source/build certification | Final PID1/service restart and shipped-target report |
| `analytics` | Catalog and application baseline only | Rejected | Selected Admin/Insights Cargo boundary, Web graph and contracts |
| `reports` | Catalog, Reports application and notification producer slices only | Rejected | Selected Admin/Reports Cargo boundary, Web graph and contracts |
| target `full` | Default full regression builds and journeys exist, but optional Admin/Insights/Reports selected producers are incomplete | Rejected | Exact target-full selected producers and its own full acceptance report |
| `current-full-regression` | Internal test fixture | Rejected | Permanently ineligible for production certification |
| `custom` | Resolver closure only | Rejected | Explicit producer assessment and all gates for that exact composition |

Admission means only that all producer families required by the source/build
gate are present. It does not change `producerReadiness`, open the production
release gate, produce a native target, sign an archive, install, deploy or prove
a product journey.

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

The Monitor native producer embeds only the verified composition-qualified copy at `apps/admin/selected-web/<compositionId>/dist`; the generic `apps/web/dist` is used only by the full composition. The Admin build script rejects a missing selected inventory, a composition mismatch, or an absent selected `index.html`. Docker validates `DISTRIBUTION` as exactly `full` or `monitor`. Monitor physically emits server `rz-admin`/`rz-monitor` to `release/server/bin` and Agent `rz-monitor-agent` to `witness/bin`; full retains the five-binary `/out/bin` output. The Monitor branch builds and verifies selected Web before copying it to that embedded directory and compiling the Admin binary.

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

Monitor P4 protocol descriptor is implemented: Controller and Agent produce the
same canonical wire descriptor/digest for offline pairing. The selected protocol
artifact verifies both command outputs against its reviewed golden and supplies
manifest/build identity directly; archive/signature/install remain closed.

Monitor P4 selected API export is implemented and verified: contract-only Admin
and Monitor commands expose real selected registrations before startup inputs.
The composition-qualified producer rejects deviations from the reviewed route
corpus, and the release manifest binds the verified artifact bytes. Schema and
configuration artifacts are implemented as separate selected contracts.

P8f-A implements the selected-Web build binding: the Monitor producer stamps
only its generated HTML after deriving the normalized file-table digest, writes
the canonical descriptor and records final file hashes in inventory v2. The
container export carries descriptor and selected API source bytes; its host-side
validator rejects a changed descriptor, inventory, API source, stamped HTML or
emitted asset before native staging/release derivation. HTTP bootstrap and
browser behavior remain later closures.

P8f-B implements the Monitor-distribution HTTP bootstrap boundary. Admin validates
and embeds the P8f-A binding, rejects a packaged/runtime composition mismatch before
listening, serves anonymous `GET /__web-binding` and authenticated
`GET /api/installation` with no-store caching, and keeps both routes out of Full
until Full has an equivalent packaged binding. Selected HTML contains a dependency-free
inline bootstrap instead of an eager module tag; it validates the anonymous response
before dynamically loading the content-addressed entry with SRI and applies one bounded
cache-busting recovery attempt. The Admin build gate validates the exact 11-field
inventory-v2 schema and canonical metadata, sorted route/asset/module/emitted/file
tables, the actual file set and bytes, selected API digest and normalized Web digest.
`scripts/verify-selected-web-bootstrap-browser.py` is the executable P8f-B
Chromium gate. It requires a fresh output path plus an explicit live selected-Admin
`--admin-url`; it starts only loopback fault fixtures and headless Chromium, never a
user-facing browser. Its fixture records every pre-entry request and
exercises the success, binding-mismatch, network, SRI/entry-failure and retry paths.
It publishes a canonical receipt only when the anonymous binding is the first API or
JavaScript request after the document/static assets, carries neither its fixture
proof cookie nor authorization, and the successful authenticated installation
inventory has the same digest. Failures request no API at all, except the anonymous
binding, and never execute the business entry; the SRI case may request its entry
but still must not execute it. The receipt separates the one automatic cache-busting
reload from a manual Retry navigation, which must return to the canonical deep link
without the cache-busting parameter. It also binds Chromium version, Admin health
identity, HTML/binding/installation digests and every case assertion to that run.
The gate reads the owner secret only from a caller-supplied mode-`0600` file and
never accepts it as an argument, stores it in evidence, or prints it. Its SRI
fault payload has a unique execution side effect; the SRI case fails if that marker
appears, while the sensitivity fixture removes bootstrap integrity and must execute
the same marker so the gate proves that its SRI assertion can detect execution.
After every case, including manual Retry's separate one-reload budget, the gate
rereads health and writes a compact sorted canonical JSON receipt.
The secret reader opens its `0600` current-user regular file once with no-follow
semantics and validates/reads that same descriptor. CDP transport reads complete
handshakes and frames under deadlines, while browser state uses bounded polling.
Each case records equal before/after selected
health binding identities, including the SRI sensitivity negative case.
The module table is checked by selected owner, safe path, allowed dependency and
required generated/source roots. Its complete JSON byte digest is not fixed because
host and fresh Linux Vite graphs may contain different allowed module entries.
Its mutation harness invokes the real Monitor `cargo check` for every metadata class
and restores a clean positive build. Source and focused tests close generation, route,
authentication, cache and failure-state behavior. A newly derived signed artifact and
external Chromium runtime remain the P8f-B acceptance evidence rather than reusing a
pre-change P8d/P8e tuple.

The generic server-staging entry applies the same complete selected-Web policy
and derives its build-route identity from the verified inventory. It cannot
publish a self-consistent Full or Reports bundle under a Monitor selection.

Monitor P4 selected schema export is implemented from the two authoritative
fresh-install migrations. The manifest derives schema fingerprints and
data-contract IDs from the canonical artifact and binds its byte digest into the
build identity. Archive, signature and installer production remain P4 work.

The three selected Config descriptors and contract-only CLI commands are now
implemented. They are feature-specific, deterministic without environment or
working-directory state, and contain no configuration values. Server and Agent
canonical config artifacts are now produced from those outputs; the manifest
derives and binds their byte digest instead of accepting `configDigest` input.

The `monitor-notify` selection additionally obtains the notifications descriptor
from the actual `rz-admin` binary. It owns inbox capacity, filesystem reserve,
WAL-pressure thresholds, the dedicated ingress port, current signing key and the
optional previous-key tuple with its at-most-120-second cutoff. Monitor owns the
exact loopback ingress URL and current producer key. Pure `monitor` has none of
these fields. The full/default Monitor Cargo and Docker build selects
notifications; the explicit controller-only build remains the negative artifact.

The P6a Linux gate uses fresh Admin and Monitor databases and real loopback TCP
between the selected processes. It proves open/duplicate/resolved delivery,
durable Admin receipt/inbox state, invalid producer authentication rejection and
the pure Monitor negative artifact. The primary agent's final Colima run
published `target/rz/monitor-notification-runtime/current/manifest.json`
binding the executed source tree, four feature-selected binary hashes, build
provenance and verifier identity.
P6a is **Closed locally / Passed**. Native systemd, production deployment,
Reports, SSE, UI and sustained load remain outside this gate.

P6b is **Closed locally / Passed** at the source, focused-test and disposable
Linux process layers. The default Reports build selects its independent outbox
ledger, relay, transport, configuration and authorized delivery diagnostic; the
explicit no-default build proves those owners absent while retaining nullable
run provenance. Full
Admin accepts Reports events through producer-scoped keys and recomputes the
single persisted initiator against current access in the admission transaction.
The final `linux/arm64` Reports-to-Admin gate is published at
`target/rz/reports-notification-runtime/current/manifest.json`. Its manifest
records 25 exact receipts, six manual terminal classes, immutable retry
initiator, scheduled silence, outage backfill, duplicate reconciliation,
authentication denials and complete selected/pure runtime identity evidence.
Native systemd, production deployment, SSE, UI and sustained load remain
unverified.

The selected-native layout producer now emits and verifies canonical Monitor
server and node-agent unit/config manifests. The release manifest derives their
byte digest and binds it into the build identity. It deliberately does not alter
the legacy deployment templates or installer; archive, signing and installation
evidence remain P4 blockers.

P8b captured-byte staging is implemented locally: host-verified snapshot bytes
produce the private Monitor server staging tree and its unsigned release manifest
without rereading the export root. It remains before signing, release
publication, installation, systemd, browser and load evidence.

The Chromium gate is admitted only through the existing P8e published
source-build certificate verifier. It requires the export root, release result,
certificate, trusted public key, expected source identity and the running
`rz-admin` binary. Before any evidence directory is made, that verifier derives
and checks the release build/composition identity and the certified Admin binary
digest; the gate then rejects different health, installation or stamped Web
digests and hashes the binary again after the run. The receipt records the
verified release tuple and a canonical path/SHA-256 table for the gate, fixture,
case table, admission helper and certificate verifier sources. Binding mismatch
and network failure have a cumulative zero-entry-JavaScript assertion across the
initial navigation, its automatic reload and manual Retry; the SRI entry-load failure remains allowed to request its
integrity-bound entry, but it must never execute it. The separate SRI
sensitivity receipt records its requests, marker and health identities before
and after the deliberately removed integrity attribute. Focused fixture tests inject
eager entry receipts for both binding failures and exercise the same runtime request
boundary used by the Chromium gate.

Admission also derives the repository's current Admin-browser source identity with
`scripts/admin-browser-source-identity.sh`, strictly parses its three tab-separated
`HEAD`, `state`, and tree-digest fields, normalizes them as
`git:<HEAD> tree:<digest> state:<state>`, and requires that value to equal the supplied
expected identity. The gate records both values and includes that script in the
provenance table. It repeats the complete admission after Chromium closes and
rejects any canonical admission change before the sole evidence-directory write.

For containerized Admin runs, `--runtime-container` adds a runtime attestation
before and after Chromium. It fixes a running Linux/amd64 container ID and the
unique Admin host-port-to-container-port mapping, resolves the container-side
LISTEN socket inode from `/proc/net/tcp` or `/proc/net/tcp6`, and accepts exactly
one owning PID found through `/proc/*/fd`. Its executable SHA-256 must match the
certificate-admitted `rz-admin` binary; the observed PID, device and inode are held
stable only by the before/after runtime-tuple comparison. Both tuples must be equal
before the receipt is published.

P8g implements the pure-Monitor exact-artifact load gate. It revalidates the
complete P8e signed release/runtime evidence set, then independently binds the
current P8f Admin tuple and current certified Monitor binary; historical P8e
process IDs are not required to equal disposable P8f container processes. It uses the documented fixed 100-node/four-disk
sentinel corpus through the Admin gateway, and publishes a canonical receipt
only after both measured lanes, resource checks, controlled Monitor outage and
recovery complete. The generic notification SSE capacity target remains an
explicit pure-Monitor N/A with signed absence and runtime 404 evidence.
The controlled outage freezes Admin after admitting four Monitor-side in-flight
sockets, kills the frozen Monitor MainPID, confirms the old PID is absent, and
records that confirmed-dead boundary before stopping Monitor and thawing Admin.
Its dedicated requests use a ten-second timeout so the observer can prove their
responses finish after confirmed death; normal load requests retain two seconds.
Every load, drain, quiet, fault and recovery phase records exactly two resource
snapshots: immediately before work and immediately after work. The work duration
therefore excludes both Docker probes and has no periodic sampling cadence.
Readings preserve current and cumulative PID values separately. Non-fault phase
snapshots require unchanged Admin and Monitor owners; fault snapshots require
the documented Monitor restart and unchanged Admin owner. The parser rejects
missing, additional, reversed or too-short snapshot pairs and peak/current
substitution.
The Monitor Nodes handler uses one process-local 250ms serialized immutable response snapshot with a
single refresh owner. Accepted reports and successful policy mutations invalidate
it; cache hits never skip the Admin gateway's authoritative authorization. The
shared database pool default is one minimum and eight maximum connections; the
eight-connection maximum is the measured default for the certified four-CPU,
512MiB pure-Monitor runtime.
