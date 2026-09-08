# Implementation sequence and acceptance

Status: planned tests and implementation gates; only evidence explicitly named
below is claimed complete. Native staging tests cover exact inventory, stable
reads, no replacement, publish locking, temporary cleanup and manifest snapshot
binding.

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
sets. The current admitted set is `monitor` and `node-agent`.
`monitor-notify`, target `full`, `analytics`, `reports`,
`current-full-regression` and `custom` return a machine-readable blocked result;
`--require-ready` exits nonzero.
Admission is not a certificate and cannot satisfy any native, browser, load,
installation or release gate.

## P8b Monitor source-build certificate contract

The pending Monitor P8b issuer must accept an admitted production `monitor`
selection and one explicit release binary root. It must reject an implicit Cargo build, a
debug/rebuilt contract producer, a different composition, missing or extra
server binary, fixture helper on the positive path, noncanonical contract or
manifest, and any archive or envelope digest mismatch. It records one immutable
`source-build-manifest` after staging and local detached-signature verification.

Its structural parser rejects unknown fields/layers, non-Monitor selections and
every attempt to set `runtime`, `browser`, `load` or `releaseReady` true. The
expected-input verifier separately rejects altered source/build/artifact identities.
The issuer must capture those inputs itself; CLI-provided digest or toolchain
claims are not authoritative. Successful P8b evidence still requires
separate Linux `rz verify`, dry-run and fresh apply verification, then native
runtime, browser and load evidence before a release gate can open or a `current`
pointer can change.

The source-build certificate closure verifies only the canonical schema against
authoritative inputs and makes contract extraction require an explicit binary
directory. Its issuer and crash-safe publication remain **Not implemented /
Not verified**. The separate container-export closure now supplies stable
same-batch BuildKit and Linux artifact evidence, but does not issue that
certificate.

The first P8b container closure is verified separately from certification. Its
static boundary test must reject a non-Linux/amd64 build stage, an omitted Monitor
source-identity requirement, a target other than x86_64 musl, a missing explicit
binary root for protocol extraction, omission of any Web/API/schema/config/protocol/native
output, a misplaced or extra server/witness binary, a missing output manifest or
provenance record, and a build path that omits the exact command arrays. Its
artifact test creates an exact temporary payload and rejects missing contracts,
unexpected payload paths, links, unsupported modes and non-Linux/amd64 provenance.
Passing these tests establishes source and static producer closure only. The
local P8b gate additionally rebuilds through Colima BuildKit for Linux/amd64,
requires identical source identity before and after the build, validates the
whole export on the host, and executes seven selected binary contract commands
in a networkless read-only container with all capabilities dropped. Six API,
config and protocol outputs must match the retained contracts; the Agent config
witness must satisfy its selected structural policy. The host then validates the
export again. The accepted local export is
`target/rz/p8b-container-export-verified`.

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
| D08b | Pass empty, unknown or custom Docker `DISTRIBUTION` | Docker fails before Web or binary build; omitted argument resolves only through the declared `full` default |
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
| D27 | Keep buildId but change Web bytes/digest | HTML/backend webDigest mismatch blocks bootstrap; final file signatures and entry integrity also checked |
| D28 | node-agent with Web/server fields; server missing schema owner | Discriminated manifest validation fails before installation writes |
| D28a | Canonical manifest producer/validator receives unknown/duplicate/escaped/link/mode/size/hash fields, server-Agent mixing or a digest without its actual source discriminator | Reject before archive/signature/install; canonical resolver, selected-Web file table and binary byte digest inputs are the only accepted sources |
| D28b | Monitor recovery journal has a changed archive/envelope/manifest tuple, unsafe metadata, an occupied root, a replaced empty/sentinel/wrong-nonce work root, a copied marker on an incomplete root, or a final root with missing/extra/truncated/replaced payload or retained triplet | Reject without modifying the root. The exact tuple converges after either cleanup fault only after the completion marker and manifest-derived final layout, payload digests/modes, retained triplet and current link all verify; no continuation marker or sibling journal remains. |
| D28c | Run `rz verify` and `rz apply --dry-run` against valid Server and Agent triplets; alter a tuple field, PEM/key ID, member header/hash/path, destination or input type | Verification rejects before destination writes; dry run leaves no root; Linux root apply publishes only selected payload/layout and retained triplet; systemd and DB are not started or initialized |
| D28d | Activate a fresh signed Monitor server under PID1 with root-only config and a unique one-time owner secret; retry, alter config/unit/source/database metadata, preinstall or enable Reports/Insights residue, swap in a same-schema foreign DB, interrupt either database rename or each daemon-reload/start/readiness/enable/marker/marker-dirsync/journal-remove boundary, or inspect excluded files | Only Admin+Monitor config/unit/schema paths exist; the owner logs in with the supplied secret, not a migration default; exact retry and every same-tuple interruption converge; invalid ports/password/path parents and unselected service residue fail before account or destination writes; schema corruption and foreign installation identity are rejected without DB mutation; conflicts/failures leave no false ready marker and unselected services/config/database files remain absent |
| D29 | Same DDL but different build or data semantics; legacy compatibility field supplied | Existing-root reuse and unsupported fields rejected; use final fresh baseline |
| D30 | Excluded route has lazy side effect/glob; excluded asset exists in source public | Selected generator directory, Vite graph and copied file inventory exclude all sentinels |
| D31 | Remove used backend operation or import excluded client through shared selector | Frontend operation/schema pairing or owner gate fails before release |
| D32 | Cached stamped full HTML/old missing entry against current monitor backend | Anonymous binding prevents credentials/business calls; at most one cache-busting reload, then matching login or explicit error |
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

