# Composable distribution implementation plan

Status: execution started; complete distributions are not yet implemented.
Analytics P8b step 2 supplies only an Analytics host-synthetic export, immutable snapshot and
captured native-staging seam for the exact Analytics selection. It remains rejected
from Docker/container, certificate, signing, publishing, installer and runtime work.
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
validated `monitor` or `monitor-notify` server container snapshot and a reread signed release triplet, then
atomically writes one canonical certificate. It does not change installation,
`current`, systemd, browser, load or deployment state.

P8e now takes an explicit reviewed server selection and preserves that preset
through published-certificate admission, runtime evidence and revalidation.
The selected notification activation input supplies its ingress/event-key values
and records its notification check while retaining only Admin and Monitor units;
pure Monitor rejects those keys and records an absent ingress. The retained
monitor-notify Linux/amd64 runtime record is
`target/rz/p8e-monitor-notify-native-runtime-20260910-r2/monitor-native-runtime-evidence.json`
(SHA-256 `aa15c4985ff5ab16b6dd5ef52bd7bbd6709724cfdb5193b2e1b678ebb4710921`).
It binds the old dirty source identity
`git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty`,
composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`,
build `d2c4e0f452c7400927059bc499822ef1218a9c47547ca88eb25706bbfa1426bd`
and certificate `8eba6380fe0e130288d0038d561b9a61a07abdbaad3ef049068164c3aefd1436`.
A fresh install passed PID1 activation, restart and both start orders, health and
MainPID bindings, owner login/default-password rejection, and Insights/Reports
absence. The unauthorized ingress probe returned `401` with `bad-producer`.
Its literal runtime flag is true; browser, load and `releaseReady` remain false.
It is not a rebuild of current HEAD and does not prove browser, load or deployment.

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
scripts/verify-monitor-native-runtime-linux-amd64.sh --selection distribution/fixtures/monitor.json --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$expected_source_identity" --output "$runtime_output"
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
| P6 | Monitor and Reports optional outbox/relay, trusted initiator, fenced claims and bounded ambiguity | Lifecycle/crash/dedupe/recipient tests | P6a Monitor incident delivery is **Closed locally / Passed** by its source-bound disposable `linux/arm64` Colima gate. P6b Reports terminal publication is **Closed locally / Passed** by its source-bound disposable `linux/amd64` Colima gate. Both retain selected/pure process, schema, transaction, relay, authorization and negative-artifact evidence. |
| P7 | Direct Admin fetch-SSE and shell integration | Stream/parser/cancellation tests and real browser/proxy reconciliation | **Closed locally / Passed.** P7a auth/session authority and P7b backend SSE passed source, independent-review and disposable `linux/arm64` runtime gates. P7c adds the notifications-owned durable inbox shell and one authenticated fetch-SSE lifecycle. Its final browser manifest is `target/rz/message-center-browser/runs/20260908T034234Z-91415/manifest.json` (SHA-256 `abf2d85978f080c339bce8498205cdb650303009ea104da7dc68051426afdd77`), binding dirty source tree `c407429cf9cc88ddf340540f3f656566417521adc22fdb075bf4ab3de21eb9d6` to six Linux binaries, selected/pure Web inventories and 18 exact receipts. Eleven Chromium journeys prove desktop/mobile layout, durable loading/empty/detail/paging/read states, one browser-origin SSE lifecycle per journey, event-driven count/list reconciliation, stream-401 login recovery, 403 clearing and in-app Monitor incident deep linking. Direct and same-origin-proxy SSE return `200 text/event-stream` with Bearer headers and no URL secret. The pure Monitor composition has no notification-owned Web module, schema object, API route, ingress listener or service owner. Global-1,000 sustained load, native systemd, production reverse-proxy and production deployment remain **Not verified** for P8 or target acceptance. |
| P8 | Certify each shipped selection and current product journeys | Measured artifact, native runtime, browser, load and absence report for exact build | P8a readiness admission is implemented: every named catalog preset has a canonical fixture and an exhaustive source/build producer audit. It admits `analytics`, `monitor`, `monitor-notify` and `node-agent` to later source/build certification commands; target `full`, `reports`, test-only regression and every `custom` selection fail closed. Analytics keeps its host-synthetic P8b export/snapshot/captured-staging seam with `host/<platform>/<arch>` metadata and closes the real Linux/amd64 BuildKit container export with six restricted contract-command bindings at `target/rz/p8b-analytics-container-20260911T020403Z`. Its P8c/P8d signing and source-build certificate are closed at `target/rz/p8cd-analytics-evidence-manifest-20260911.json`: build `e10b0fd6f289f5035b4d4147d0b47f77be6eeadfa1d419ede6eb05570aa2f333`, archive `0eae6e53a591c740a66051f6d32a8f836ddecfae73b417774d66fec0b68a3297`, manifest `a87435008418c4cce05ec15ec7807673b4c2984aede41dd7178ac88d81bb9fad`, envelope `3a045610c2328657e94724651fcb9cd0daeab3f3d98cbcae8e57ccabb7dbd183`, certificate `221f07f0fc5517f32dff3b56dfbb22189fc9a08c06d106a6a2f026e053e24583`, and byte-identical host plus linux/amd64 `rz verify` JSON at SHA-256 `b718e05cba0b16f67997ff7f4e0aee17750e56a204196c48c1ebf75633221d79` against public key `9596184838e7a636ff8816a70c4a198428d8fd5840f5d9c5bd79a164373d74e0`; the private key was deleted. The later P8e/P8f chain (fresh Insights schema) closes native runtime and real-browser journeys; load and release readiness remain rejected. `monitor-notify` now has a Linux/amd64 exact container export at `target/rz/p8b-monitor-notify-export-20260910T065139Z`, bound to dirty source identity `git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty` and composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`. Its 50-file export passed the host validator before and after the restricted run; `target/rz/p8b-monitor-notify-export-20260910T065139Z-contract-checks/manifest.json` records nine restricted commands (eight retained selected contract checks plus one Agent config witness) with network disabled, read-only filesystem and all capabilities dropped. P8c/P8d now have exact old-export monitor-notify evidence at `target/rz/p8cd-monitor-notify-evidence-manifest-20260910.json`: the retained build is `d2c4e0f452c7400927059bc499822ef1218a9c47547ca88eb25706bbfa1426bd`; the signed archive, manifest, envelope and certificate SHA-256 values are `92a5c59f0dcc8b34058f785a840187212a7cb7eb53a656d338010a7b183fa4ce`, `c74e970ca5729e5e985b233b7153d604a66866e855c720c2132e758e8c1f1b5b`, `806ee234f003f815a86b230b5665e6ec7b5b83a67c2d7bfd3944d7599aca60c7` and `8eba6380fe0e130288d0038d561b9a61a07abdbaad3ef049068164c3aefd1436`. Host and Linux/amd64 verifier JSON are byte-identical at SHA-256 `19acf4cb9484f42f618ccf87a0e164ccc949f922bd29487f71ec2d854d0e63a3`; the private key was deleted and only public key `04db0497facd7cf34e8e0cb8821a386c852bf947275a47a6465263165bc31d8e` remains. This is exact evidence for that old export, not a rebuild of current `0d99f28` source. Its `runtime`, `browser`, `load` and `releaseReady` flags are all false, so native installation/PID1, browser and load remain **Not verified**. P8b's Linux/amd64 Monitor container export and host-side stable-snapshot validator are **Closed locally / Passed** at `target/rz/p8b-container-export-verified`: the source identity was unchanged before and after BuildKit, the host validator accepted the exact export before and after a restricted Linux runtime pass, six binary outputs matched the exported API, config and protocol contracts, and the seventh Agent config witness passed its structural policy. P8c signs retained captured bytes into an immutable three-file release triplet and rereads it with independent trust material. P8d derives and atomically publishes the source-build certificate from the retained snapshot and that verified triplet; its synthetic Linux gate and the current Monitor export/release certificate issue pass locally. R9 closes the current monitor-notify business browser journey at `target/rz/p8fb-r9-monitor-notify-export-20260910T125008Z`, bound to source `git:cb357c671b88ecfaf5987a30b7de39f0ef125a0d tree:a419bf42ace680f08ff6b3f25631aa7c9516c4fe68324377a6352b510e29d427 state:dirty`, build `d1aab233941aae6b513733c5f6f4f78a5e2cd264cdfc32e937692a1be2dd52c5`, composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`, a signed certificate, systemd PID1 and its native/browser receipts. It passes witness Agent sequences 2–4 through incident/inbox, no-URL-token Bearer GET SSE, strict same-clock ordering, selected/absent routes, detail deep link/read/reload and 1440×900 screenshots. It records `browser:true`, `load:false` and `releaseReady:false`; P8g load and production deployment remain Not verified. |

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
| `monitor-notify` | Current exact Linux/amd64 50-file export at `target/rz/p8fb-r9-monitor-notify-export-20260910T125008Z`, dirty source `git:cb357c671b88ecfaf5987a30b7de39f0ef125a0d tree:a419bf42ace680f08ff6b3f25631aa7c9516c4fe68324377a6352b510e29d427 state:dirty`, build `d1aab233941aae6b513733c5f6f4f78a5e2cd264cdfc32e937692a1be2dd52c5`, composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`, signed archive/manifest/envelope/certificate `045b0094d8822783c923bc0402fdc06925e54fd51ad12907790dbc2e12dd313c` / `bd0355b61da631780f98bfdd6d049b665e7e36c40d521612e4e6272bc06efaa7` / `0f50bcbafa225a7bd74c0b4f545990fe08f1dcefe95e5a69ae1744349c790ccb` / `d62a6d9db4d71369636783e43205ea1b75558a16c4192e233f1b8331e94822cf`, native evidence `target/rz/p8fb-r9-native-20260910T125008Z/monitor-native-runtime-evidence.json` (SHA-256 `bdaea45a3b5b78e53501c36ecbc0b8aaa4512f5b8d5fea3b1739ab97745443f8`) and business receipt `target/rz/p8fb-r9-business-20260910T125008Z/receipt.json` (SHA-256 `3ab76a4af846da1e56fff144d942fd1da52888440195f8542b50f7026a066c89`) | Signed certificate, systemd PID1, verified Agent sequences 2–4, incident/inbox, Chrome 153 Bearer GET SSE without a URL token, strict same-clock ordering, Monitor-visible and Insights/Reports-absent routes, detail deep link/read/reload, and 1440×900 visual screenshots passed | `browser:true`; `load:false` and `releaseReady:false`. P8g load and production deployment remain Not verified |
| `node-agent` | Agent Cargo, config, protocol, native layout, signed apply and controlled Linux pairing evidence | Admitted to source/build certification | Final PID1/service restart and shipped-target report |
| `analytics` | Explicit Admin+Insights selected Cargo, fresh schema, composition-qualified Web containing only access and Analytics routes/API imports, reviewed access/Insights config, canonical API, delegation protocol and native-layout producers; host-synthetic export/snapshot/captured staging with exact Admin+Insights and no Agent witness | Admitted to P8a source/build certification | P8b container export closed with a restricted six-command receipt; P8c/P8d signed release and source-build certificate closed with byte-identical dual-platform `rz verify`; P8e native install, activation and systemd PID1 runtime closed at `target/rz/p8e-analytics-runtime-20260911/analytics-native-runtime-evidence.json` (SHA-256 `0d6d8c3df93bb7c1d36bc1c6316df4f8a7d0e7f839fb427bbc9b9d8e157682b1`), bound to the fresh Insights schema tree `git:1060952abae2f16d0bb0639308d6ba0f53e30252 tree:562c2a6040562febdd648a0fde6c71b9e12885ec2b74b860f5d79d8ec59bdb0d state:dirty`, build `8f1a2e993977edcad7390f445eb201ab53e37b29dbfb288ef46f50c386eabc79` and composition `62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40`. P8f real-browser journeys closed at `target/rz/p8f-analytics-business-browser-20260914T023318Z/receipt.json` (SHA-256 `00e75e6163fe7ed39824b4a594e9e990b8ffbbee4e77e9b367d6ab7fecf1de29`): seventeen Chromium 153 CDP journeys against the same signed build on one retained PID1 container cover deployment identity, unauthenticated redirect and API denial without secret leakage, wrong/default credential rejection, owner login, Analytics-only navigation, overview empty/loading/populated states, mobile zh details empty, real collection-policy and 24 tracker events ingestion, details filter/page reset/pagination, a real rz-insights outage error state with restart retry recovery, a full service restart recovery, Monitor/Reports API and route absence, and logout session revocation with re-login, with seven verified screenshots; its `runtime` flag references the unchanged P8e evidence above. Load certification and release readiness remain rejected |
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
The shared minimal-host code uses the internal `selected-distribution` feature;
the reviewed `monitor-distribution` preset enables it. The Admin entry point
still rejects an internal-feature-only build, so this refactor does not create
another shippable composition or broaden the Monitor package.

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

Analytics selected-Web input is now generated and verified for composition
`62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40`. Its
artifact contains access plus Analytics overview/detail routes and the two
Insights reads only; Monitor, Reports, notifications and management owners are
rejected by the inventory and API-source policies. The pure Insights service,
Analytics Admin composition with its isolated fresh schema, and this Web
artifact are source/build evidence only. Complete Cargo, selected
API/schema/config/protocol/native producers and runtime certification remain
required before P8 can admit Analytics.

The Monitor native producer embeds only the verified composition-qualified copy at `apps/admin/selected-web/<compositionId>/dist`; the generic `apps/web/dist` is used only by the full composition. The Admin build script rejects a missing selected inventory, a composition mismatch, or an absent selected `index.html`. Docker accepts only the reviewed `monitor` and `monitor-notify` presets, each mapped by a literal Docker case to its fixture and exact Admin/Monitor Cargo features. Each physically emits server `rz-admin`/`rz-monitor` to `release/server/bin` and Agent `rz-monitor-agent` to `witness/bin`; selected Web and API/schema/config/protocol/native-layout contracts carry the same composition. The branches build and verify selected Web before copying it to the corresponding embedded directory and compiling Admin.

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

P8f selected-server browser work begins with explicit `monitor` or `monitor-notify`
admission. The bootstrap verifier may record its current checkout as verifier provenance,
but it accepts product identity only through the signed certificate/export tuple and
rejects cross-preset input before Chromium. `verify-monitor-notify-bootstrap-browser`
uses a retained notify P8e container only to execute the four existing bootstrap/SRI/retry
cases; it is separate from `prepare-monitor-load-runtime` and cleans that container. This
source/static implementation alone does not claim a monitor-notify business journey,
browser acceptance or P8g/load result; an exact runtime receipt is required.

That bootstrap closure passed for the old signed monitor-notify tuple at
`target/rz/p8f-notify-bootstrap-browser-20260910-r1/manifest.json` (SHA-256
`1785ad619ee750fb705771fdf1b3aeefa4ac773a51414e0b2f7a4d808de07c87`).
Chrome 153 executed the selected entry only for the success case and rejected
binding mismatch, binding fetch failure and SRI entry failure without changing
the retained Admin executable/runtime attestation. The paired fresh P8e record
is `target/rz/p8f-notify-bootstrap-native-20260910-r1/monitor-native-runtime-evidence.json`
(SHA-256 `870f2dec63ed679c18c957a1b7285aed0f747d193a22927e2a0b4879fe92551b`).
The retained owner-labelled container was removed after receipt publication.
Authenticated product journeys, alert delivery, inbox/SSE behavior, load and
deployment remain separate gates.

P8f-B is a separate monitor-notify business journey: it retains a fresh P8e PID1 container, drives the installed signed Admin/Monitor path with the verified export witness Agent, and records browser/SSE/inbox evidence in one canonical receipt. It does not use the P7 temporary Admin/Monitor runtime or open P8g/load/deployment. The current R9 closure binds dirty source `git:cb357c671b88ecfaf5987a30b7de39f0ef125a0d tree:a419bf42ace680f08ff6b3f25631aa7c9516c4fe68324377a6352b510e29d427 state:dirty`, build `d1aab233941aae6b513733c5f6f4f78a5e2cd264cdfc32e937692a1be2dd52c5`, composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`, its 50-file export, signed certificate, native evidence SHA-256 `bdaea45a3b5b78e53501c36ecbc0b8aaa4512f5b8d5fea3b1739ab97745443f8` and receipt SHA-256 `3ab76a4af846da1e56fff144d942fd1da52888440195f8542b50f7026a066c89`. It passed systemd PID1, Agent sequences 2–4, incident/inbox delivery, no-URL-token Bearer SSE, same-clock ordering, route absence/visibility, detail deep link/read/reload and 1440×900 screenshots. Its `browser:true` does not change `load:false` or `releaseReady:false`; P8g load and production deployment remain Not verified.

