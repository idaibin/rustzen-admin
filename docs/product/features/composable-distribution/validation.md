# Implementation sequence and acceptance

Status: planned tests and implementation gates; only evidence explicitly named
below is claimed complete. Native staging tests cover exact inventory, stable
reads, no replacement, publish locking, temporary cleanup and manifest snapshot
binding.

Analytics P8b step 2 validates synthetic host bytes, identity, modes, contracts,
selected Web and captured staging only. It is not container, signing, installation
or runtime evidence and does not change the Analytics P8 admission result.

P8c adds focused unit and CLI integration tests for captured-snapshot signing:
strict arguments; private/public PEM ownership, mode, link, size and stable-read
rejection; strict single-SPKI public PEM and key-pair rejection; exact three-file
immutable final inventory; publication-bound signature reread; and
existing-final refusal. The Linux verifier is a synthetic end-to-end triplet
gate: it creates a fixture export and ephemeral Ed25519 key inside Linux, then
checks `find`/`stat`, reread verification, tamper rejection and final-output
refusal without emitting a PEM. It is not an actual P8b container export,
installer, systemd, browser, load or deployment gate.

Execution progress and the current fresh-only repository policy are tracked in
[implementation](implementation.md). Tests concerning historical upgrades or
rollback compatibility from the reviewed draft have been replaced below by
fresh-root rejection and same-build interrupted-install recovery scenarios.

## Supported build matrix

Official first-release presets are `full`, `monitor`, `monitor-notify`,
`analytics` and `reports`. `custom` is an explicitly generated developer
selection, eligible for distribution only after the same tests pass for its
exact composition and build. Passing `full` never certifies a subset.

Before a source/build certification command runs, the P8a admission audit must
cover every named catalog preset and must reject unknown or incomplete producer
sets. The current admitted set is `analytics`, `monitor`, `monitor-notify` and
`node-agent`. Analytics additionally has locally verified host-synthetic P8b
export, immutable snapshot and captured-staging seams. These are not container
evidence: Docker, certificate, signing, installer, runtime, browser and release
certification remain unimplemented, and Docker rejects Analytics.
Target `full`, `reports`, `current-full-regression` and `custom`
return a machine-readable blocked result; `--require-ready` exits nonzero.
`monitor-notify` has exact Linux/amd64 build/export evidence at
`target/rz/p8b-monitor-notify-export-20260910T065139Z`, bound to dirty source
identity `git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty`
and composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`.
Its 50-file export passed host validation before and after the restricted run;
`target/rz/p8b-monitor-notify-export-20260910T065139Z-contract-checks/manifest.json`
records nine commands (eight retained selected checks plus one Agent config
witness), network disabled, read-only filesystem and all capabilities dropped.
Native installation/PID1, browser and load remain Not verified; this is neither
certification, publication nor release evidence.
Admission is not a certificate and cannot satisfy any native, browser, load,
installation or release gate.

## P8b selected-server source-build certificate contract

P8d accepts an admitted production `monitor` or `monitor-notify` server snapshot and signed release root,
not an explicit binary root or caller-provided source/build claims. It rejects
forged snapshots, invalid source identity, release mutations, cross-evidence
mismatch, unexpected inventory/modes, unsafe publication parents, races and
existing final output. Tests also delete the sibling payload before issue,
reject a fully signed manifest that differs from retained snapshot bytes, and
reject forged publication capabilities. Linux/amd64 synthetic end-to-end
validation covers the issuer, canonical certificate, reread and tamper rejection.

Its structural parser rejects unknown fields/layers, every selection except the
exact `monitor` or `monitor-notify` selected-server plans, and
every attempt to set `runtime`, `browser`, `load` or `releaseReady` true. The
expected-input verifier separately rejects altered source/build/artifact identities.
The issuer must capture those inputs itself; CLI-provided digest or toolchain
claims are not authoritative. Successful P8b evidence still requires
separate Linux `rz verify`, dry-run and fresh apply verification, then native
runtime, browser and load evidence before a release gate can open or a `current`
pointer can change.

The source-build certificate closure verifies only the canonical schema against
authoritative inputs and makes contract extraction require an explicit binary
directory. Its issuer and crash-safe publication are implemented for the reviewed selected-server snapshots; native, browser, load and release readiness remain **Not verified**. The separate container-export closure now supplies stable
same-batch BuildKit and Linux artifact evidence, but does not issue that
certificate.

The exact `monitor-notify` P8c/P8d evidence manifest is
`target/rz/p8cd-monitor-notify-evidence-manifest-20260910.json`. It binds the
old dirty export identity `git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty`, composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d` and build `d2c4e0f452c7400927059bc499822ef1218a9c47547ca88eb25706bbfa1426bd` to archive `92a5c59f0dcc8b34058f785a840187212a7cb7eb53a656d338010a7b183fa4ce`, manifest `c74e970ca5729e5e985b233b7153d604a66866e855c720c2132e758e8c1f1b5b`, envelope `806ee234f003f815a86b230b5665e6ec7b5b83a67c2d7bfd3944d7599aca60c7` and certificate `8eba6380fe0e130288d0038d561b9a61a07abdbaad3ef049068164c3aefd1436`. The host and Linux/amd64 verifier JSON SHA-256 values are both `19acf4cb9484f42f618ccf87a0e164ccc949f922bd29487f71ec2d854d0e63a3` and their bytes match exactly. The private key was deleted; only public key SHA-256 `04db0497facd7cf34e8e0cb8821a386c852bf947275a47a6465263165bc31d8e` remains. This records one old-export tuple, not a rebuild of current `0d99f28` source; all literal `runtime`, `browser`, `load` and `releaseReady` flags remain false, and it proves neither native/PID1, browser nor load behavior.

## P8e selected-server native runtime gate

The Linux/amd64 gate requires an explicit reviewed `--selection` (`monitor` or
`monitor-notify`) and first rereads the canonical certificate through the issued
capability, then runs `rz verify`, dry-run, fresh `apply`, `install-status` and
`activate-monitor-server` inside a disposable PID1 container. It captures
publication/activation markers, enabled and active units, MainPID executable
inode/digests, two health bindings, owner login and default-password rejection,
omitted Insights/Reports absence, restart and both service start orders. A
separate process rereads and parses the runtime evidence. This is local
target-like evidence; browser, load and deployed-host verification remain
separate.

Its CLI rejects missing, duplicate, unknown or positional arguments, historical
pointer files, paths outside the allowed boundary and any existing/symlink
output. A run must supply a newly produced release result and certificate for
the explicit export plus an independently obtained expected source identity.
It rejects every ancestor, descendant or equal relationship between output and
the export, release root, certificate directory or public key. Executable
negative tests prove rejection before output creation or Docker invocation and
prove the input tree remains byte-for-byte unchanged. Evidence can be published
only into one new child of `target/rz`; failure evidence may be appended there
but the directory is never reused or recursively removed.

Its selected evidence parser and independent revalidator require the same
preset/composition as published admission. `monitor-notify` requires a rejected
unauthenticated notification-ingress request and its four exact activation
owners; pure Monitor requires the ingress to be absent and rejects notification
activation keys. Both selections retain exactly the Admin and Monitor PID1
services and keep browser, load and `releaseReady` false. These are source/static
checks until each selected tuple has a separately retained Linux runtime record.

