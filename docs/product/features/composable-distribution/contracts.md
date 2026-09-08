# Distribution, event and inbox contracts

Status: normative target with incremental implementation. Each implemented
section names its executable evidence; unspecified distribution, signing and
installation paths remain pending. The release manifest accepts one strict
staging reference (root, tuple, files and inventory digest), then verifies its
single stable snapshot has only selected binaries, contracts, native units and
server `web/**`.

## Distribution identities

Three separate artifacts have different owners:

1. Authored selection/catalog: build inputs only; owner is the repository.
2. Resolved build plan: generated exact dependency closure and build actions.
3. Signed release manifest: actual installed files and hashes; installer input.

Do not use the build catalog as an independently maintained route catalog.
Routes and permissions are exported from the compiled Rust registration.

```json
{
  "manifestVersion": 1,
  "releaseClass": "production",
  "releaseVersion": "<workspace version>",
  "target": "x86_64-unknown-linux-musl",
  "artifactClass": "server",
  "preset": "monitor-notify",
  "capabilities": ["access", "monitor", "notifications"],
  "compositionId": "<sha256 of contract version, artifact class and feature closure>",
  "buildId": "<sha256 of exact canonical build plan>",
  "sourceIdentity": "<commit plus dirty-content identity for local artifacts>",
  "services": ["admin", "monitor"],
  "configDigest": "<selected config artifact sha256>",
  "nativeLayoutDigest": "<selected native layout artifact sha256>",
  "protocolArtifactDigest": "<selected protocol artifact sha256>",
  "configOwners": ["access", "monitor", "notifications"],
  "schemaFingerprints": {"admin": "<sha256>", "monitor": "<sha256>"},
  "dataContractIds": {"admin": "<descriptor sha256>", "monitor": "<descriptor sha256>"},
  "agentProtocolContractId": "<selected Monitor wire-contract sha256>",
  "webDigest": "<sha256>",
  "files": [{"path": "bin/rz-admin", "size": 123, "sha256": "<sha256>"}]
}
```

The example's file list is abbreviated, not a valid package fixture. The real
manifest enumerates every allowed archive file, mode and content hash. The
signature covers the complete manifest and package bytes through one documented
signature procedure. Fresh installation validates composition, target, current
schema fingerprints and data contract IDs; these do not authorize old-data reuse. `buildId` identifies the exact build plan and changes for
a source/version/toolchain change; it neither proves equal output bytes nor is
an upgrade compatibility requirement. Compare exact `webDigest` for Web pairing.
File and Web content hashes are separate manifest outputs, excluded from their
own build-plan input. A version string alone cannot authorize replacement.

The manifest is a discriminated union, not a server object with empty Agent
fields. Both classes require version, target, class, composition/build identity,
file inventory and signature. Historical rollback compatibility fields are rejected.


### Canonical manifest core

P4's canonical core accepts only UTF-8 canonical JSON: object keys sort by UTF-16 code units (the ECMAScript/JCS ordering), arrays retain their declared order, and no whitespace is emitted. Its
`selectionDigest` is SHA-256 of the canonical resolver plan, `webDigest` is
SHA-256 of the canonical `{path,sha256}` table read from verified selected-Web files, and each binary/file digest
is SHA-256 of bytes read from the named producer output. Digest records carry a
required source discriminator (`resolved-selection`, `selected-web-files`,
or `binary-file`); handwritten values with a missing or cross-purpose source are
rejected before a packager can consume them.

The manifest producer accepts only builder-created, current-user-private, quiescent
staging. Node has no portable `openat`: the producer therefore uses no-link directory
and same-descriptor identity checks as an accidental-change guard, not as a general
adversarial archive reader. Untrusted tar extraction safety belongs to the installer
closure.

Each `files` entry is exactly `{path,type:"file",mode,size,sha256}`. Paths are
relative POSIX paths with no empty, dot or parent segments; links, directories,
devices and implicit archive members are not valid file entries. Mode is exactly
`0755` for `bin/*` and `0644` for other members. The canonical core rejects
unknown fields, duplicate paths, noncanonical digest spelling, invalid sizes and
out-of-class binary inventories. It does not sign, archive or install anything;
those remain P4 producer stages and the release gate remains closed.

| Field / behavior | server | node-agent |
| --- | --- | --- |
| capabilities | access plus exact selected product closure | Exactly monitor-agent; no access/server product IDs |
| services | admin plus exact selected controller services | Exactly monitor-agent |
| schemaFingerprints/dataContractIds | Required for every selected DB/artifact owner; missing owners rejected | schemaFingerprints forbidden; dataContractIds only for Agent-local config/state format |
| webDigest | Required and nonempty | Forbidden |
| inventory/bootstrap | Authenticated installation API and selected Web | Local CLI only; no Web/API listener |
| fresh installation identity | Exact signed build/class/composition/target and current schema/data contracts | Exact signed build/class/composition/target and current Agent config/protocol |

Docker Monitor producer output is physically separated before any archive work:
`/out/server/bin` contains exactly `rz-admin` and `rz-monitor`; `/out/agent/bin`
contains exactly `rz-monitor-agent`. Full remains `/out/bin` with its five binaries.
This is a build-output boundary only; manifest, archive, signing and installation remain P4 work.

No omitted field is inferred as an empty server contract. Cross-class packages
and extra owner fields are rejected before install writes.

For Monitor P4, recovery is only a root-owned fresh-install journal: it may resume
interrupted payload publication for the same archive, signature envelope and
manifest tuple in the same fresh root after revalidating every input. The journal
contains a random work-root nonce and is a root-owned descriptor-relative sibling
of the destination, so a successful rename never carries it into the final root.
Before any payload write, the installer fsyncs a root-owned descriptor-relative
work-root marker containing the same nonce; it deletes a prior work root only when
both nonce values match. The durable final publication marker contains the complete
journal tuple and nonce as its completion credential. A same-tuple retry can finish
cleanup after either marker or sibling-journal removal; a completed final root with
no sibling journal is idempotently accepted only for that exact tuple after the
manifest-derived payload file set, digests and modes, retained manifest/envelope/key
and exact `current` link also verify. An occupied
destination, changed tuple, unsafe journal or replaced work root fails without mutation. It creates no
`rz-recovery.service`, does not reuse the full DeployService, and never starts a
unit, initializes, rolls back or restores a database. `rz.target` contains only
Admin and Monitor. An online-update recover executor is a later closure. Agent-to-controller
wire compatibility is also checked against the advertised protocol version;
shared schema identity does not establish network compatibility.