P8f-A implements the selected-Web build binding: the Monitor producer stamps
only its generated HTML after deriving the normalized file-table digest, writes
the canonical descriptor and records final file hashes in inventory v2. The
container export carries descriptor and selected API source bytes; its host-side
validator rejects a changed descriptor, inventory, API source, stamped HTML or
emitted asset before native staging/release derivation. HTTP bootstrap and
browser behavior remain later closures.

P8f-A/bootstrap implements the Monitor-distribution HTTP bootstrap boundary. Admin validates
and embeds the P8f-A binding, rejects a packaged/runtime composition mismatch before
listening, serves anonymous `GET /__web-binding` and authenticated
`GET /api/installation` with no-store caching, and keeps both routes out of Full
until Full has an equivalent packaged binding. Selected HTML contains a dependency-free
inline bootstrap instead of an eager module tag; it validates the anonymous response
before dynamically loading the content-addressed entry with SRI and applies one bounded
cache-busting recovery attempt. The Admin build gate validates the exact 11-field
inventory-v2 schema and canonical metadata, sorted route/asset/module/emitted/file
tables, the actual file set and bytes, selected API digest and normalized Web digest.
`scripts/verify-selected-web-bootstrap-browser.py` is the executable P8f-A/bootstrap
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
external Chromium runtime remain the P8f-A/bootstrap acceptance evidence rather than reusing a
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
The final P6b `linux/amd64` Reports-to-Admin gate is published at
`target/rz/reports-notification-runtime/runs/20260910T182747Z-31773/manifest.json`
(SHA-256 `32d9a5d38c1a86c823da1f61064cf21c9a10ad25b525548a09ddd2a5a5da9552`),
for head `1c96269d341bd693b8d5486c4f42fb36fe6a314e` and dirty source tree
`dd299df31d9d2cfd524062f21546280b5340d3475bbf51ff3ef0a5af56eb007b`.
Its build provenance SHA-256 is
`0199b0630ac97d98c2b6c34ab8ece35a1be85115225a7e2f564b9a1ed30964e9`,
and it binds 27 exact receipts, six manual terminal classes, immutable retry
initiator, scheduled silence, outage backfill, duplicate reconciliation,
authentication denials, and selected/pure runtime identity evidence. The
manifest binds binary hashes and build provenance but does not retain binary
bytes, so `current` is not a basis for rehashing binary files. Selected state
has eight `stored`, one `no-recipients`, eight messages, eight recipients, zero
outbox rows and five zero gap counters. Pure Web is composition
`8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b` with
digest `1ce67f44c7ae9f4f7a511ae3bd4c583b5166bf54c951fc0dc2245c3cc71957ec`
and feature IDs `access,monitor`; selected and pure runtime identities are
`rz-reports` UID/GID `999` with `0750` directories. Failed runs are diagnostic
history only and are never published as `current`. Native systemd, production
deployment, SSE, UI and sustained load remain **Not verified**.

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

