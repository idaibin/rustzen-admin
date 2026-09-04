# Architecture review record

This is the completed historical design-review record. Subsequent execution
follows the current fresh-only AGENTS.md; historical upgrade/rollback suggestions
below are superseded by [implementation](implementation.md), not fresh approval
to add compatibility code. The ten captured responses remain unchanged.

Status: reviewed design; external ChatGPT rounds completed: 10 / 10.
All ten responses were captured in full, read back and reconciled locally.
Final external verdict: design PASS WITH CHANGES; runtime NOT VERIFIED.
The final two corrections were made locally and were not sent for an eleventh
review. Coordinator disposition: ready for staged implementation, not release.

## Basis and authority

- User request: complete and physically pruned Rustzen deployments, optional
  message center/realtime integration, ten ChatGPT design/review rounds, bounded
  practical scope and community best practices; subagents explicitly authorized.
- Local checkout HEAD: `fa18de2909bc456eea42e6b6b891efa30b2c4297` plus existing
  staged/unstaged/untracked source captured in the ignored review basis ledger.
- Existing dirty work was retained. No implementation, build, package, commit,
  push, database removal or deployment is authorized by this design record.
- A read-only investigator verified current coupling in build/package/install,
  Admin startup, module delegation, frontend routes and recovery.
- Coordinator owns all candidate text and locally verifies external advice.

## Initial local dispositions

| Observation / recommendation | Disposition |
| --- | --- |
| Current module-disable flags do not remove shipped binaries/schema/Web | Accepted; build selection is required |
| Minimal Admin host + Monitor is less disruptive than standalone auth inside Monitor | Accepted as initial topology |
| Full-only installer, package validator and recovery loops cannot be reused unchanged | Accepted; generated selected inventory must govern all three |
| A separate copied `admin-monitor` application might make selection clearer | Not selected; first separate composition from features in the existing host, avoiding two identity implementations |
| Only full and monitor should initially be certified | Full/monitor are the first implementation proof; other presets must pass their own gates before release |
| No cross-host control-plane or general cluster topology | Accepted; node Agents remain the explicit remote boundary |

## Independent local review and coordinator corrections

A separate read-only reviewer inspected the six-document candidate and relevant
source. It reported two P1 and three P2 design findings. These are local review
results, not ChatGPT rounds or executed implementation tests.

| Finding | Coordinator disposition | Required validation |
| --- | --- | --- |
| P1: source/toolchain in selection digest prevents normal upgrades | Accepted; split stable compositionId from exact buildId and file hashes | D14, D17, D21 |
| P1: pending-only budget leaves terminal outbox and Admin growth unbounded | Accepted; terminal deletion, bounded quarantine, total Admin row/byte admission budgets, receipt protection and shared-disk limits | N29, N30, N34 |
| P2: Reports has no persisted trusted initiator | Accepted; capture verified manual actor at run creation; scheduled runs initially send no personal notifications | N31, N32 |
| P2: queued cancellation skips the proposed event path | Accepted; event follows every winning legal first terminal transition | N25, N33 |
| P2: minimal install recovery still depends on absent release management | Accepted; nonresident CLI executor and disk journal own apply/recover; optional UI is a caller | D17, D18 |

Coordinator also made the combined Monitor/Agent binary split explicit (D19)
and clarified essential account/role UI ownership (D22). These corrections are
in the candidate and were supplied to the subsequent external rounds.

Document checks passed for all six files: local links, balanced code fences,
English-only content and whitespace. The three existing documentation index
edits passed `git diff --check`. The initial design contained 22 distribution and 34
notification/access acceptance scenarios; these scenarios have not been run.
Final baseline comparison detected concurrent changes in existing frontend,
Reports and related product/UI documentation. Their hashes are recorded in the
ignored final validation receipt. The coordinator did not edit or revert those
paths; they remain outside this design-only change set. Source observations in
review prompts belong to the frozen basis, not a claim to have re-audited every
concurrent edit. Slice 1 must refresh the source+dirty baseline and existing
journey oracle before implementation begins.