### Selected native layout artifact

The Monitor native-layout producer emits one canonical `native-layout.json` from
the resolved selection, not from caller-provided unit or configuration hashes.
For the Monitor server it contains exactly `rz.target`, `rz-admin.service`, and
`rz-monitor.service`; the target wants only those two services. The services
read `config/rz-admin.env` and `config/rz-monitor.env` respectively. The
node-agent artifact contains only `rz-monitor-agent.service` and
`config/rz-monitor-agent.env`.  Each configuration manifest names one consumer
and only that consumer's reviewed variable keys; it contains no values or
secrets. The generator is the selected-native source while the existing full
deployment templates and installer remain outside this closure.

The artifact binds `compositionId`, `artifactClass`, preset, resolver units and
`configOwners`.  Its strict reader accepts one stable, non-link `native-layout.json`
with canonical bytes and rejects unknown, missing, duplicate, stale-composition,
or server/Agent-crossed members before packaging.
Manifest callers provide `nativeRoot`; direct `nativeLayoutDigest` input is
rejected. The manifest derives the digest from the stable canonical artifact and
binds it into `buildId` for both Server and Agent.
The generated source also exposes the exact unit bytes for a later writer. It
uses a distinct non-root User/Group for Admin, Controller and Agent. It
does not include recovery conditions, so the old `rz-recovery.service` cannot
be carried into this selected topology.

### Fresh-root admission executor

`rz verify` consumes `archive.tar`, `release-manifest.json`, and
`signature-envelope.json`, with a separately supplied trusted PEM public key and
key ID. `rz apply --destination <absent-root>` performs the same verification;
`--dry-run` writes nothing. The executor admits only a production Server or
node-Agent tuple, strict regular ustar members, exact member hashes and a
detached Ed25519 envelope before creating the root. It publishes only the
immutable selected payload, retained manifest/envelope, `current` link,
root-private trust and a publication marker. `runnable` remains false: this is
not a completed fresh installation and it creates no runtime config, journal,
systemd state, database, update, rollback, restore or recovery service.
`keyId` is a single safe identifier (`[A-Za-z0-9][A-Za-z0-9._-]{0,63}`), while
`buildId` and `compositionId` are lowercase SHA-256 identities. Apply and dry
run require the release target to match the executing Linux architecture before
opening a destination for publication.

agentProtocolContractId is required for node-agent and server selections with
Monitor, and forbidden for other server selections. Derive it from the canonical
versioned Agent report/response wire descriptor, including required fields,
encodings, sequencing/fencing and acceptance semantics; export it from the
actual selected protocol code and cover it with conformance fixtures. Include
it in the resolved plan and signed actual manifest. Initial pairing requires
exact equality, not an undefined version range. Each newly built pair requires
current fixture certification and a fresh installation; no historical protocol
fallback is added.

Before node-agent activation, run `rz prepare-monitor-agent-access` once. It
prepares only `/opt/rz` directories and the retained manifest for the fixed
`rz-monitor-agent` identity; the signed Agent binary remains `0755`. Then
`rz pin-monitor-controller` receives the target Controller's signed production
manifest/envelope, an independent trusted PEM, key ID and explicit endpoint. It
writes only the fixed `/opt/rz/controller-profile.json` after verifying the Controller tuple
and signature, then requires the Agent
manifest protocol to match exactly. It writes one canonical root-owned profile
with endpoint, both build IDs and manifest digests, protocol ID, key ID, and
trusted-key fingerprint. Its mode is `0640`, root-owned and assigned to the
explicit Agent group; tokens and payload-bundled keys are excluded. Repeating
the same tuple is idempotent, while a different tuple never overwrites it. Every Agent
deployed production startup checks that profile before logger or network initialization. Local
development keeps the direct Agent command available without installed `/opt/rz` state. Pair
certification exercises both binaries with mismatched protocol fixtures.
This offline gate certifies the declared pair, not that an arbitrary endpoint
actually serves the supplied profile; endpoint provisioning remains the local
operator's responsibility. No enrollment service or remote identity database.

The Agent unit uses system-service readiness as a report-delivery fact, not a process-start fact.
After profile validation and startup, it emits a single `READY=1` notification only when the paired
Controller returns `accepted` or `duplicate` for a report. A duplicate is a confirmed prior delivery.
`stale`, `401`, other non-success responses, malformed response bodies, TLS failures, and network
failures never emit readiness; later scheduled reports may do so. This signal stores no report data
and does not replace Controller fencing or liveness.

Activation publishes the verified config and unit, reloads systemd, enables the unit, then queues
`systemctl --no-block start rz-monitor-agent.service`. Its marker means this exact tuple was
published and start was accepted for queueing; it does not mean the service became ready or that the
Controller was contacted. The activation lock is released after that bounded request. A real PID1
must separately observe the later report-delivery readiness transition.

### Agent activation

`rz activate-monitor-agent --config <root-only-file>` is the production
activation input. The source is a regular, non-symlink root-owned file without
group/world write access, so its token is never an argv value. Its only keys are
`RUSTZEN_ENV`, `RUSTZEN_MONITOR_NODE_ID`, `RUSTZEN_MONITOR_CONTROLLER_URL` and
`RUSTZEN_MONITOR_AGENT_TOKEN`. Production requires `RUSTZEN_ENV=production`, a
valid node ID, a canonical HTTPS URL byte-equal to the paired profile endpoint,
and a nonempty non-placeholder token. Activation atomically publishes
`/opt/rz/config/rz-monitor-agent.env` as root:rz-monitor-agent `0640`, rejects
different existing config bytes, and never exposes the token in output, profile
or logs. It installs only the selected `rz-monitor-agent.service`, never an old
deploy unit or a member of `rz.target`.

Activation holds one root-owned `/opt/rz/state/.monitor-agent-activation.lock` through unit
publication and systemctl. It writes a canonical root-only activated marker only
after all three systemctl actions succeed. Same tuples wait and return from that
marker without repeating systemctl; different tuples conflict. A failed action
writes no marker, so retry converges from identical published bytes.

releaseClass is required: production or test. Production tools reject test
artifacts/current-full-regression regardless of key trust. The full preset must
match its exact declared feature closure; a preset label cannot override omitted
features. Test identity/trust/root restrictions are defined in validation.md.

### P8b Monitor source-build certificate