The current P8g R8 closure is a local Colima Linux/amd64 exact-artifact load
run for `monitor`, not a production deployment. It binds dirty source
`git:a4a8beed33e842857a535cd051902dea3a4bf5c6 tree:c4fa33e1d205ae6e354f282346e25a4af08248ce0d3578fdd6fbed0b66f074bb state:dirty`, export
`target/rz/p8g-r8-monitor-export-20260910T145456Z`, build
`49630979fee226a3f1850d09a0296e752a1a4bce8fc67d4f8751e260a887e492` and
composition `8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b`.
Its signed archive/manifest/envelope/certificate SHA-256 values are
`6ba42a9b005710500bb33f6160fcbd265f8ab0c1e93f694c28e8a4b73725b7ca` /
`3e612c631a806abb6341b1856c496e0616b6e5f8a858ce9f953fde7f9fca9d65` /
`8bab407380e6d784546fd65951ea76124723658fb03bc9992ca9d243e8a1c03f` /
`3c0e8b2b8eb2180939cea36a93c1bb2a7c435ba8cd7c28750e6796d3a01cd314`.
The native, browser and load receipts are
`target/rz/p8g-r8-native-20260910T145456Z/monitor-native-runtime-evidence.json`
(SHA-256 `e9610e21899c91945525f2b4a54b9ded8eb7f71e8714ba77ac115e4d9ce2c8db`),
`target/rz/p8g-r8-browser-20260910T145456Z/manifest.json`
(SHA-256 `1b2504dd188ca890dbcba110d7218443c10b46fb4c6c00f7a683bc24dc997213`) and
`target/rz/p8g-r8-load-20260910T145456Z/monitor-load-evidence.json`
(SHA-256 `34c26a895b9236085ccfbbadcc521c94c52d72f5a52a33bbc0cf0acfa7fe32aa`).
The two lanes completed 12,389 and 12,661 requests with zero failures (p95/p99
227/270ms and 220/271ms); recovery completed 2,138 requests with zero failures
(p95/p99 224/268ms). Overall peaks were RSS 28,614,656 bytes, VmHWM 45,490,176
bytes, cgroup memory 74,985,472 bytes and 42 processes. The controlled fault
recorded stop, listener-gone (503/code 40001), listener-ready and registry-healthy
in that order; pure-Monitor SSE was runtime 404 with empty signed API/Web/schema/service
forbidden sets. Retained runtime cleanup completed. This records `load:true` for
that local exact artifact; production deployment and `releaseReady` remain **Not
verified**.