The monitor-notify record is
`target/rz/p8e-monitor-notify-native-runtime-20260910-r2/monitor-native-runtime-evidence.json`
with SHA-256 `aa15c4985ff5ab16b6dd5ef52bd7bbd6709724cfdb5193b2e1b678ebb4710921`.
It binds old dirty source `git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty`,
composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`,
build `d2c4e0f452c7400927059bc499822ef1218a9c47547ca88eb25706bbfa1426bd`,
certificate `8eba6380fe0e130288d0038d561b9a61a07abdbaad3ef049068164c3aefd1436`,
activation marker `a5a181259015276d3ef7db41c760249c17e88d6a7e79319b1f72bd2cc077587c`
and publication marker `54476c47a392412d60d8fe588cfbebf536df70f11a981d8c91019937ff6c3e9d`.
Its Admin and Monitor executable digests are respectively
`9b73ea1eadc69661fa5ba060367c4d76e658d297bc38f3f1119a246ba313b3cd` and
`e47ec801568d59a0c70561c75f8b561798da9a83de94238423c447c45c3dd46f`.
The fresh install/PID1, restart, both start orders, health/MainPID, owner login,
default-password rejection, Insights/Reports absence and rejected notification
request all passed; the ingress response was `401` with body `bad-producer`.
The record has `runtime:true` and literal `browser:false`, `load:false` and
`releaseReady:false`. It is exact evidence for that old tuple only: it does not
prove a current-HEAD rebuild, browser, load or deployment.

The P8b selected-server container closure is verified separately from certification. It
static boundary test admits only the reviewed `monitor` and `monitor-notify`
fixture/feature cases and must reject a non-Linux/amd64 build stage, an omitted Monitor
source-identity requirement, a target other than x86_64 musl, a missing explicit
binary root for protocol extraction, omission of any Web/API/schema/config/protocol/native
output, a misplaced or extra server/witness binary, a missing output manifest or
provenance record, and a build path that omits the exact command arrays. Its
artifact test creates an exact temporary payload and rejects missing contracts,
unexpected payload paths, links, unsupported modes and non-Linux/amd64 provenance.
Passing these Monitor-only Docker tests establishes source and static producer
closure only; Analytics remains rejected by Docker and has no container or
runtime evidence. The
local P8b gate additionally rebuilds through Colima BuildKit for Linux/amd64,
requires identical source identity before and after the build, validates the
whole export on the host, and executes the selected binary contract commands in a networkless read-only
container with all capabilities dropped: Monitor has seven commands (six
server API/config/protocol checks plus one Agent config witness), while
monitor-notify has nine (eight selected server checks plus the same Agent
witness). Their API, config and protocol outputs must match retained contracts;
the Agent config witness must satisfy its selected structural policy. The host then validates the export again. The Monitor accepted local export is
`target/rz/p8b-container-export-verified`; the monitor-notify export is
`target/rz/p8b-monitor-notify-export-20260910T065139Z` with its separate
contract-check manifest above. Neither export opens native installation/PID1,
browser, load, certification, publication or release status.

The P8b host export-validator test corpus builds a temporary canonical Monitor
export and must reject every unknown/duplicate/missing CLI argument, changed
source identity, noncanonical JSON, unknown record field, changed
selection/composition/target, changed manifest file mode/size/digest, missing or
extra payload member, empty Web `dist`, altered provenance command array, wrong
Rust host tuple, malformed/non-x86/non-PIE/interpreted ELF and a missing or
wrong release marker. It retains the exact bytes returned by the one stable root
read; the test must prove that a post-read replacement cannot affect the exposed
snapshot. This is host artifact verification only, not a Linux execution,
certificate, native staging, install or release-gate test.

Exhaustively validate dependency closure and generated inventories for the
finite capability catalog. Compile and integration-test official presets plus
every custom selection actually released. Additional pairwise combinations are
added when they expose a concrete interaction; do not promise every theoretical
combination works without testing it.

## Implementation slices

Each slice has one implementation owner. An independent reviewer checks the
fixed candidate only after it exists. Preserve unrelated work in the current
checkout; no worktree, commit, package replacement or deployment is implicit.

| Slice | Deliverable | Exit gate |
| --- | --- | --- |
| 0. Design | Product boundaries, architecture, contracts, sources and ten actual external review rounds | Material design issues resolved or explicitly left pending; no invented approvals |
| 1. Inventory | Finite capability catalog, selected plan, actual full-build inventories | Full preserves every selected current journey; no source path is assigned ambiguously |
| 2. Minimal access | Separate auth/security startup from deploy/tasks/log console; selected registry and permission seeds | Monitor-only API tests pass without optional Admin services or schemas |
| 3. Physical pruning | Selected Rust features/packages, generated baseline and Web graph | Full and monitor artifacts pass positive and negative inventory tests |
| 4. Native distribution | Signed selected manifest, packager/installer/recovery and CLI | Fresh Linux full/monitor installs and selected-service failure tests pass |
| 5. Inbox | Admin persistence, recipient/read APIs and UI states | Access/retry/concurrency/retention tests pass |
| 6. Incident notifications | Monitor optional outbox, internal ingress, durable receipt/fanout | Trigger/recovery and crash-window tests pass |
| 7. Realtime | Fetch SSE, permission lifecycle, bounded queues and reconciliation | Browser and proxy disconnect/slow-client tests pass |
| 8. Second producer | Reports terminal events and selected recipient policy | Reports execution remains independent; feature-absence tests pass |
| 9. Release matrix | Other official presets, measurable capacity and operational instructions | Every shipped digest has its own acceptance report |

The full builds in slices 1, 3 and 4 are an internal current-behavior regression
fixture while new notification slices are unfinished. They are not certified
target-full releases and cannot silently omit the newly specified notification
capability. Slice 9 certifies target full only after slices 5-8 pass. Monitor can
be certified earlier after its applicable access/pruning/native gates; neither
that result nor the regression fixture certifies monitor-notify or target full.

Name that nonrelease selection current-full-regression, with releaseClass=test.
It cannot claim preset=full, use production signing keys/trust or production
installation roots, or produce a releasable acceptance report. Only the explicit
test harness accepts it. Production resolver/packager/installer reject it even
if someone adds its key to a trust store. Full preset validation requires its
exact enumerated capability closure, including notifications; renaming an
incomplete fixture cannot bypass the rule.

Before refactoring, run and freeze core existing-journey assertions on the
designated current source+dirty baseline. Each current capability needs a success
and material denial/failure path with observable API fields/status, persistence
and UI results. Reuse those assertions unchanged on current-full-regression and
target full, except for explicitly accepted contract changes with a recorded
before/after expectation. Inventories alone cannot prove preserved behavior.

Implement tests for each slice before its behavior where they establish
cross-layer contracts. Do not write mirror tests for simple documentation edits.
Each slice updates applicable product/UI/API/architecture/build documentation.
The root `justfile` remains the executable command owner. Proposed script names
and commands in the design are not current working commands.

## Distribution tests

| ID | Trigger | Required result |
| --- | --- | --- |
| D01 | Resolve monitor | Closure includes access + monitor only; no notifications implied |
| D02 | Unknown feature / cycle / unsupported custom selection | Fail before build/install writes |
| D03 | Build full then monitor using the same source checkout | No stale full Web assets or generated schema in monitor output |
| D04 | Inspect monitor Rust dependency graph | No feature-owned Insights/Reports/browser automation or Admin deploy/task dependencies |
| D05 | Inspect tar and native install | Exact allowlisted files/units/directories; no excluded executable, config or image. The Monitor native producer binds its descriptor, selected Web inventory and package manifest to one composition ID before signing. |
| D05a | Read selected-native layout with an added/removed/repeated/cross-class unit or config member, stale composition, link, or replaced file; supply a native digest directly | Canonical stable single-file reader rejects it; Monitor server names only target/Admin/Controller and consumer-scoped config files, Agent names only its unit/config file; manifest derives and binds the native digest. |
| D06 | Fresh monitor database initialization | Only access and Monitor tables/indexes/seeds; no notifications/tasks/deploy/unused dictionary tables |
| D07 | Owner calls every excluded namespace | 404 JSON/API failure, never 200 SPA or privileged fallback |
| D08 | Inspect compiled Web entry + all lazy chunks/maps/assets | No excluded route/API/search/nav/product module in graph. The Monitor producer records Vite module IDs and emitted files, validates them against the resolver-derived route/public allowlist, then scans text as a secondary sentinel check. Its mutation suite rejects excluded, foreign-composition, relative or absolute-escape source modules; unknown dependency or virtual module IDs; dynamically composed namespaces; extra static assets; and symbolic links at every output/generated/API-source path ancestor, as well as every inventory identity/list field drift. The selected Vite output is composition-qualified and never reuses `apps/web/dist`. |
| D08a | Compile Monitor Admin while full `apps/web/dist` is present | Build uses only `apps/admin/selected-web/<compositionId>`; missing or mismatched selected inventory fails before compilation and embedded assets contain no excluded API/capability sentinels |
| D08b | Pass empty, unknown, custom or unreviewed Docker `DISTRIBUTION` | Docker fails before Web or binary build; only literal `monitor` and `monitor-notify` fixture/feature mappings are accepted |
| D08c | Monitor Docker output contains a binary in the wrong server/agent directory or an extra selected binary | Static Docker verifier rejects; server is exactly rz-admin/rz-monitor and Agent exactly rz-monitor-agent |
| D08d | Protocol artifact has altered descriptor/digest/class/composition, extra file, link, or replacement during read | Controller/Agent output and reviewed golden must agree; canonical single-file reader rejects every mutation before manifest/build identity derivation. |
| D09 | Run monitor for 10 minutes with request tracing | Only two server processes/ports; no probe/timer/config lookup for absent services |
| D10 | Stop installed Monitor | Entry login works; authorized monitor navigation remains; API shows unavailable |
| D11 | Change env to enable an absent module | Startup validation rejects it; cannot create route/schema/process |
| D12 | Mix full Web assets with monitor backend | Digest mismatch blocks feature bootstrap before business requests |
| D13 | Tamper signature/member/hash/path or inject unknown unit | Installer fails before switching or executing archive content |
| D14 | Wrong target / composition / build / schema identity | Reject installation or same archive/envelope/manifest tuple fresh-root journal continuation without DB mutation |
| D15 | Selected services start in either monitor order | Eventually ready; no dependency on omitted services |
| D16 | Full-to-monitor attempt on existing root | Reject; instruct fresh-root workflow; preserve data and prior installation |
| D17 | Crash during fresh payload publication; retry the same exact archive/envelope/manifest tuple | Local CLI journal resumes only its owned fresh payload publication after revalidating every input; it creates no deploy table, service-manager action or historical DB restore |
| D18 | Request historical rollback or reuse another build's databases | Reject; preserve old data and require a fresh destination |
| D19 | Agent-only artifact inspected | No controller/Web/Admin executable closure or DB initialization |
| D20 | Omit notifications | No inbox APIs/tables/UI, outbox, relay, SSE queue/listener/worker or periodic retries |
| D21 | Mix old/new Web and backend plans of the same composition | Different webDigest blocks bootstrap; different buildId alone does not, provided Web/API pairing gates pass |
| D22 | Monitor-only access bootstrap, role change and session revoke | Essential access-settings UI/API work; no optional Admin console route/table is created |
| D23 | Baseline behavior has no owner, two owners, or is removed from catalog | Full completeness fails against independent current-behavior baseline |
| D24 | Provision/rotate shared Agent secret; stop Entry then Monitor | Monitor performs final credential check; old secret rejected after rotation; report gaps/retry/unavailable semantics match the declared shared-secret model |
| D25 | Inspect server artifact and invoke Agent mode/config | Controller-only closure rejects Agent mode; no Agent collector/service/config initialization; artifact-class mismatch blocks installation |
| D26 | Inject forbidden default feature, OpenAPI component, config key, daily timer or build-script output | Each binary's inventory/closure rejects the leak before packaging, even if another selected process needs that owner |
| D27 | Keep buildId but change Web bytes/digest | Build validation rejects descriptor/inventory/stamp disagreement; at runtime HTML/backend webDigest mismatch blocks the business entry and credentials, and the dynamically loaded entry has matching SHA-256 integrity |
| D27a | Add/remove an inventory-v2 field; change or reorder routes, assets, module owners/paths, emitted/file tables, generated/output roots, binding descriptor, selected API source, stamped HTML or any emitted asset | Admin compile and selected-Web/container validation reject before staging; the exact 11-field metadata, semantic selected-owner policy, canonical order, normalized digest and final stamped-file inventory must all agree. Allowed host/Linux module-graph differences pass without a platform-specific inventory-byte digest |
| D27b | Feed direct staging a self-consistent Web bundle with another preset/composition/routes/assets/modules or excluded Reports text | The complete selected-Web policy rejects it; caller route strings cannot alter the build identity derived from verified inventory |
| D27c | Re-sign a release after changing binding.json, the index stamp, or another Web file and updating its final file hash | `rz verify` rejects because the canonical binding, normalized HTML and actual Web-byte digest no longer form the signed manifest tuple |
| D28 | node-agent with Web/server fields; server missing schema owner | Discriminated manifest validation fails before installation writes |
| D28a | Canonical manifest producer/validator receives unknown/duplicate/escaped/link/mode/size/hash fields, server-Agent mixing or a digest without its actual source discriminator | Reject before archive/signature/install; canonical resolver, selected-Web file table and binary byte digest inputs are the only accepted sources |
| D28b | Monitor recovery journal has a changed archive/envelope/manifest tuple, unsafe metadata, an occupied root, a replaced empty/sentinel/wrong-nonce work root, a copied marker on an incomplete root, or a final root with missing/extra/truncated/replaced payload or retained triplet | Reject without modifying the root. The exact tuple converges after either cleanup fault only after the completion marker and manifest-derived final layout, payload digests/modes, retained triplet and current link all verify; no continuation marker or sibling journal remains. |
| D28c | Run `rz verify` and `rz apply --dry-run` against valid Server and Agent triplets; alter a tuple field, PEM/key ID, member header/hash/path, destination or input type | Verification rejects before destination writes; dry run leaves no root; Linux root apply publishes only selected payload/layout and retained triplet; systemd and DB are not started or initialized |
| D28d | Activate a fresh signed Monitor server under PID1 with root-only config and a unique one-time owner secret; retry, alter config/unit/source/database metadata, preinstall or enable Reports/Insights residue, swap in a same-schema foreign DB, interrupt either database rename or each daemon-reload/start/readiness/enable/marker/marker-dirsync/journal-remove boundary, or inspect excluded files | Only Admin+Monitor config/unit/schema paths exist; the owner logs in with the supplied secret, not a migration default; exact retry and every same-tuple interruption converge; invalid ports/password/path parents and unselected service residue fail before account or destination writes; schema corruption and foreign installation identity are rejected without DB mutation; conflicts/failures leave no false ready marker and unselected services/config/database files remain absent |
| D29 | Same DDL but different build or data semantics; legacy compatibility field supplied | Existing-root reuse and unsupported fields rejected; use final fresh baseline |
| D30 | Excluded route has lazy side effect/glob; excluded asset exists in source public | Selected generator directory, Vite graph and copied file inventory exclude all sentinels |
| D31 | Remove used backend operation or import excluded client through shared selector | Frontend operation/schema pairing or owner gate fails before release |
| D32 | Cached stamped full HTML, wrong digest, binding outage, or old missing entry against current monitor backend | Anonymous exact-schema/no-store binding is requested before the business entry with credentials omitted; failure makes no login, installation or module request, preserves the local deep link, permits at most one cache-busting reload, then shows a static explicit retry/error view |
| D33 | Deep link, revoked grant, unavailable service and access-only user | State priority, generation-aware cache purge and authorized landing work without stale content or unauthorized diagnostics |
| D34 | Public first-owner attempt, two local initializers and restart | No claim endpoint; exactly one local owner transaction; persistent marker prevents reuse |
| D35 | Two owners concurrently disable/delete/demote each other via different paths | Atomic write invariant preserves at least one login-capable owner; no separate stale preflight count |
| D36 | Selected query references excluded table; stale full SQLx metadata present | Selected scratch-schema query suite/macro validation fails before packaging |
| D37 | Wrong-owner or same-schema foreign DB, dropped index, extra trigger, changed identity or pending migration | The owner-local installation singleton, exact ledger and canonical inventory reject without product schema/data/ledger mutation or normal startup |
| D38 | Changed durable encoding with unchanged descriptor | Current code/descriptor conformance fails; tests use current fresh data, not old-reader fixtures |
| D39 | Crash at each initialization boundary; concurrent fresh initializers | Final DB is absent or complete; no overwrite, partial publication or public bootstrap |
| D40 | Fragment mutates another owner's object; required full seed removed | Ownership and independent full-baseline gate fail against actual scratch DB |
| D41 | Add or remove notifications on same root, then new root | Same-root apply rejects before DB open; new root contains only selected fragments, old data preserved |
| D42 | Service UID writes trust/current/journal/units or reads another owner's DB/secret | OS permissions deny access; only paired keys are shared; final service units still run |
| D43 | Release UI submits arbitrary action/path, retries, or loses Admin mid-install | Fixed one-shot admission rejects arbitrary authority; one accepted job continues/dedupes; absent UI has no privilege rule or spool |
| D44 | Crash before/after fresh unit publication, reload, first writer and readiness | Same verified build resumes or remains stopped; no mixed identities/deadlock; corrupt executor does not fall back to an older build |
| D45 | Swap input after verification, replace staging path with link, tamper recovery executable | Same-object extraction and same-build recovery revalidation reject before publication/execution |
| D46 | Reports browser absent or fixture fails under generated unit; monitor installed | Reports/full readiness fails with selected diagnostic; monitor never probes/packages browser dependencies |
| D47 | Keep all routes but break an Analytics field, Reports transition or release result | Existing-journey oracle fails despite unchanged ownership/file inventories |
| D48 | Relabel incomplete fixture full; publish/install test-only selection | Production tools reject fixture and nonexact full closure; only test harness accepts test artifacts |
| D49 | Authorized bundle A replaced by valid B before root snapshot; reuse request ID with B | Exact authorized digest/build tuple rejects before service stop; identical A retry dedupes |
| D50 | Protocol-v1 controller profile paired with v1/v2 Agent; alter emitted descriptor | v1 fixture passes; v2/missing profile fails pairing/install/start; code/descriptor mismatch fails build; no first-report discovery dependency |
| D55a | Activate with a placeholder token, invalid node/URL, profile mismatch, unsafe source, or conflicting env | Reject before activation; config/profile/unit bytes and metadata remain unchanged; token is absent from output. |
| D55b | Activate valid production config | Publish only `/opt/rz/config/rz-monitor-agent.env` as `root:rz-monitor-agent 0640`; selected unit recorder sees daemon-reload, enable, start; no `rz.target` or deploy unit. |
| D55d | Concurrent same and different activation tuples; fail each systemctl boundary | Same tuple serializes and reuses one activated marker without duplicate actions; different tuple conflicts; failure writes no marker and retry converges. |
| D55c | Installed arm64 Agent reports to a controllable Controller/TLS fixture | The Linux-only Python standard-library fixture runs the installed Agent under its service UID against a local HTTPS Controller and Unix datagram readiness receiver. It records request/readiness evidence as JSONL, requires `READY=1` after `accepted` and `duplicate`, and rejects readiness after 401 or a dropped network response; bounded timeouts terminate each Agent and remove sockets. Python is used because Bun is absent from the installed Linux container and Python provides HTTPS plus Unix datagrams without an added dependency. Real PID1 enable/start/restart remains Not verified. |

`D04` and `D08` use package/module reachability and emitted inventory, not only
string scanning. Symbol strings can be stripped; common dependencies can be
legitimately shared. File listings and production browser requests are separate
evidence. A source lockfile listing an omitted dependency is not runtime leakage.

## Notification and access tests

| ID | Trigger | Required result |
| --- | --- | --- |
| N01 | Three abnormal samples then three normal samples | One open + one resolved transition; no event per sample |
| N02 | Duplicate/stale/retired-boot Agent report | No new liveness/sample/incident/notification mutation |
| N03 | Offline incident + next accepted report | Open and resolve events follow existing business transitions |
| N04 | Disable policy for active incident | Resolution and optional event commit together |
| N05 | Crash before producer commit | Neither incident transition nor outbox record is partially committed |
| N06 | Crash after producer commit before send | Relay recovers committed event |
| N07 | Consumer commits then response is lost | Same-ID retry returns duplicate; one receipt/message/recipient set |
| N08 | Same ID with modified body | 409, no new writes; producer quarantines |
| N09 | Admin down within configured capacity/horizon | Monitoring and Reports continue; events replay after recovery |
| N10 | Outbox capacity/horizon exceeded | Business detection continues; durable gap/expiry counters visible; no false complete-delivery claim |
| N10a | Authorized delivery status read | Each module's exact protected route returns aggregate status; public or allowlisted access is rejected |
| N11 | Fanout fails mid-transaction | Receipt and partial recipients roll back; later retry can complete |
| N12 | New permission grant after original acceptance | No automatic historical subscription |
| N13 | Permission revoked or module disabled | List/count/detail/read/SSE never expose now-inaccessible messages |
| N14 | Concurrent read-all and new message | Only snapshot-bounded records marked read; new arrival remains unread |
| N15 | Duplicate reads from two tabs | Idempotent read timestamp, consistent count; no negative counters |
| N16 | Logout/token expiry/user disable during idle SSE | Connection closes within revalidation target |
| N17 | Reconnect between subscribe and list snapshot | Buffered revisions cause required refresh; no missed terminal invalidation |
| N18 | Slow socket / queue full | Bounded memory, coalesce or disconnect; no business transaction blocked |
| N19 | Ingest without producer auth / cross-producer topic / body mutation | Reject before business writes |
| N20 | Public request to internal ingestion path | No route/proxy access, even with ordinary user JWT |
| N21 | HTML/external action URL/secret-like body/oversize payload | Reject or sanitize by fixed contract; no HTML execution or arbitrary navigation |
| N22 | Recipient fanout over cap | Atomic diagnosable rejection, never silent partial audience |
| N23 | Events delivered out of subject revision order | History truthful; latest summary never regresses |
| N24 | Retention cleanup + delayed expired original | No recreation after receipt expiry; UI documents expired history |
| N25 | Reports terminal event retry/cancellation | One event per terminal transition; no screenshot/input payload |
| N26 | SSE chunks split UTF-8/CRLF/multiline data/comments | Correct protocol parsing or explicit failure/reconnect |
| N27 | Reverse proxy buffers or imposes timeout | Detect delayed heartbeats/frames; documented streaming config tested |
| N28 | Browser closed / reopened | Stored inbox still accessible; no claim of background Web Push |
| N29 | Fill pending, quarantine and successful-delivery paths independently | Each budget remains bounded; terminal rows deleted, quarantine evictions counted, incident writes continue within available DB resources |
| N30 | Saturate Admin message/recipient/receipt/byte budgets independently | New events receive atomic 503; duplicate receipt still returns 200; no premature receipt eviction or silent history truncation |
| N31 | User A creates run; B cancels; restart before completion | Immutable verified A remains sole candidate recipient; no JSON recipient spoofing |
| N32 | Scheduled run completes; initiator later disabled/deleted for manual run | Schedule emits no personal event; ineligible manual initiator gets no message |
| N33 | Queued cancellation, running cancellation, recovery and completion race | Exactly one event for each winning terminal state change; repeated requests emit none |
| N34 | Long reader/checkpoint pressure or storage reserve crossed | Notification admission stops with visible diagnostics; bounded queues remain; no claim that shared DB writes survive total disk exhaustion |
| N35 | Two sessions; logout one, then reset password; change roles | One sid only revoked at logout, all old auth epochs denied after reset, current-policy cache refreshed without trusting JWT grants |
| N36 | JWT expires between periodic SSE checks | No server-side frame admission after deadline; cancel registration without five-second grace; already transmitted bytes may arrive later |
| N37 | Cross-domain/module/instance signature, altered body/query, repeated nonce or retired key | Reject before admission; public headers cannot choose internal authority |
| N38 | Revoke user after permit issued, then submit new request | No new permit; old in-flight permit only within five-second admission boundary; no claim to cancel already accepted work |
| N39 | Rotate producer key with pending outbox; restart verifier | Retry immutable event using new signature/key; one inbox acceptance; old user permits fail new instance identity |
| N40 | Lease expires while old send is blocked; next subject revision waits | Only current lease token changes state/counters; backoff and live claim block later same-subject sends |
| N41 | Acceptance response lost at expiry; consumer absent; restart reconciliation | Duplicate counts success, 410 confirms expiry, unresolved deadline counts unknown once; original deadline and budgets preserved |
| N42 | Same-timestamp arrival after snapshot; 50 concurrent reads | New greater sequence remains unread; first read_at stable and one actual revision increment |
| N43 | Barrier before SSE registration or between list-data/revision queries | Registered first-frame and single DB snapshot yield included data or a newer hint; no old-data/new-revision pair |
| N44 | Socket stops reading during expiry/revocation | Cancellation removes hub/quota independent of body polling, bounded memory and no post-deadline admission |
| N45 | Fifth tab, 1001st connection, HTTP/error/oversize fixtures and BFCache restore | Atomic quotas, classified fallback without fast retry storms, bounded parser and fresh restored subscription/snapshot |

P6a's disposable Linux gate is `just verify-monitor-notification-runtime-linux`.
It starts the notification-selected Admin ingress and Monitor relay on real
loopback TCP listeners with fresh SQLite databases, drives one opened and one
resolved incident, and verifies durable receipt/inbox ordering plus same-event
deduplication and signature rejection. The same source basis builds an explicit
pure Monitor negative pair and verifies that its schema, selected config/API
contracts and runtime listeners contain no notification owner. Its manifest is
source-tree and binary-digest bound. The final Colima run is **Closed locally /
Passed** at `target/rz/monitor-notification-runtime/current/manifest.json`
on `linux/arm64`. The manifest binds the executed source tree, four binary
hashes, build provenance and verifier identity. It records opened and resolved
as stored, the repeated event as
duplicate, two receipts/messages/recipients, unsigned `400`, bad-signature
`401`, public-internal `404`, and zero notification schema objects/listeners for
the pure selection. The initial `401` diagnosis remains at
`target/rz/monitor-notification-runtime/failed-runs/20260907T124245Z-41694/`;
the published `current` manifest is the final result. The gate does not cover
systemd, production deployment, Reports, SSE, UI or sustained load. Its small
signing client uses Python's standard library because the pinned verifier image
already includes Python but intentionally contains no Bun or OpenSSL; the gate
installs no package or helper at runtime.

P6b source, focused-test and disposable Linux process acceptance is **Closed
locally / Passed**. It uses
file-backed WAL databases with at least two pool connections and verifies that
selected Reports migrate, validate, reopen and retain their separate
notification ledger, while no-default Reports reject that
ledger and have no outbox tables, delivery diagnostic or relay task/config
fields. Both selections retain the base nullable initiator provenance column.
Startup checks a nonempty existing database through a read-only connection
before opening the writable pool or running migrations; selected-to-pure and
pure-to-selected fixtures preserve the exact database bytes and object inventory
when this preflight rejects them.
Handler tests prove the initiator comes from the
verified delegated context and cannot be supplied in JSON. Transition tests
cover manual create/retry, scheduled `NULL`, queued and running cancellation,
startup recovery, repeated terminal actions and completion/cancellation races.
Each winning terminal transition must bind the persisted initiator, exact topic,
subject revision and immutable event bytes in the same transaction. Admin ingest
tests cover current Reports authorization, disabled/deleted users, revoked grants,
module disablement, duplicate IDs and cross-producer topic/key rejection.
Reports-owned relay tests directly exercise stale-lease fencing, subject order,
the 25-item worker slice of a 100-item batch, every terminal/retry/quarantine
result, expiry and reconciliation horizons, completion-clock backoff,
Retry-After bounding, deterministic 1-to-60-second jitter, all five gap counters,
the 60-second diagnostic limiter and startup accounting tamper refusal. The
Linux Reports runtime gate is `just verify-reports-notification-runtime-linux`.
The separate Reports UI-state Linux Chromium extension for the Runs delivery
card is **Closed locally / Passed** only when
`target/rz/reports-ui-state/current/manifest.json` validates against the current
checkout. The current manifest is authoritative for source identity, platform,
browser version, journeys, receipts, and screenshots. Its four manager/viewer
journeys bind deterministic aggregate API receipts and final SQLite equality;
error and forbidden Retry rendering remains a component-test seam. Native
systemd and production deployment are still unverified.
It uses fresh Admin and Reports databases, the real Admin gateway and
delegation signature, a real Reports worker/relay, real loopback TCP ingress and
the pinned Linux Chromium verifier. It covers manual success/failure, queued and
cooperative cancellation, restart recovery to failed/cancelled, immutable retry
initiator, scheduled `NULL` initiator silence, current Reports module disablement,
producer-commit-before-Admin availability, a committed response loss followed
by duplicate reconciliation, authentication rejection and the pure Reports
negative artifact. Current-grant revocation is covered by the Admin source
tests, not this Linux process gate. Controlled SQLite fixtures may establish
scheduled and crashed-running preconditions; every resulting terminal
transition is executed by the real Reports repository startup or worker path.
The published manifest
binds source and binary digests, exact receipts and negative-selection
evidence, while failed runs remain separate and `current` changes atomically
only after bounded container cleanup. The small transport probe uses Python's
standard library because the pinned verifier already contains Python but no Bun
or OpenSSL; it installs no runtime dependency.

P6a remains the final `linux/arm64` Monitor notification gate. The final P6b
Colima run is **Closed locally / Passed** at
`target/rz/reports-notification-runtime/runs/20260910T182747Z-31773/manifest.json`
on `linux/amd64`, SHA-256
`32d9a5d38c1a86c823da1f61064cf21c9a10ad25b525548a09ddd2a5a5da9552`. It binds
head `1c96269d341bd693b8d5486c4f42fb36fe6a314e`, dirty source tree
`dd299df31d9d2cfd524062f21546280b5340d3475bbf51ff3ef0a5af56eb007b`, build
provenance SHA-256
`0199b0630ac97d98c2b6c34ab8ece35a1be85115225a7e2f564b9a1ed30964e9`, and 27
exact receipts. Selected state contains eight `stored` and one `no-recipients`
receipt, eight messages, eight recipients, zero pending outbox rows and zero
values for all five gap counters. It records immutable retry initiator,
scheduled silence, outage backfill, a lost-response retry resolved as duplicate,
unsigned `400`, bad-signature `401` and public-internal `404`. Selected and
pure Reports both run as non-root `rz-reports` UID/GID `999`; their runtime,
database, log and artifact directories use mode `0750`. Pure Web binds
composition `8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b`,
digest `1ce67f44c7ae9f4f7a511ae3bd4c583b5166bf54c951fc0dc2245c3cc71957ec`, and
feature IDs `access,monitor`; pure selection records zero notification schema
objects, config, routes, tasks and listeners. The manifest/current record
binary hashes and build provenance but do not retain binary bytes, so they do
not support rehashing binary files. Failed runs, including stale-cache
diagnostics, are not published as `current`. This gate does not cover native
systemd, production deployment, SSE, UI or sustained load, and its controlled
SQLite setup is limited to scheduled and crashed-running preconditions.

## User interface acceptance

Use the current DESIGN.md components, navigation rules and language handling.
Before implementation, create a scoped UI contract for the bell, inbox and
monitor-only landing/access settings; no new visual system is needed.

- Full: all selected, authorized product journeys remain discoverable.
- Monitor: default landing is Monitoring for authorized users, otherwise an
  authorized access setting/profile; no Analytics/Reports/Admin product dashboard
  or message placeholders; access settings remain reachable.
- Inbox: loading, empty, error, retry, unread/read, retention, inaccessible
  subject, expired subject and permission-changed states are explicit.
- Notification click navigates through the existing router and current access
  gate; it does not embed a privileged URL or expose stale private content.
- Two tabs and a reconnect cannot leave an indefinitely stale unread count.
- Keyboard access and screen-reader names use existing accessible components.

## Measurement targets

Use a declared reference Linux VM (4 vCPU, 8 GiB RAM, local SSD) and record OS,
build target, proxy/TLS placement, composition/build IDs, node/user/event counts and
test duration. Provisional acceptance targets:

- 1,000 concurrent SSE connections, 100 event ingests/second for a 10-minute
  burst with one eligible recipient per event, and a 24-hour steady run with a
  declared lower event rate that fits retention/admission budgets. Benchmark
  larger recipient fanout separately; overload must show specified rejection,
  not be misreported as successful throughput.
- Healthy local ingress-to-durable-inbox latency p95 <= 1 second; online
  invalidation observed by the browser p95 <= 2 seconds.
- No unbounded queue or monotonically growing connection memory after clients
  disconnect; report measured RSS and queue maxima, not a presumed percentage.
- User/session revocation applied to active streams within 5 seconds.
- A 30-minute Admin outage at an event rate below the configured capacity
  replays all retained events without duplicate inbox rows.
- Install startup/idle RSS/disk size compared between full and monitor under
  the same hardware/build conditions, with raw numbers and no invented target.

Adjust the provisional target only through an explicit recorded design decision
with measured evidence; passing a smaller test is not passing the original one.
Large connection counts from the earlier generic SSE proposal are not product
requirements or verified capacity.

Each load scenario begins with a separately recorded fresh test database unless
its fixture explicitly requires prior retained data. Use 0.5 event/second with
one recipient for the 24-hour run (43,200 messages), and 10 events/second for the
30-minute outage (18,000 producer events), with a fixed bounded payload whose
charged bytes demonstrably fit the respective budgets. The 10-minute burst has
60,000 messages. Do not concatenate these fixtures and claim the 100,000-message
budget can hold all of them. Independently saturate row/byte/fanout budgets and
verify rejection. Retention capacity is a storage envelope, not a promise to
sustain the burst rate for 30 days; at the initial message cap the 30-day average
must be below approximately 3,333 stored messages/day before other limits.

The 100 events/second target is aggregate across selected producers, and means
durable unique acceptance, not offered requests or successes with dropped errors.
After each fixture, verify exactly 60,000 / 43,200 / 18,000 expected receipts,
messages and recipient rows respectively, no duplicates and no unexplained
rejections/producer gaps. Duplicate retries are counted separately. Report offered,
accepted, retried, rejected and unresolved totals alongside latency, including
all failures. Run the burst beyond nonce-retention duration, with unique nonces;
unexpected nonce/rate/capacity/auth refusal fails the target rather than shrinking
the latency sample to successes. An overload fixture separately proves rejection.

## Evidence and readiness

Each released selection records source+dirty basis, dependency and file
inventories, schema/API/Web digests, tests executed, native installation,
browser/proxy observations and limitations. Build success, static absence,
browser behavior and actual deployment are reported separately.

Design is ready for implementation only after review issues are reconciled.
Runtime acceptance requires the above relevant executable evidence; ten AI
reviews cannot substitute for it. Implemented rows require their named gates;
unimplemented rows remain pending.

Monitor P4 protocol evidence is produced by `just verify-monitor-protocol`,
the Controller and Agent focused tests, and route-level conformance tests.
They build both binaries, compare descriptor output, exercise gateway delegation
plus the Agent token, and check the fixed fencing fixture corpus.
Analytics selected protocol evidence compares the Admin and Insights canonical
delegation descriptor before dotenv or runtime initialization. Both closures
require matching descriptor output and digest from their real peers.

| D51 | Run a selected API producer with empty environment and temporary cwd; mutate a route, owner, namespace or artifact byte | Contract-only binaries remain deterministic; the producer rejects omissions/additions and records only verified artifact bytes |
| D52 | Produce selected schema evidence from final initialization migrations; mutate owner, digest, canonical bytes, file identity or artifact inventory | Manifest accepts only stable canonical schema evidence for the exact Admin/Monitor or Admin/Insights owner set and derives every schema/data ID from it |
| D53 | Build all three selected Config feature sets and run each descriptor command under empty environment from a temporary directory | Exact owner/consumer/key surfaces are emitted deterministically and no value or secret literal is present |
| D54 | Produce Server and Agent config artifacts; mutate composition, owner, field metadata, canonical bytes, inventory or file identity | Exact class-specific descriptors are required; Manifest derives `configDigest` from stable artifact bytes and rejects caller hashes |
| D55 | Pin signed Monitor Server and published Agent fixtures; alter signature, tuple, endpoint, Agent protocol, profile bytes or replace the profile with a link | Pinning fails closed before a profile write; a valid root-owned profile gates deployed production Agent startup before logging/network work; the isolated development command remains runnable without `/opt/rz` installation state |

| D08e | Verify a selected-server container export, then delete or replace its export root before captured-byte staging | Staging and its unsigned manifest retain only verified server binaries/contracts/flattened Web and succeed without reopening the export; witness and metadata remain absent. |

### P8f selected-server browser admission

P8f requires an explicit reviewed `monitor` or `monitor-notify` selection. The signed
certificate/export validates product source identity; verifier checkout identity and
hashed verifier inputs are separate receipt provenance. Cross-preset input must fail
before Chromium/output creation. The notify bootstrap wrapper has source/static coverage
for the existing four bootstrap/SRI/retry cases and uses a separately retained notify P8e
container; it has no monitor-notify business journey receipt, so browser, load and release
readiness remain Not verified.

The exact monitor-notify bootstrap run is
`target/rz/p8f-notify-bootstrap-browser-20260910-r1/manifest.json` with SHA-256
`1785ad619ee750fb705771fdf1b3aeefa4ac773a51414e0b2f7a4d808de07c87`.
Headless Chrome 153 passed success, binding-mismatch, binding-network-failure and
SRI-entry-failure cases; only the success case executed the selected entry. The
receipt binds product source `git:3d6aeec719d5acc9b26b114ceedef5ac9c700444 tree:a1f5e7e55952fd22d36ca8228c93ee5109a1316cf0e76b72cb7fa5f456af1b9a state:dirty`
and verifier source `git:f4921380a3c7c074ac9810855b87c5a73fcb9352 tree:c191cd5eb8cde27c8d95235a2fe1821e5d724399ad4089410edd90921d75edfa state:dirty`
separately. Its retained runtime remained byte-bound before and after the browser
gate, and the owner-labelled container was removed afterward. The paired fresh
P8e runtime record is
`target/rz/p8f-notify-bootstrap-native-20260910-r1/monitor-native-runtime-evidence.json`
with SHA-256 `870f2dec63ed679c18c957a1b7285aed0f747d193a22927e2a0b4879fe92551b`.
This closes bootstrap/static browser behavior for the old signed tuple only;
authenticated Monitor, alert, inbox, SSE, persistence, load and deployment
acceptance remain Not verified.

### P8f-B monitor-notify business journey

P8f-B must prove a signed retained notify runtime from Agent report through Monitor alert and Admin inbox, then an authenticated same-origin Bearer SSE browser journey with no URL secret, selected/absent routes, detail deep link, refresh persistence and read state. Its receipt is canonical and binds P8e/P8f-A sidecars; `browser:true` does not certify load or release readiness.

The current R9 P8f-B closure is the 50-file export at
`target/rz/p8fb-r9-monitor-notify-export-20260910T125008Z`, bound to dirty source
`git:cb357c671b88ecfaf5987a30b7de39f0ef125a0d tree:a419bf42ace680f08ff6b3f25631aa7c9516c4fe68324377a6352b510e29d427 state:dirty`,
build `d1aab233941aae6b513733c5f6f4f78a5e2cd264cdfc32e937692a1be2dd52c5`, composition `0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d`, signed archive/manifest/envelope/certificate
`045b0094d8822783c923bc0402fdc06925e54fd51ad12907790dbc2e12dd313c` / `bd0355b61da631780f98bfdd6d049b665e7e36c40d521612e4e6272bc06efaa7` / `0f50bcbafa225a7bd74c0b4f545990fe08f1dcefe95e5a69ae1744349c790ccb` / `d62a6d9db4d71369636783e43205ea1b75558a16c4192e233f1b8331e94822cf`, native evidence
`target/rz/p8fb-r9-native-20260910T125008Z/monitor-native-runtime-evidence.json`
(SHA-256 `bdaea45a3b5b78e53501c36ecbc0b8aaa4512f5b8d5fea3b1739ab97745443f8`) and
business receipt `target/rz/p8fb-r9-business-20260910T125008Z/receipt.json`
(SHA-256 `3ab76a4af846da1e56fff144d942fd1da52888440195f8542b50f7026a066c89`).
Chrome 153 passed the signed-cert/systemd-PID1 journey through witness Agent sequences
2–4, incident and inbox reconciliation, no-URL-token Bearer GET SSE, strict same-clock
ordering, Monitor-visible with Insights/Reports-absent routes, detail deep link/read/reload,
and 1440×900 visual screenshots. It records `browser:true`, `load:false` and
`releaseReady:false`; P8g load and production deployment remain Not verified.

### P8g pure-Monitor exact-artifact load certification

P8g accepts one fresh container only when its P8e native-runtime evidence and
P8f browser receipt bind the identical signed release/source/build/composition
and runtime Admin tuple. Its fixed dataset is 100 nodes with four disks each and
a fixed sentinel. Every response must be `200`, `code==0`, contain all and only
100 unique node IDs and exactly four disks per node. Login is setup, never a load
sample. Each of two lanes uses 32 closed-loop workers, four warmup requests per
worker, then
60 seconds of monotonic offered requests with at least 3,200 completed requests;
each request has a 2-second hard timeout, no retry, and measures submission
through full body. Each lane requires zero HTTP/transport/timeout failures,
nearest-rank p95 <=500ms and p99 <=1000ms, then a 5-second drain and 30-second
quiet observation. Each phase has exactly one pre-work and one post-work usage
snapshot; probes are excluded from its work duration and there is no periodic
sampling cadence. The quiet baseline uses the post-quiet current RSS and
`pidsCurrent` values;
the run therefore exercises the Controller's bounded Nodes snapshot cache while
the Admin gateway still performs authoritative authorization on every request.
The admitted binaries use the shared default database pool of one minimum and
eight maximum connections; an environment override is not part of the certified
fixture.
Each snapshot pair is strictly ordered and spans the recorded phase work duration;
the receipt rejects a missing, third, reversed, too-short or identity-discontinuous
pair. Non-fault phases retain the same Admin and Monitor identity; the fault phase
allows only the separately recorded Monitor restart and requires unchanged Admin.
`pidsCurrent` is used for quiet and cross-lane bounds, while `pidsPeak` and
`memoryPeak` apply the global cumulative limits. Other Docker operations have a ten-second bound. The receipt records measured lane, drain, quiet, fault, and
recovery work durations; the two lanes require at least 60s, each drain 5s,
each quiet observation 30s, and recovery 10s. It records the fixed eight
authoritative inputs, all twelve P8e sidecars, and fault/recovery readings in
the overall maximum and event-drift comparison.
lane two may add at most 16MiB RSS and two processes over lane one.

The disposable container has exactly four CPUs, `memory.max=512MiB` and
`pids.max=256`. Combined Admin+Monitor RSS peak is at most 384MiB and process
peak at most 64. `memory.events` max/oom/oom_kill and `pids.events` max do not
grow; receipts retain cgroup memory/pid samples plus VmHWM or smaps_rollup.
The fault stage records stop, listener disappearance, a new listener PID, and
healthy registry in monotonic order. Stable outage permits only 503/code 40001;
boundary in-flight results are separate. The verifier freezes Monitor before
launching those requests, admits the boundary only after four Monitor-side
established sockets contain unread request bytes (idle keepalive sockets do not
count), then freezes Admin so those responses cannot finish early. It kills the
frozen Monitor MainPID, confirms that exact PID is absent, records the
confirmed-dead boundary, stops Monitor, and only then thaws Admin. The four
dedicated boundary requests have a ten-second timeout; normal load requests keep
their two-second timeout. This leaves no thaw-to-stop success window. A 401, 500, timeout, stale 200,
unchanged PID, or wrong executable fails. Recovery is 32 workers for 10 seconds
with at least 512 samples and the same latency/zero-failure bounds. Pure Monitor
has no SSE owner: the generic 1,000-SSE target is `notApplicable`, proven by
signed API/Web/schema absence and a runtime SSE-route 404.

R8 supplies the current pure-Monitor P8g local exact-artifact load evidence on
Colima Linux/amd64: source
`git:a4a8beed33e842857a535cd051902dea3a4bf5c6 tree:c4fa33e1d205ae6e354f282346e25a4af08248ce0d3578fdd6fbed0b66f074bb state:dirty`,
export `target/rz/p8g-r8-monitor-export-20260910T145456Z`, build
`49630979fee226a3f1850d09a0296e752a1a4bce8fc67d4f8751e260a887e492`,
composition `8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b`,
and signed archive/manifest/envelope/certificate
`6ba42a9b005710500bb33f6160fcbd265f8ab0c1e93f694c28e8a4b73725b7ca` /
`3e612c631a806abb6341b1856c496e0616b6e5f8a858ce9f953fde7f9fca9d65` /
`8bab407380e6d784546fd65951ea76124723658fb03bc9992ca9d243e8a1c03f` /
`3c0e8b2b8eb2180939cea36a93c1bb2a7c435ba8cd7c28750e6796d3a01cd314`.
Its native/browser/load receipts are
`target/rz/p8g-r8-native-20260910T145456Z/monitor-native-runtime-evidence.json`
(`e9610e21899c91945525f2b4a54b9ded8eb7f71e8714ba77ac115e4d9ce2c8db`),
`target/rz/p8g-r8-browser-20260910T145456Z/manifest.json`
(`1b2504dd188ca890dbcba110d7218443c10b46fb4c6c00f7a683bc24dc997213`) and
`target/rz/p8g-r8-load-20260910T145456Z/monitor-load-evidence.json`
(`34c26a895b9236085ccfbbadcc521c94c52d72f5a52a33bbc0cf0acfa7fe32aa`).
The measured lanes completed 12,389 and 12,661 requests with zero failures and
p95/p99 227/270ms and 220/271ms; recovery completed 2,138 with zero failures
and p95/p99 224/268ms. Overall peaks were RSS 28,614,656 bytes, VmHWM
45,490,176 bytes, memory 74,985,472 bytes and 42 processes. The fault order was
stop, 503/code-40001 listener-gone, listener-ready, registry-healthy. SSE was
404 with signed empty forbidden API/Web/schema/service sets, and retained runtime
cleanup completed. This is a local `load:true` closure only; production deployment
and `releaseReady` remain **Not verified**.

Receipts are strictly parsed at publication time by the same-process validator; historical
pure-Monitor load receipts recorded the equivalent boundary under the previous `pureMonitorSse`
key and are not re-parsed by the current validator (`schemaVersion` stays 1; the field rename
tracks the generalized per-preset shape).

The exact `node-agent` PID1/service-restart evidence is
`target/rz/agent-pid1-20260914T234321Z/manifest.json` (SHA-256 `388f1d49197164cecdc14e985a5f487144e676a393db6926433d3b7883210813`). One disposable `linux/amd64`
systemd PID1 container applied the installer-test-signed agent release with the
current-source musl CLI, created the service account, prepared access, pinned the controller
profile, and activated `rz-monitor-agent.service`; the Type=notify unit reached active only
after its first confirmed report delivery through `https://monitor.internal` (a locally
generated CA trusted in the system store fronted by socat), MainPID executable identity
matched the payload binary, `systemctl restart` produced a changed PID over the same binary
sha with post-restart sequence delivery, stop/start returned to active, and a second
`activate-monitor-agent` re-run was idempotent. The receipt records its limits
(installer-test keys, single host, socat TLS front, no production TLS or remote host) and
binds the CLI/admin/monitor/agent digests plus both fixture tuples. This run also surfaced
and fixed a latent product defect: `agent_manifest` read `marker["buildId"]` and
`marker["artifactClass"]` from the publication marker whose identity tuple actually nests
inside its journal, so `prepare-monitor-agent-access` failed after every fresh apply since
the marker-format change; the fix reads the journal tuple and ships with a shape regression
test. Physical Linux/Windows agent hosts and production TLS remain **Not verified**.