The final P6b Colima run is **Closed locally / Passed** at
`target/rz/reports-notification-runtime/current/manifest.json` on
`linux/arm64`. The manifest records 25 exact receipts and six manual terminal
classes; selected state contains eight `stored` and one `no-recipients` receipt,
eight messages, eight recipients, zero pending outbox rows and zero values for
all five gap counters. It records immutable retry initiator, scheduled silence,
outage backfill, a lost-response retry resolved as duplicate, unsigned `400`,
bad-signature `401` and public-internal `404`. Selected and pure Reports both
run as non-root `rz-reports` UID/GID `999`; their runtime, database, log and
artifact directories are owned by that identity with mode `0750`. The pure
selection records zero notification schema objects, config, routes, tasks and
listeners. The first root-run Chromium failure is retained separately at
`target/rz/reports-notification-runtime/failed-runs/20260907T162006Z-31299/`;
the published `current` manifest is the final result. This gate does not cover
native systemd, production deployment, SSE, UI or sustained load, and its
controlled SQLite setup is limited to scheduled and crashed-running
preconditions.

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

| D51 | Run the Monitor selected API producer with empty environment and temporary cwd; mutate a route, owner, namespace or artifact byte | Contract-only binaries remain deterministic; the producer rejects omissions/additions and records only verified artifact bytes |
| D52 | Produce selected schema evidence from both final initialization migrations; mutate owner, digest, canonical bytes, file identity or artifact inventory | Manifest accepts only stable canonical schema evidence for exactly Admin and Monitor and derives every schema/data ID from it |
| D53 | Build all three selected Config feature sets and run each descriptor command under empty environment from a temporary directory | Exact owner/consumer/key surfaces are emitted deterministically and no value or secret literal is present |
| D54 | Produce Server and Agent config artifacts; mutate composition, owner, field metadata, canonical bytes, inventory or file identity | Exact class-specific descriptors are required; Manifest derives `configDigest` from stable artifact bytes and rejects caller hashes |
| D55 | Pin signed Monitor Server and published Agent fixtures; alter signature, tuple, endpoint, Agent protocol, profile bytes or replace the profile with a link | Pinning fails closed before a profile write; a valid root-owned profile gates deployed production Agent startup before logging/network work; the isolated development command remains runnable without `/opt/rz` installation state |

| D08e | Verify a Monitor container export, then delete or replace its export root before captured-byte staging | Staging and its unsigned manifest retain only verified server binaries/contracts/flattened Web and succeed without reopening the export; witness and metadata remain absent. |