`source-build-manifest` is the canonical JSON certificate contract for one
production `monitor` server tuple. It binds the resolver's exact preset, target,
artifact class, composition ID, capabilities and services to three and only three
certified layers: `source` (source identity and tree digest), `build` (toolchain,
build ID and exact `rz-admin`/`rz-monitor` file digests), and `artifact` (manifest,
archive and detached-envelope digests). The manifest is parsed against the same
selection before its digest is admitted.

The certificate has no extensible status field. `runtime`, `browser`, `load` and
`releaseReady` are required literal `false`; no `current` pointer, installation,
deployment, Agent witness or later acceptance result can be included. A changed
field, additional layer, changed capability closure or production label is a new
certificate input and must be rebuilt. Verification requires both the structural
parser and the authoritative selection, verified release manifest, captured
source identity/toolchain and archive/envelope digests; parsing valid hash shapes
alone is never acceptance. The source-build certificate is evidence for its
three named layers only.

Contract extraction accepts an explicit release binary directory. It never invokes
Cargo itself: the caller supplies `rz-admin` and `rz-monitor`. This extraction
step does not prove that they share a build batch. The later certification
orchestrator must capture one exact release inventory before and after extraction.
Agent protocol extraction remains a separate witness operation and cannot make
the Agent binary a member of the Monitor server archive.

### P8b Monitor container export

The Monitor Docker export accepts only `DISTRIBUTION=monitor`,
`TARGET_TRIPLE=x86_64-unknown-linux-musl` and a nonempty `SOURCE_IDENTITY`
build input. The caller must select Linux/amd64 and the producer verifies that
runtime platform inside the build stage. Before the terminal
export it must have these payload members:

| Root | Required members |
| --- | --- |
| `release/server/bin` | `rz-admin`, `rz-monitor` |
| `witness/bin` | `rz-monitor-agent` |
| `release/web` | `inventory.json`, nonempty selected `dist/` |
| `release/contracts` | `api/api.json`, `schema/schema.json`, `config/config.json`, `protocol/protocol.json`, `native/native-layout.json` |

No other payload root, server binary or witness binary is valid. The canonical
`release/output-manifest.json` records every payload file's relative path,
mode, size and SHA-256. `release/container-provenance.json` records the exact
selection input and canonical digest, source-identity input, `rustc -Vv`, target
triple, `linux/amd64` build platform, exact command arrays and the output-manifest
digest. Both documents reject links and unsupported file modes. They are not a
`source-build-manifest`, cannot set a certification boolean and cannot be used to
publish, install, sign or update a current pointer.

The installer rejects duplicate paths, absolute paths, parent traversal,
unexpected links/files, oversized members, incorrect modes, unknown capabilities,
missing units/assets, and hash/signature mismatches before switching anything.
No unknown field silently changes executable behavior. Generated units and
commands use fixed argument arrays, not manifest-controlled shell fragments.

### Monitor server activation

`rz activate-monitor-server --config <root-only-file>` accepts only an absolute,
regular, non-symlink root-owned source under root-owned non-writable parents.
It accepts the exact union of selected Admin and Monitor config fields and
splits them into `rz-admin.env` and `rz-monitor.env`; each is root-owned `0640`
and grouped only to its service identity. The source must be production, contain
the selected database paths, JWT, IPC and Agent secrets, and include a
one-time owner credential input consumed only by the Admin bootstrap command.
Installer-only `RUSTZEN_ADMIN_RUNTIME_ROOT` and
`RUSTZEN_MONITOR_RUNTIME_ROOT` must exactly be `/var/lib/rustzen-admin` and
`/var/lib/rustzen-monitor`; direct `RUSTZEN_RUNTIME_ROOT` input is rejected.
The selected SQLite paths are fixed to `admin.db` and `monitor.db` under those
roots. The owner secret is streamed anonymously to the Admin bootstrap command,
never written to disk or the activation marker. Fresh schema contains only a
disabled owner with a deliberately non-verifiable hash; bootstrap enables it.

The two fresh SQLite files are initialized under deterministic service-owned
staging names. A root-only journal binds their hashes and the selected activation
tuple before either no-replace rename. A same-tuple retry resumes an interrupted
first or second rename; existing databases without that journal are rejected.
The journal remains until the ready marker is durable so failures after database
publication can retry without recreating or replacing either database.
Completion and exact retry execute each selected binary's database validator.
Each staging database receives exactly one `rustzen_installation_identity` row
before publication. The row binds build, composition, owner-specific schema
fingerprint and data contract ID from the retained signed manifest. Runtime and
exact retry require that singleton, the exact embedded SQLx migration ledger,
and a canonical `sqlite_master` inventory equal to a fresh in-memory application
of that binary's embedded migration. Selected services validate these values
before starting application work and never run migrations against an existing DB.

It admits only a retained signed Monitor server manifest and matching current
Admin/Monitor binaries and three selected native units. It validates both
rendered configs with the selected binaries before any destination write,
rejects any known Reports/Insights unit, enabled state, config or runtime root,
publishes only `rz-admin.service`, `rz-monitor.service` and `rz.target`, invokes
fixed `daemon-reload`, `start rz.target`, verifies both systemd MainPID executable
identities and health endpoints, then
`enable rz.target` and writes a root-only marker. Same tuples require active
services, exact health identity and Admin password verification; different or
unsafe destinations conflict. A failed start, readiness, enable or marker write
stops and disables the target and leaves no marker.
If marker publication has already renamed the exact bytes, a directory-sync
failure is reconciled by syncing and re-reading the marker. Journal cleanup
failure returns an error while leaving the ready services and marker intact;
the next exact retry verifies them and completes cleanup.

Reuse the existing Ed25519 bundle signing primitive, with a versioned envelope
that signs a fixed canonical payload containing format version, component,
release version, target, archive SHA-256 and exact manifest-byte SHA-256. The
manifest is an archive member; its file inventory excludes only itself and the
external signature envelope to avoid recursion. Retain its exact bytes and the
verified envelope for installed-manifest verification. Verify the trusted key
and archive digest before inspecting/extracting members; then check the signed
manifest digest and every member. The archive's supplied public key is never
a trust root. Initial trusted keys come from independently provisioned local
installer configuration; rotation is an explicit local trust update, not an
unsigned package field. No new signature algorithm or remote key service.

Copy input archive/envelope into root-private staging once, then hash, parse and
extract the same opened immutable file objects. Never reopen an untrusted path
after verification. Extract relative to a trusted directory descriptor with
beneath/no-symlink resolution and exclusive file creation; reject archive links,
devices and path-component substitutions. Apply final modes only after content
checks, fsync files/directories, then publish. A signed package is still untrusted
input until these path and size checks pass. Retained releases are root-only
writable. Same-build interrupted-install recovery revalidates its exact files;
it never chooses a historical release or restores old data.