The exact `monitor-notify` P8g load evidence is
`target/rz/p8g-notify-load-20260914T144026Z/monitor-load-evidence.json` (SHA-256 `5bf25e6b869a0974b9e0f411d7573e5f878523902b88186ad5d7f7dd61b6747b`). One fresh
retained PID1 deployment of the signed old-export build
`d2c4e0f452c7400927059bc499822ef1218a9c47547ca88eb25706bbfa1426bd` re-ran the P8e native gate
and bootstrap browser admission, then certified load: two 60-second 32-worker gateway lanes
offered 12,398 and 11,781 requests at p95 234-253ms and p99 279-298ms with zero failures,
five-second drains and thirty-second quiet phases drifted under one megabyte at stable pids,
the controlled freeze/kill fault boundary failed reads with 503/40001 and recovered through a
same-binary rz-monitor restart followed by a 2,021-request recovery lane, cgroup OOM/kill
counters never moved, and end-of-run re-hashes reproduced every input plus the signed
composition check. The receipt records the live Bearer `text/event-stream` SSE boundary and a
notifications-present signed composition (`apiOwners:3`, `schemaOwners:4`,
`notificationWebRoutes:1`) where the pure-Monitor receipt records SSE 404 and signed absence.
`load:true`; `releaseReady` remains false and production deployment stays **Not verified**.