## External review transport

Provider: ChatGPT. Actual target: one new conversation inside the configured
`AI Review` Project, through the Codex in-app Browser. The provider and round
count are authorized by the current user request. Persistent Project selection
comes from the existing ask-ai defaults. Login was initially required; the user
completed sign-in while local design continued. The AI Review Project and
personal-account surface were verified before submission. The visible model
label is Pro; an internal model ID is not verified. No browser switch occurred.
A separately frozen candidate was submitted once per round in the same
conversation:
`6a9916dc-1934-83e8-a3be-5900c53340b1`, inside Project
`g-p-6a7052bb5bf88191900a8a0c869c5291`. All ten attributed responses have complete
final artifacts with independent content hashes and filesystem readbacks.
[Open the review conversation](https://chatgpt.com/g/g-p-6a7052bb5bf88191900a8a0c869c5291-ai-review/c/6a9916dc-1934-83e8-a3be-5900c53340b1).

## External dispositions

### Round 1: scope and ownership

Result: Revise. Full response captured as 6,526 characters; response SHA-256
`1356b6cecc4dcf28fbd980265cdb69a82b66262a2e40653b7bea39cb57200236`.

- Notification authorization objection: the first package omitted the preceding
  user request about message center/alerts. Reject the implied need to ask again;
  supply that context next round. Accept the wording correction that notifications
  are target behavior, not already released, and keep pruning independently gated.
- Full completeness: accept an independent current-behavior inventory before
  catalog generation; add orphan/duplicate/missing-owner gate D23.
- Minimal Access: accept a closed owner matrix including routes, schema, tasks,
  configuration, bounded audit and optional diagnostics ownership.
- Agent ingress: verify the current Monitor-owned shared secret, manual rotation
  limit and best-effort 30-second collection/5-second timeout contract. Keep one
  public ingress; explicitly accept Entry outage coupling and add D24.
- Agent/controller symmetry: accept artifactClass identity and controller-only
  server targets with reverse absence gate D25.

The fixed external result is advice, not implementation proof. Its source claims
about the repository were explicitly not independently verified; coordinator
checked the relevant local registration, schema, Agent loop and token middleware.

### Round 2: Cargo and artifact identity

Result: Revise. Full response: 4,598 characters; SHA-256
`b5305ade19e7b0483f574d1d49f8e4c3f61363768ca7b2773aeffba016c4f70a`.
All five findings accepted with bounded corrections:

- Freeze current resolver 2 and per-package/bin/target/features/default-edge
  plans; inspect actual per-binary outputs, not only cargo tree.
- Include OpenAPI, config/CLI, registered tasks and compiler-generated outputs
  in owner inventories; D26 probes each leakage class.
- Treat buildId as plan identity; use exact Web content digest and a defined
  nonrecursive HTML stamp/signature process for pairing (D27).
- Specify server/node-agent manifest union and forbidden fields (D28).
- Add durable dataContractIds and explicit tested rollback compatibility; initial
  semantic data changes also require a fresh root (D29). This is stricter than
  accepting arbitrary same-DDL forward changes and avoids a migration framework.

The thin composition-binary alternative remains viable if the existing app
cannot pass per-binary absence gates. It does not justify copying identity or
business implementations now. The root Cargo.toml already pins resolver 2;
Cargo target/build-script/tree limits were checked against official sources.

### Round 3: frontend and bootstrap

Result: Revise. Full response: 2,980 characters; SHA-256
`7c332c1c0a337ec45090f95c772c9980a9767dc99d26bff9890aec555635ca20`.
Accepted four findings: explicit selected TanStack discovery/output directories,
typed frontend-operation/backend-contract pairing, anonymous pre-login Web
binding with bounded cache recovery, and ordered route states/cache purge/
authorized landing. Corrected stale D21 wording to compare webDigest, not plan
identity. Added D30-D33. The coordinator also closed Vite publicDir copying and
the contract-export/Web/final-host build-order dependency. Relevant TanStack and
Vite behavior was checked against their official documentation.

One state-order detail is deliberate: uninstalled routes return 404 before
authentication rather than exposing an installed-service state. Installed
protected routes check authentication/authorization before availability.

### Round 4: identity and trust

Result: Revise. Full response: 4,783 characters; SHA-256
`1459b034ee5c4d5d8229c34def150650b057c92a815dcedd2940f0247ed2901c`.
Accepted controlled local first-owner initialization, atomic post-mutation owner
invariant, explicit session/authentication/policy epochs and separate directional
HMAC contracts/key rotation. Added D34-D35 and N35-N39; local JWT claims/HS256
and request loader were inspected, and SQLite/JWT/signature mechanisms checked
against primary documentation.

Two refinements avoid unnecessary complexity: policy changes refresh current
authorization instead of forcing all identity sessions to log in again; modules
consume short-lived signed decisions rather than replicating Admin's role DB.
Revocation blocks new decisions, while an already-issued five-second permit may
be admitted and already accepted work may finish. This is an explicit admission
boundary, not a promise of retroactive cancellation. Shared Agent credentials
remain an explicitly bounded initial trust model; the reviewer did not find a
new requirement for per-node identity.

### Round 5: persistence and composition changes

Result: Revise. Full response: 12,284 characters; SHA-256
`f1aafdb5ada142d2e1c9e23a9a0d50c4449633271f960a42d3cee7c99ad19aa6`.
Accepted selected-schema query validation, read-only observed-schema admission,
owner data descriptors/conformance fixtures, crash-atomic fresh DB publication
and fragment object ownership. Added D36-D41. Current source uses dynamic SQL;
therefore query integration tests are mandatory and macro metadata isolation is
conditional, not a forced SQL rewrite. Descriptor hashes cannot prove arbitrary
semantic compatibility; owner review and prior-build fixtures remain required.

### Round 6: message reliability

Result: Revise. Full response: 2,536 characters; SHA-256
`b3baf63d9a6a9c8d360bdbf930f7d694ee4eeed692b9325c817b5f7125973ee7`.
Accepted fenced claims and head-of-subject eligibility, a bounded post-expiry
ambiguity window with a separate unconfirmed counter, and non-reused inbox
sequences plus conditional idempotent reads. Added N40-N42. These are local
transaction rules, not a new queue service or a claim of exactly-once delivery.

### Round 7: SSE and browser lifecycle

Result: Revise. Full response: 2,411 characters; SHA-256
`a144583004252a310fbeae83c1d76df205922805fe691f62d0a20abfa79c8bcb`.
Accepted registered-subscription and DB-snapshot barriers, cancellation outside
slow socket progress, and bounded fetch/status/browser recovery states. Added
N43-N45 and narrowed expiry acceptance to server-side frame admission: network
bytes already sent cannot be recalled. No SharedWorker, Service Worker or push
service is required. Fetch and browser lifecycle primary references checked.

### Round 8: installation and recovery

Result: Revise. Full response: 3,339 characters; SHA-256
`aa86430a0f35dfb9baed053e8490e7842e07d977b2b25d44c681005b46a9c4b2`.
Accepted explicit root/service ownership, stable recovery launcher and ordered
unit/reload/admission journal, same-object safe extraction and rollback recheck.
Added D42-D46. Recovery must not wait on services ordered after itself.

Rejected removing release-ui installation from target full: it would silently
remove an existing required journey. Instead, preserve current API permission
checks and a fixed, constrained one-shot installation request; only the executor
writes privileged state. The optional spool/unit/OS rule disappear with release-ui.
This refinement requires D43 and final integrated review; it is not claimed
externally approved. Reports fixture runs through the final generated unit.
Systemd and Linux path-resolution primary documentation were checked.

### Round 9: executable gates and capacity

Result: Revise. Full response: 2,242 characters; SHA-256
`bf75e63275bc7577d30719f5cc362bea68157ee00119c7375c370e5a5c54fdd4`.
Accepted existing-journey behavior oracles in addition to inventories, explicit
test-only current-full-regression identity and production rejection, and exact
durable-acceptance totals with nonce/rate budget alignment. Added D47-D48 and
closed measurement defaults; the three independent fixture counts fit the
initial row limits. No new test framework or service was introduced.

### Round 10: integrated final review

Result: Design PASS WITH CHANGES; Runtime NOT VERIFIED. Full response: 2,365
characters; SHA-256
`b2b6419e6dd931ddf7c7ab380536283391e1e1f97efb38c61a4c7c46337d5942`.
No new P1; two P2 contract gaps were accepted and corrected locally:

- Bind release-ui authorization and request-ID deduplication to exact current/
  target build IDs and archive/manifest/envelope hashes. Content-addressed,
  no-replace spooling and root-side tuple recheck close the pre-copy substitution
  window (D49).
- Add signed, code-derived agentProtocolContractId to matching Monitor/Agent
  artifacts, selected build inventories, replacement and pre-start pairing
  gates (D50). The offline signed profile proves the declared pair; operator
  endpoint provisioning is not represented as verified live discovery.

The external reviewer found the overall full/minimal composition, persistence,
trust, realtime and capacity design consistent enough to implement in slices.
It did not verify source/build/runtime behavior. The revised bytes after these
two corrections were not externally re-reviewed; no unconditional ChatGPT
approval of them is claimed. There is no unresolved design blocker identified
by the coordinator. The 50 distribution and 45 notification/access scenarios
remain planned acceptance tests, not executed results.

## Ten-round agenda

Each round is a separate actual submission and attributed response. The
coordinator freezes that round's candidate hash, sends only relevant sanitized
facts, captures the full response, verifies findings, records dispositions and
revises the candidate. Later rounds explicitly see accepted/rejected changes;
they are an iterative review, not ten independent reviewers.

| Round | Focus | State |
| --- | --- | --- |
| 1 | Product boundary, physical absence and two-process monitoring alternative | Captured; findings reconciled |
| 2 | Finite capability model, Cargo closure and build contamination | Captured; findings reconciled |
| 3 | Frontend route/assets/API-client exclusion and selected navigation | Captured; findings reconciled |
| 4 | Minimal identity, RBAC, delegation and producer trust boundaries | Captured; findings reconciled |
| 5 | Per-feature initialization, DB ownership and feature-set changes | Captured; findings reconciled |
| 6 | Inbox/outbox crash windows, dedupe, ordering, recipients and bounded loss | Captured; findings reconciled |
| 7 | SSE framing, revocation, reconciliation, backpressure and client lifecycle | Captured; findings reconciled |
| 8 | Signed packaging, installer/recovery, selected units and rollback | Captured; findings reconciled |
| 9 | Failure tests, realistic capacity targets and minimal implementation slices | Captured; findings reconciled |
| 10 | Integrated final candidate: contradictions, residual blockers and implementation readiness | Captured; final two P2 corrected locally |

Do not count an agenda, a prepared prompt, a partial response, a model's claim
to have reviewed ten times, or local subagent work as a completed ChatGPT round.
If round ten raises a material issue, record and resolve it locally with evidence;
do not claim ChatGPT approved the revised bytes without a new authorized round.

## Evidence storage

Operation/basis/prompt/raw-response files live under the repository's ignored
`.codex/reviews/` with the `rz-composable-20260903` prefix. Each real external
round requires its own prepare/submit/capture identity, full response artifact,
hash and final-path readback. This public record summarizes conclusions only;
it contains no private account details or authentication state.