`GET /__web-binding` is an access-owned anonymous bootstrap response containing
only `{ "bindingVersion": 1, "webDigest": "<sha256>" }`. It carries no module,
user, version, path or service-health inventory. Set no-store and validate the
response's content type/schema/size before loading any login or feature code.
The server value comes from its packaged Web digest, not a requested query value.

`GET /api/installation` is an authenticated, read-only build/runtime inventory:
release version, composition/build IDs, exact webDigest, installed feature IDs, selected service state,
and user-visible capabilities. It exposes no secrets, filesystem paths or
unselected module advertisements. Web compares its HTML-stamped webDigest through
the anonymous binding response before enabling login/feature code; authenticated
inventory returns the same digest. A differing buildId alone is not a mismatch
when Web bytes and the verified client/API contract still match. Public health
exposes only liveness/readiness, not the inventory.

## Internal producer API

### Directional signatures and key lifecycle

Use separate typed codecs/verifiers and headers: `x-rustzen-delegation-*` for
Admin-to-module user decisions and `x-rustzen-notify-*` for producer events.
Their HMAC domains are `rz-user-delegation-v1` and
`rz-notification-producer-v1`. Keys are generated randomly per installation and
per module/producer, at least 256 bits, and never reused between these domains,
JWT signing or Agent collection. Private runtime configuration contains keys;
packages, HTML, logs, user JWTs and database business payloads do not.

Canonical encoding is a fixed-order, length-prefixed UTF-8 field sequence with
versioned test vectors, not ad-hoc concatenation. Reject duplicate signature
headers, unsupported algorithms/versions and ambiguous request targets. Sign
the request target exactly as forwarded after one defined URI normalization;
do not independently reorder/decode query parameters at the receiver.

| Direction | Covered fields |
| --- | --- |
| User delegation | Domain/version, keyId, target module and current module instance, operation ID, method, path/query, content type, body SHA-256, user ID, sid, current authz epoch, created/expiry, nonce |
| Notification event | Domain/version, keyId, producer ID, Admin audience, method, fixed path with no query, content type, body SHA-256, created/expiry, nonce |

The gateway strips all old/new internal signature headers and user Authorization
before adding its own delegated envelope. Agent-token forwarding is allowed only
on the selected Agent-report operation and never substitutes for a user decision.
Hash exact bounded request bytes before forwarding; each route declares its body
limit. Requests needing unsupported streaming request signatures are rejected,
not forwarded unsigned. Response streaming remains separate.

Each module validates target/operation/route equality, signature, current process
instance, expiry and a single-use nonce before admission. A runtime instance ID
is established through a fresh challenge authenticated with that module's IPC
key, using a separate `rz-user-delegation-instance-v1` message tag that covers
the Admin challenge, keyId, target module, instance ID and short expiry. It is
not a user-delegation envelope and cannot authorize an operation. An old
handshake cannot replace current instance metadata. Cache nonces
through permit expiry; reject at capacity rather than evict live entries. A new
process instance rejects pre-restart envelopes without durable nonce tables.
No route may fall back to ordinary JWT acceptance on the module listener.

Admin is the authority for user policy and issues a delegation only after current
session/user/policy checks. A user permit expires within 5 seconds of issuance
and no later than JWT expiry. Modules consume that signed decision; they do not
maintain another user/role database. Revocation blocks new decisions immediately
after its transaction commits. A decision issued before revocation is an
in-flight authorization and may be admitted within its short validity; an
already admitted operation may finish. This does not claim retroactive rollback
of work or automatic cancellation of queued Reports runs. Replays, expired
permits, cross-instance envelopes and new requests using revoked sessions fail.

Every direction has a configured current keyId and an explicitly bounded
previous-key overlap. Install the new verification key before switching signers;
retire the old key after the maximum transport lifetime plus tolerance (at most
120 seconds for events; at most 10 seconds for user permits). Revocation may
remove a compromised key immediately and tolerate transient failed requests.
Retries sign immutable business bytes with the current key and a new nonce;
outbox never stores an old transport signature. Unknown/retired keys fail closed.

### Event ingress

`POST /internal/v1/notification-events` is registered only with notifications,
on a dedicated loopback listener owned by the entry host. The public gateway
never forwards `/internal/**`. Remote access and container-network adaptations
are outside the first native deployment contract.

Headers carry the signed producer/key identity, created/expiry, nonce and signature
defined above. Limit body size before parsing or hashing
unbounded input. Timestamp tolerance is 60 seconds; replay-nonce retention is
120 seconds and bounded by ingress rate limits; a saturated cache rejects new
admissions. Event transport expiry is at most 60 seconds after signed creation,
checked independently of the business event's seven-day horizon; no expiry
leeway is added. A restart may lose the nonce
cache; durable event-ID deduplication still prevents duplicate business writes.

Initial authenticated admission limits are 100 requests/second per producer
(burst 200) and 150/second aggregate (burst 400). Nonce storage allows 20,000
entries per producer and 30,000 aggregate, with 120-second retention. This covers
rate times retention plus burst/in-flight margin for the 100/second aggregate
acceptance fixture. Validate that inequality when config changes; incompatible
limits fail configuration validation. Rate and size limits precede nonce allocation;
only authenticated envelopes consume producer nonce quota. Saturation rejects
new work without evicting unexpired nonces. Retries consume transport capacity
even though duplicate business events consume no new inbox storage.

Initial topics:

| Topic | Producer | Business transition | Required recipient access |
| --- | --- | --- | --- |
| `monitor.incident.opened` | Monitor | First transition to active | `monitor:incident:view` |
| `monitor.incident.resolved` | Monitor | Active to resolved, including policy-driven resolution | `monitor:incident:view` |
| `reports.run.completed` | Reports | Running to succeeded | Initiator still allowed to view the run |
| `reports.run.failed` | Reports | Nonterminal to failed | Initiator still allowed to view the run |
| `reports.run.cancelled` | Reports | First legal nonterminal to cancelled transition | Initiator still allowed to view the run |

Final permission names for Reports come from its existing route registration;
the implementation must not invent a second capability declaration here.
No notifications are emitted for duplicate/stale Agent reports or an unchanged
active incident. Recovery caused by policy changes uses the same transition
contract as recovery caused by new samples.