The exact `analytics` P8g load evidence is
`target/rz/p8g-analytics-load-20260914T100047Z/receipt.json` (SHA-256
`65109f1cec248def7857d7375050f8c5e4431e2f6d6109dfa917753221682a0d`). One fresh retained PID1 deployment of the same signed build served the
published admin endpoint while a host-side client ran two rounds of overview/events read
lanes (32 workers, 60s, minimum 3,200; observed 7,108-9,855 offered at p95 248-337ms and
p99 288-366ms), the write lane at the product's documented per-origin admission ceiling
(30 requests / 300 events per 60s; fifteen 20-event batches inside each window plus one rolling past it, 69-73ms), five-second
drains and thirty-second quiet phases with stable service owners and under one megabyte of
quiet RSS drift, a real `rz-insights` stop that failed reads with 503 and recovered through a
same-binary restart in 2.7 seconds followed by a 1,039-request recovery lane, unchanged cgroup
OOM/kill counters (enforced at runtime across the first and last snapshots), and end-of-run
re-hashes of every input plus the P8e/P8f references; the password and project-key inputs are
run-time temporaries deleted with the work directory, so only their in-run hashes are
recoverable from the receipt. The
receipt records `load:true` with `releaseReady:false`. Production deployment remains
**Not verified**.

The exact `analytics` P8f browser evidence is
`target/rz/p8f-analytics-business-browser-20260914T023318Z/receipt.json` (SHA-256
`00e75e6163fe7ed39824b4a594e9e990b8ffbbee4e77e9b367d6ab7fecf1de29`). One fresh retained PID1 deployment of the same signed build
`8f1a2e993977edcad7390f445eb201ab53e37b29dbfb288ef46f50c386eabc79` (retain native evidence
`target/rz/p8f-analytics-native-retain-20260914T023318Z/analytics-native-runtime-evidence.json`)
served the published admin endpoint on a loopback port; a headless Chrome 153 CDP session at
1440x900 (plus one 390x844 zh-CN mobile case) executed seventeen journeys: deployment identity
bindings, unauthenticated route redirect and API denial without internal-detail or credential
leakage, wrong-password and shipped-default-password rejection, owner login, Analytics-only
menu navigation, overview empty/loading/populated states with real ingestion of one
collection-policy update and 24 tracker events, details filter auto-query with page-one reset
and page-two pagination, a real `rz-insights` stop outage rendering the reload error state with
restart retry recovery, a full service-restart recovery, Monitor/Reports API 404s and absent
module surfaces, and logout session revocation with re-login. The receipt records 315 network
requests, zero console errors and seven hash-verified screenshots, carries no credential or
token material (the session token appears only as a SHA-256 prefix), and its `runtime` reference
points to the unchanged P8e evidence `0d6d8c3df93bb7c1d36bc1c6316df4f8a7d0e7f839fb427bbc9b9d8e157682b1`
rather than re-inferring it; `browser:true` while `load` and `releaseReady` remain false. A
separate process re-verified the persisted receipt, its screenshot hashes and the runtime
reference. Load and production deployment remain **Not verified**.