Analytics P8b step 3 is **Closed locally / Passed** at `target/rz/p8b-analytics-container-20260911T020403Z`. One real Colima `linux/amd64` BuildKit export bound dirty source `git:6c12c7227e5ec2519c6fee506a9d0407c7494448 tree:df92f84c8b22c6a73224c71082001921750800e64888f5e61455831c00362900 state:dirty` and composition `62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40`. The host validator accepted the exact 43-file export before and after the restricted pass with unchanged source identity, and a fresh process re-verified the persisted receipt (SHA-256 `c2ba3766b33aa301c00aa59109c18d98db34d780ffbd695c897f12a4602d36d4`). The receipt binds six real restricted commands (Admin/Insights selected API, access/Insights config, Admin/Insights protocol) to the exported `api.json`, `config.json` and `protocol.json` bytes with `manifestSha256` `34ffa2c02729171995039a15387ca095a69f76674c8fe1417ac3ba666f07394c` and `provenanceSha256` `4e78ffb0872e6d4dd4a2eca9a54d8b5e2fff07a591242335bbc37ea448434012`, plus verifier image `debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171` and restrictions `network:none`, read-only rootfs, tmpfs `noexec,nosuid,nodev`, all capabilities dropped, no-new-privileges, 32 pids and 256m memory. Its `runtime`, `certificate`, `signing`, `installer`, `browser` and `load` flags are all false; P8c and later remain rejected.