```json
{
  "schemaVersion": 1,
  "eventId": "<UUID generated once in the business transaction>",
  "producer": "monitor",
  "topic": "monitor.incident.opened",
  "occurredAt": "2026-09-03T06:00:00Z",
  "expiresAt": "2026-09-10T06:00:00Z",
  "subject": {"kind": "monitor-incident", "id": "<incident UUID>", "revision": 1},
  "audience": {"policy": "monitor-incident-readers"},
  "content": {"title": "Resource alert", "summary": "An incident requires review"}
}
```

All sizes and identifiers are validated. The authenticated producer must equal
the body producer and own the topic/subject. An event contains a plain-text
summary and typed business reference, not HTML, credentials, telemetry dumps,
an arbitrary URL, permission grants or arbitrary user lists. The server maps
subject kinds to a fixed frontend action. For Reports, a verified initiator
identifier is allowed only under its producer policy and authorized run model.

Reports must add an immutable `initiator_user_id` to the run model and fresh
schema. A manual create handler captures it from verified delegated user context,
never from request JSON. Persist it in the same transaction as run creation;
later edits, cancellation by another user and process restart do not change it.
Scheduled runs initially have a null initiator and emit no personal terminal
notification. Scheduling continues normally; notifying a schedule owner would
require a separately specified ownership lifecycle, not an inferred recipient.
Admin resolves the supplied immutable ID against its current user/access data;
disabled, deleted or unauthorized users receive no message. Reports is the
trusted owner of the run-to-initiator binding, not a source of permission grants.

Generate terminal events only when the database conditional state update wins.
Cover queued-to-cancelled, cancelling-to-cancelled, and recovery transitions that
actually enter cancelled. Repeated cancellation, recovery of an already terminal
run and completion/cancellation races cannot create a second terminal event.
The transition and outbox insert share one transaction. A scheduled run with no
recipient policy does not allocate an event only to discard it downstream.

| HTTP | Result | Producer behavior |
| --- | --- | --- |
| 201 | `stored`: receipt and recipient projection committed | Record acceptance and remove outbox row |
| 200 | `duplicate`: same producer/event ID and payload digest | Record acceptance and remove row; do not recreate recipients |
| 200 | `no-recipients`: durable receipt, no eligible user at ingest | Remove row; do not replay for future role grants |
| 409 | Same ID, different payload | Quarantine event; report producer bug |
| 410 | Event expired before first acceptance | Remove row and count expiry loss atomically |
| 400 / 413 / 422 | Invalid protocol, size or semantic input | Quarantine and surface failure |
| 401 / 403 | Bad producer identity or topic authorization | Stop rapid retries; surface configuration failure |
| 429 / 503 / network timeout | No confirmed acceptance, including `notification-capacity` | Retry same event with new transport nonce/backoff within original expiry |

A valid duplicate receipt may return 200 after its original event expiry.
Transport timeout is ambiguous: resend only the same event ID and bytes.
These status codes distinguish durable acceptance from transient SSE delivery.

After transport/schema validation, Admin checks an existing receipt and digest
before business expiry; only first acceptance is prohibited after expiresAt.
At expiry, an event never claimed for sending can be counted expired locally.
Any attempted event without a definitive result enters persisted reconciling
until expiresAt plus 60 seconds, using only the same ID/bytes and fresh transport
signatures. A duplicate is acceptance; 410 confirms expiry. A known accepted
response can still finish a live claim. At reconciliation deadline, once no
live claim exists, remove an unresolved row and increment unconfirmed_count,
not confirmed loss. Crash recovery keeps the original deadline; no endless
extension or new-event acceptance after expiry is allowed. A business horizon
bounds first acceptance, while this short additional window resolves ambiguity.

## Producer persistence and bounded reliability

Optional outbox schema belongs to each producer application:

```text
notification_outbox
  event_id primary key
  topic, subject_kind, subject_id, subject_revision
  payload_json, payload_sha256, occurred_at, expires_at
  state (pending | reconciling | quarantined)
  attempts, next_attempt_at, lease_until, lease_token, reconcile_until, last_error_code

notification_delivery_status
  id = 1, omitted_count, expired_count, unconfirmed_count, quarantined_count,
  first_gap_at, last_gap_at, last_success_at
```

One relay runs per producer process. It claims a bounded batch with a lease,
retries after crashed leases, and preserves event order per subject for pending
events. Consumers do not assume cross-subject or cross-service total order.
Subject revisions prevent delayed older events from overwriting newer subject
summaries. Inbox history retains distinct open and resolved transitions.

Claim in one write transaction with a new random lease_token. For each subject,
only its smallest pending/reconciling revision is eligible; backoff or a live
lease blocks later revisions. Cross-subject concurrency remains four. Every
response/state/counter mutation matches event_id, state and current lease_token;
a stale worker changes zero rows. Acceptance, confirmed expiry and quarantine
release that subject. Use a 5-second request timeout and 30-second claim lease;
recover crashed claims only after expiry. Cleanup never deletes a live claim.

Initial proposed defaults: 7-day pending horizon, 100,000 pending events or
64 MiB charged-data budget per producer, 16 KiB maximum body, batches of 100,
4 in-flight requests, exponential retry from 1 second to 60 seconds with jitter.
Reconciling rows share the pending count/byte budget. These are tunable test inputs,
not validated capacity claims. A hard pending
capacity limit must not reject an otherwise valid Agent report: after deleting
expired rows, a full outbox commits the incident plus a constant-size durable
gap counter and omits the new notification. The module exposes this delivery
gap to authorized operators and emits a rate-limited diagnostic.

Acceptance and expiry are terminal processing outcomes: remove those outbox
rows in the same local transaction that updates the constant-size status record.
Do not retain accepted payloads as another event history. Quarantined rows have
a separate 24-hour, 1,000-row and 4 MiB charged-byte limit; remove oldest rows
when necessary and preserve aggregate failure/eviction counters. Pending and
quarantine budgets include bounded metadata charges, not payload bytes alone;
the latter cannot borrow unused pending capacity. Startup recomputes accounting
before relaying, and every state change reserves/releases budget transactionally.

`occurredAt` may be at most 60 seconds in the future; `expiresAt` must be later
than occurrence and no later than occurrence plus 7 days. Retry never extends
these timestamps. Subject revisions are persisted monotonic values advanced
only on committed lifecycle transitions; they do not come from relay order or
wall-clock timestamps.