The exact `analytics` P8e native runtime evidence is
`target/rz/p8e-analytics-runtime-20260911/analytics-native-runtime-evidence.json` (SHA-256
`0d6d8c3df93bb7c1d36bc1c6316df4f8a7d0e7f839fb427bbc9b9d8e157682b1`). Because the Insights service gained its offline database command surface and
installation-identity binding, this gate runs a fresh chain: the export at
`target/rz/p8e-analytics-container-20260911T052505Z` binds dirty source
`git:1060952abae2f16d0bb0639308d6ba0f53e30252 tree:562c2a6040562febdd648a0fde6c71b9e12885ec2b74b860f5d79d8ec59bdb0d state:dirty`, the signed release
and certificate bind build `8f1a2e993977edcad7390f445eb201ab53e37b29dbfb288ef46f50c386eabc79` under key `p8e-analytics-20260911`, and the private key was
deleted after publication. Inside one disposable `linux/amd64` systemd PID1 container the gate
rereads the published certificate, runs `rz verify`, dry-run, fresh `apply`, `install-status` and
`activate-analytics-server`, then proves enabled/active `rz.target` with `rz-admin.service` and
`rz-insights.service`, identity-bound fresh Admin and Insights databases, owner login plus
default-password rejection, Monitor and Reports absence, restart plus both service start orders,
and MainPID executable inode/digest equality. A separate host process revalidated the persisted
evidence. `browser` and `load` remain **Not verified**.