This is a deliberate bounded-reliability policy. During an outage shorter than
the horizon and within capacity, committed events are retried without duplicate
inbox messages. Outside that window, inbox completeness is not promised;
Monitoring incident history remains authoritative. Disk-full failure of the
business database can still fail the business transaction and must be reported
as a storage failure; notifications cannot eliminate that shared-resource risk.

## Admin persistence

P5a implements the selected fresh-schema owner and the five authenticated
personal-inbox read/read-state endpoints below. Its executable evidence covers
current-user, current-grant and enabled-module authorization in one Admin
database snapshot, authenticated-encrypted sequence boundaries and concurrent
idempotent reads. The wire DTO omits the internal `inbox_seq`; only opaque
cursor/snapshot tokens carry its encrypted boundary. `monitor-notify` applies
and verifies the base and notification migration ledgers on migrate, bind,
validate and reopen. Its formal producer builds the selected Admin feature and
exports base Admin and notification routes as distinct code-derived owners.
P5b adds a `notifications` configuration owner for its admission and storage
thresholds. The internal admission service, bounded retention and durable
accounting are implemented and locally verified. Producer ingress/relay and
SSE remain P6/P7 and are not claimed by this slice.

All notification tables are excluded when the feature is absent:

```text
notification_receipts
  producer, event_id                        primary key
  payload_sha256, accepted_at, expires_at, retain_until, result

notifications
  inbox_seq integer primary key autoincrement
  id unique, producer, event_id              unique
  topic, subject_kind, subject_id, subject_revision
  occurred_at, accepted_at, title, summary, required_capability

notification_recipients
  notification_id, user_id                  primary key
  read_at nullable, created_at

notification_user_state
  user_id primary key, revision
```

An ingest transaction inserts the receipt, message and currently eligible
recipient rows and increments affected revisions. Concurrency uses database
uniqueness as the final idempotency authority. A failed fanout transaction
rolls back the receipt as well. Duplicate requests do not re-resolve recipients.

For Monitor, recipients are enabled installation users currently granted the
incident read capability. For manual Reports runs, the eligible persisted
initiator is the recipient; scheduled runs have no initial notification policy.
Granting access later does not subscribe a user to historical messages. Revoking
access immediately removes affected records from list/count/detail results;
retained rows are not an authorization grant. Module disablement also removes
its messages from the user's accessible inbox until re-enabled. The recipient
and permission decisions use Admin-owned identity data only.

Initial message retention: 30 days by acceptance time, including unread messages.
The UI states this retention limit. Receipt retention lasts through the maximum
event retry horizon plus clock tolerance and at least the message-retention
period; cleanup never permits an expired original event to be accepted again.
Cap recipient expansion at 1,000 eligible users per event initially; an
installation requiring more must explicitly raise and validate this limit.
If expansion exceeds the configured cap, reject atomically with a diagnosable
422 `audience-too-large` error; never deliver silently to a partial audience.

Initial Admin admission budgets are 100,000 messages, 1,000,000 recipient rows,
1,000,000 receipts and 512 MiB total charged notification data. Account for
message bytes with a minimum 1 KiB charge, receipts with at least 512 bytes,
recipient rows with at least 256 bytes and all user-state metadata. Fixed-sized
IDs and bounded content/error fields prevent unaccounted variable data. These
are provisional budgets to validate against real indexes and storage overhead.
The access-user inventory bounds user-state rows. Once created, a user's state
keeps its revision monotonically increasing even when retention removes the
last recipient; only deletion of that user removes the state through the
existing identity foreign-key cascade.

The fresh notification schema maintains these four values in one singleton
accounting row through insert/delete triggers. Admission never performs an
unbounded `COUNT` or `SUM`; service construction validates the singleton once
against the four business tables and fails closed on drift. Charges use UTF-8
byte lengths with the stated minimums: message charge includes every persisted
message text field, receipt charge includes its persisted text fields,
recipient charge includes its notification identifier, and user state has a
128-byte minimum. Schema byte-length checks bound every variable field.

Before admission, a bounded cleanup removes expired recipient history first,
then recipient-free expired messages and expired receipts whose `retain_until`
has passed. Each pass is limited to 500 rows per phase,
4 MiB released charge and 50 ms between phases; a large expired fanout is
drained across independently committed passes rather than deleted by an
unbounded cascade. Admission runs at most eight passes per attempt, then opens
a new immediate transaction, rechecks the receipt and atomically admits or
refuses the event. A later attempt continues from the committed cleanup state.
Affected user revisions increment at most once per cleanup transaction before
recipient deletion. State and its durable charge remain after the last
recipient so published, read and reconnect revisions cannot fall back to zero;
identity deletion releases the state through cascade. Set `retain_until` to the later
of acceptance plus 30 days and original expiry plus clock tolerance. Never evict
an unexpired receipt to make space: otherwise retries could recreate messages.
Check an existing receipt before new-event capacity checks so duplicates still
return 200 under saturation. New events, including no-recipient receipts, need
atomic budget reservation; any exhausted budget returns 503
`notification-capacity` plus Retry-After, with no partial inserts. Retained
history is not silently evicted early. Producers retry within their original
horizon and surface expiry/gaps if capacity never recovers.

All five inbox read operations apply the injected-clock acceptance cutoff, so
expired rows are silent even before physical reclamation. A notification-selected
Admin validates accounting and runs one bounded cleanup before serving, then
runs one bounded pass hourly in the same process. Its lifecycle guard cancels
and joins this maintenance task on controlled shutdown. A composition without
notifications has no such startup or periodic task.

Logical notification budgets bound growth attributable to this feature; they
are not a hard quota on the shared database, WAL or filesystem. Measure actual
page/index overhead, checkpoint behavior and reader lifetimes. Reject new
notification admission unless, while holding the final SQLite write lock after
audience and charge calculation, available space is at least the configurable
reserve (initially 128 MiB) plus projected charge,
stop notification writers on sustained checkpoint pressure, and surface storage
pressure to operators. Retention deletes reuse pages and need not shrink the
file; never run full VACUUM on the request path. Other feature writes and total
disk exhaustion remain explicit operational risks, not eliminated guarantees.

The initial selected settings are `RUSTZEN_NOTIFICATION_MESSAGE_LIMIT=100000`,
`RUSTZEN_NOTIFICATION_RECIPIENT_LIMIT=1000000`,
`RUSTZEN_NOTIFICATION_RECEIPT_LIMIT=1000000`,
`RUSTZEN_NOTIFICATION_CHARGED_BYTES_LIMIT=536870912`,
`RUSTZEN_NOTIFICATION_FREE_SPACE_RESERVE_BYTES=134217728`,
`RUSTZEN_NOTIFICATION_WAL_PRESSURE_FRAMES=1024`, and
`RUSTZEN_NOTIFICATION_WAL_PRESSURE_OBSERVATIONS=3`. The notifications-only
descriptor owns them. Capacity refusal is a typed internal result carrying
`notification-capacity`, a retry interval and a diagnostic reason; it is logged
and exposed to the Admin operator status seam before P6 adds producer transport.

Index recipient/user/read state, message creation order and receipt expiry.
Store read state once and derive unread counts with current authorization;
do not maintain an independently mutable unread counter. Coalesce live
invalidations to reduce fanout for bulk read operations.

## User inbox API

All endpoints infer the user from authentication. There is no client-controlled
`userId` used to choose the inbox.

| Method / path | Request | Response / invariant |
| --- | --- | --- |
| GET `/api/notifications` | Authenticated-encrypted opaque cursor, integer limit 1..100, optional unread filter | Accessible messages in stable newest-first order, next cursor, revision; internal sequence is omitted |
| GET `/api/notifications/unread-count` | None | Authorized unread count + revision |
| GET `/api/notifications/{id}` | None | Accessible message or 404 |
| PUT `/api/notifications/{id}/read` | Empty | Idempotent read_at + new/current revision; inaccessible is 404 |
| POST `/api/notifications/read-all` | Server-issued snapshot boundary | Mark only accessible messages at or below boundary; new arrivals remain unread |
| GET `/api/notifications/stream` | Bearer header; optional `Last-Event-ID` header only; no URL credential | SSE advisory invalidations for this session/user |

List cursors bind sort/filter/snapshot boundary and user identity using an
opaque authenticated-encrypted token; neither identity nor sequence state is
visible in the external bytes. Invalid query syntax, bounds and cursors return
the JSON error envelope with 400. Read-all's boundary is issued
by a preceding inbox query, bound to the user, and cannot include future rows.
Capability changes are re-evaluated when writing read state. Paging and bulk
read may not leak inaccessible message counts or identifiers.

Allocate non-reused inbox_seq in the ingest transaction. List order is descending
inbox_seq (accepted_at remains display metadata); its encrypted snapshot token binds
user, filter and maximum observed sequence, never wall-clock time or UUID order.
Read-all changes only currently authorized unread recipients within that boundary
and filter. Single read uses WHERE read_at IS NULL and preserves the first read
time. Increment the user's revision once per transaction only when visible inbox
state actually changes. Duplicate reads return the current revision without
incrementing it. New same-timestamp arrivals have greater sequences and remain
unread. Retention cannot reset/reuse the sequence counter.

## SSE contract

```text
event: inbox.changed
data: {"schemaVersion":1,"revision":42}

event: reconcile.required
data: {"schemaVersion":1,"reason":"connected"}

```

No notification body or cross-user IDs are sent. `revision` is a durable
per-user invalidation hint, not a universal event position. SSE event IDs and
`Last-Event-ID` do not imply server replay; initial/reconnected clients always
query the current authorized snapshot. Heartbeats are comments, not inbox events.

Client order: establish subscription, buffer invalidation revisions, fetch the
snapshot, then reconcile any newer buffered revision. Repeat on reconnect or
auth-context change. While visible, run a 60-second bounded reconciliation as
a safety net for a lost final invalidation. A hidden tab reconciles when visible.
Read responses update cache and other open connections receive invalidation.

Authenticate, atomically reserve both connection quotas and register the bounded
hub queue before returning 200/first reconcile.required frame. List and unread
count each read authorized data, sequence boundary and user revision from one
DB snapshot; never pair old data with a newer separately-read revision. The
client begins its snapshot after the first registered-stream signal. A stream
close/overflow or auth-generation change invalidates an in-flight snapshot;
discard it and repeat the handshake. No persistent DB-to-hub atomicity is
claimed: a crash after commit is covered by reconnect and periodic
reconciliation. Admission, read and retention cleanup publish their affected
user revisions only after commit; rollback and no-change cleanup publish
nothing.

Initial production defaults: 15-second heartbeat, queue capacity 16, 1,000
total connections, 4 per user across all sessions, 5-second access
revalidation, 45-second server-body poll inactivity and a 5-minute absolute
connection age. Clients reconnect and reconcile after the absolute age. The
server observes body polling rather than a transport write acknowledgement, so
this bound does not claim end-to-end socket acknowledgement. Retry is 1..30
seconds with jitter. Abort on logout, terminal authentication failure or tab
teardown; require a fresh login on credential expiry; no refresh-token service
is added.
When a queue is full, coalesce inbox invalidations to the newest revision or
disconnect the slow client. Never block business persistence on a slow socket.

One cancellation handle covers an independent JWT-expiry deadline, bounded authorization recheck, disconnect
and shutdown. Cancellation removes the hub entry, releases quotas exactly once
and stops admitting application frames independently of socket backpressure.
Race frame production/write progress against cancellation and a bounded stall
timeout no later than authentication expiry; the HTTP body/transport adapter
must be tested under a socket that stops reading. An unpolled body must not keep
the authorization task or queue registration alive. The five-second recheck
budget includes the DB check; timeout fails closed. Bytes already handed to the
transport can arrive later: expiry forbids new server-side frame admission, not
retroactive recall of network bytes.

Fetch accepts only 200 with text/event-stream, same-origin URL and no redirects.
Use streaming UTF-8 decoding and a tested SSE parser; cap an unterminated line
at 8 KiB and accumulated event at 16 KiB. Oversize or malformed input closes the
stream with a visible diagnostic. A 45-second heartbeat watchdog reconnects an
otherwise live visible page; it is reset on visibility restoration.

401 clears identity and requests login; 403 stops live subscription until a new
authorized context; 204 ends it intentionally. Per-user 429 and global 503 have
stable error codes/Retry-After and enter existing visible-page reconciliation,
with at most one stream retry per max(60 seconds, Retry-After) plus jitter.
Network/clean unexpected EOF use bounded 1..30-second retry; HTML, redirects and
protocol failures stop automatic fast retry and allow an explicit retry. Never
parse an error/login page as SSE. All paths release reserved quotas on failure.