The exact `analytics` P8c/P8d evidence manifest is
`target/rz/p8cd-analytics-evidence-manifest-20260911.json`. It binds the retained
P8b export identity `git:6c12c7227e5ec2519c6fee506a9d0407c7494448 tree:df92f84c8b22c6a73224c71082001921750800e64888f5e61455831c00362900 state:dirty`, composition `62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40` and build `e10b0fd6f289f5035b4d4147d0b47f77be6eeadfa1d419ede6eb05570aa2f333` to archive `0eae6e53a591c740a66051f6d32a8f836ddecfae73b417774d66fec0b68a3297`, manifest `a87435008418c4cce05ec15ec7807673b4c2984aede41dd7178ac88d81bb9fad`, envelope `3a045610c2328657e94724651fcb9cd0daeab3f3d98cbcae8e57ccabb7dbd183` and certificate `221f07f0fc5517f32dff3b56dfbb22189fc9a08c06d106a6a2f026e053e24583`. The host and linux/amd64 verifier JSON SHA-256 values are both `b718e05cba0b16f67997ff7f4e0aee17750e56a204196c48c1ebf75633221d79` and their bytes match exactly; the linux/amd64 run executed the current `rz` from the full export branch inside the digest-pinned `debian:bookworm-slim` image with network disabled, a read-only rootfs and no-new-privileges. The private key was deleted; only public key SHA-256 `9596184838e7a636ff8816a70c4a198428d8fd5840f5d9c5bd79a164373d74e0` remains. This records one old-export tuple, not a rebuild of current source; all literal `runtime`, `browser`, `load` and `releaseReady` flags remain false, and it proves neither native/PID1, browser nor load behavior.

Analytics P8b step 3 is **Closed locally / Passed** on the real Colima BuildKit export at `target/rz/p8b-analytics-container-20260911T020403Z`: the exact Linux/amd64 container identity, six retained restricted commands and source/manifest/provenance equality all verify, the host validator accepted the export before and after the restricted pass with unchanged source identity, and a fresh process re-verified the persisted receipt (SHA-256 `c2ba3766b33aa301c00aa59109c18d98db34d780ffbd695c897f12a4602d36d4`). Runtime, signing, installer, browser and load remain **Not verified**.