The stream returns `200 text/event-stream` only after authority validation and
atomic quota/queue registration. Missing, invalid or expired identity is `401`;
an enabled-producer/current-grant mismatch is `403`; a draining hub is `204`;
the fifth connection for one user is `429`; and the global limit or an authority
database failure is `503`. `429` and `503` use the JSON error envelope, stable
error codes and `Retry-After: 60`; all responses disable caching, and successful
streams also disable intermediary buffering. Established streams end with EOF
after expiry, authority change, database failure, shutdown or client drop.
Request tracing and operation logging record only the URI path; rejected query
credentials are never persisted or emitted to traces.

The authenticated shell owns the stream, so ordinary feature navigation does
not multiply subscriptions. Abort on shell teardown/logout/pagehide; pageshow
and visibility restoration revalidate access and start a fresh subscription/
snapshot if needed. Hidden pages may suspend work; no timer punctuality is
assumed while frozen. Poll fallback is single-flight, uses the same auth-generation
guards, and never continues business polling after a terminal identity failure.

Disable intermediary buffering and compression that delays SSE frames; use a
streaming route with an explicit idle/heartbeat policy, not the current generic
10-second module HTTP timeout. The endpoint is owned by Admin directly and
does not proxy one SSE stream per module. Reverse-proxy and real-browser tests
are required; response type correctness alone is insufficient.

## Access and revocation

Keep the existing user-ID identity and HS256 validation, adding installation
issuer, entry audience, random `sid` and `user_auth_epoch` claims alongside
issued/expiry times. Reject missing legacy claims in this fresh-install target;
do not infer a valid session from a signed user ID alone. `exp` is a hard limit
with no leeway. Tokens contain no authoritative roles or permissions.

Access owns `access_sessions(sid, user_id, auth_epoch_at_issue, expires_at,
revoked_at)`; users have a monotonic `auth_epoch`, and one access-policy metadata
row has a monotonic `authz_epoch`. Login creates a session before issuing its
token. Single-session logout marks that sid revoked; password reset, disablement
and explicit revoke-all advance the user's auth epoch. Every role/grant/status
mutation increments the global authz epoch in the same transaction. Authorization
caches are keyed by user ID and the current authoritative authz epoch. A role
change refreshes policy without requiring every valid identity session to log in
again; authz epoch is not a permanently frozen permission claim in the JWT.

Every protected request checks signature/algorithm/issuer/audience and expiry,
then authoritative unrevoked session, matching user/auth epoch, enabled user,
and current policy. A process-local cache cannot overrule changed authoritative
epochs. State-changing Access operations recheck actor authorization within their
write transaction. Single-session logout does not invalidate another sid. Limit
active sessions initially to ten per user; expire/revoke excess oldest sessions
explicitly and clean expired metadata in bounded batches.

SSE sets a close deadline at the exact JWT/session expiry and checks that deadline
before writing a frame; the five-second periodic check is only for session/user/
policy changes, not an expiry grace period. Revalidate authoritative state within
five seconds, close revoked sessions, and reconcile permission changes using
current policy. Reauthentication creates a new sid; do not revive a revoked one.
Notification read/list/count operations re-evaluate current authorized ownership
in their database transaction/snapshot. Previously delivered data cannot be
erased from a user's knowledge, but no new response treats old grants as current.

This adds a precise access lifecycle mechanism, not a second login system.
No Cookie conversion or ticket service is required for the initial fetch-based
client. User-facing auth failures and disabled-module states remain distinct.

Monitor P4 protocol specification: the shared Rust module defines version 1
request, response, limits, authentication header, sequence and fencing
outcomes. The descriptor serializes those rules for release pairing;
conformance tests bind descriptor fields to the shared runtime types and
handlers.

The selected protocol artifact is canonical JSON with `artifactClass`,
`compositionId`, `descriptor`, `digest`, `preset`, and `version`; `descriptor`
is the canonical descriptor byte string, preserving wire integers beyond the
JavaScript safe-integer range. Its digest is SHA-256 of those descriptor bytes.
It is accepted only when both real
Controller and Agent command outputs parse as the same reviewed
descriptor/digest pair. Server and node-agent artifacts have distinct
compositions; caller-supplied protocol IDs, cross-class/stale artifacts, links,
extra files, and changing files are rejected before manifest production.

### Selected API artifact

Monitor P4 selected API output is canonical JSON, not full OpenAPI. It combines
only the selected Admin route contracts and Monitor module manifest emitted by
real registrations. Its SHA-256 is calculated from emitted artifact bytes;
callers cannot provide an API digest. API, schema and configuration remain
separate artifacts; protocol and native layout use their own selected artifacts.

### Selected schema artifact

Monitor server schema evidence is canonical JSON under the exact `admin` and
`monitor` owners. Each schema fingerprint hashes the corresponding final
fresh-install SQL bytes. Its data-contract ID hashes a versioned descriptor of
owner and schema fingerprint. Release-manifest callers provide only the verified
schema artifact root; direct schema digests, fingerprints and data IDs are rejected.

### Selected configuration descriptors

`rz-admin`, `rz-monitor` and `rz-monitor-agent` expose `contract config selected`
before dotenv or runtime initialization. Monitor server descriptors are owned by
`access` and `monitor`; the Agent descriptor is owned only by `monitor-agent`.
Fields describe names and validation classes without serializing values. The
canonical combined artifact rejects missing, additional, reordered or changed
field metadata and any value field. Manifest callers provide `configRoot`; a
caller-supplied `configDigest` is rejected. Server and Agent config artifacts
are separate compositions and cannot be substituted for each other.

### Canonical selected archive

The selected archive is a deterministic ustar stream rooted at
`rz-<artifactClass>-<buildId>`. It contains exactly the staged payload files
under `payload/` plus canonical `release-manifest.json`. Headers fix uid, gid,
mtime, ownership names, file type and padding; reader validation rejects PAX,
links, devices, directories, reordered members and trailing bytes. The reader
validates the archive against the selected composition and hashes canonical
manifest bytes. Member paths are scalar Unicode encoded as UTF-8; ustar name
and prefix limits are respectively 100 and 155 bytes. The detached Ed25519
envelope is outside the archive and binds the release triplet below.

### Detached selected-release envelope

Publication creates a private, atomically renamed three-file release directory:
`archive.tar`, canonical `release-manifest.json`, and
`signature-envelope.json`. The envelope is canonical JSON with exactly
`payload` and base64 `signature`. Its Ed25519 payload fixes the selected-release
domain, key ID, release class/version, target, artifact class, composition and
build IDs, archive and manifest SHA-256 values, and the Agent protocol ID.
Verification receives an independent trusted public key and matching key ID;
it never obtains trust material from the release directory. Production signing
requires a production manifest. Installer extraction remains a later boundary.
