# Daily-summary runtime acceptance

## Goal and authority

Prove that the existing Monitor worker generates daily summaries from retained raw
inputs, persists them, exposes them only through the proper delegated capability,
and updates an existing summary after process restart. This is a bounded acceptance
slice of the current Monitoring product, not a new feature or deployment.

Product authority: `docs/product/product.md` and
`docs/product/features/monitoring/spec.md`. The existing `/monitoring/summaries`
page owns display; no UI, API, schema, scheduling or permission contract changes.

## Reproduction

Run `just verify-monitor-daily-summary-runtime` from the repository root. This runs
four Python fixture/oracle tests, builds the repository-locked default-feature
`rz-monitor`, then runs the real process twice against one newly created, owned
SQLite database. It needs Rust 1.95.0, Python 3 and a local loopback socket; it does
not require macOS, Docker, a browser or an external provider.

The verifier uses a unique output directory under
`target/rz/daily-summary-runtime/`; it never replaces an existing database. Logs,
database and JSON receipt remain there. Both owned Controller processes are stopped.
Python optimized mode is rejected because it would disable acceptance assertions.

## Acceptance-to-evidence map

| Layer / criterion | Evidence in this slice | Boundary |
| --- | --- | --- |
| PRD: retained per-node daily summaries | Registered sampled and zero-sample nodes produce one row each; tomorrow's registration is excluded | Synthetic historical input, current initialization schema |
| Backend: actual generation | No summary fixture inserts; unchanged startup timer invokes the real worker | Natural first tick; an elapsed hourly interval is not exercised |
| Backend: metric correctness | CPU 10/20/30, memory 30/50/70, separate `/` and `/data` min/average/max; coverage uses 2/2880 then 3/2880 | Fixed small input set, not load certification |
| Backend: incident windows | Cross-midnight and same-day incidents yield 180 offline seconds and two incidents | Historical incident fixtures, not live Agent alert evaluation |
| Persistence / restart | Add one raw sample after shutdown; second process must expose sample count 3 with exactly two persisted summaries | Proves upsert and actual second generation, not merely stale result reading |
| API: authorized read | Real HMAC-delegated HTTP response matches SQLite and expected values | Direct module API, not Admin login or gateway RBAC |
| API: rejection | Unsigned request 401; correctly signed wrong-capability request 403 | No claim about every permission combination |
| UI specification / page rendering | Existing display scope remains unchanged | Not newly verified here |
| Frontend interactions / full E2E | No new evidence in this slice | Next Ready item: display these generated rows through Admin gateway and exercise page refresh/pagination without seeding summary rows |

## Recorded run: 2026-10-04

Source baseline: `b94909de11c4e09a50815eceec123dbe8c00a6f6`.
Production Rust code is unchanged. Build command:
`cargo build --locked -p rustzen-monitor --bin rz-monitor`, Rust 1.95.0,
Linux x86_64, default features (including notifications). Runtime receipts record
the exact binary SHA-256 and the UTC day; this is local source/runtime evidence,
not a signed-release or installation identity claim.

The real runtime gate passed, including generated output, restart reaggregation,
persistence and delegated rejection checks. Four fixture/oracle tests passed. The complete `rz-monitor` binary test suite also
passed: 76 tests, zero failures, with one harness thread.
No screenshot, browser, systemd, signed installation, production provider or
cross-platform acceptance follows from this backend/API result.

## Subsequent Admin gateway closure: 2026-10-04

`just verify-monitor-daily-summary-gateway` builds the real Web distribution and full
Admin/Monitor binaries, then runs both services and API checks inside one execution
environment against fresh owned databases. The new verifier reuses the raw-input
fixture and metric oracle above. A real development-owner login yields the transient
JWT used for the actual Admin gateway; no token is saved in evidence. Twenty-one
additional registered nodes create 23 application-generated summaries.

- Backend/API integration passed: page 1 → page 2 → page 1 returns 20/3/20 rows,
  no duplicate node across the first two pages, retained total 23, matching sampled
  and empty-node oracle, SQLite row count 23, and unsigned gateway rejection 401.
- Source basis: `4af9c7e1eb38ca7c1f9dadafb2dadeace4be5a02` plus the gateway verifier,
  command and this documentation. Production application code is unchanged.
- Web production build and full Admin build passed; the existing large-chunk warning
  remains. These are build checks, not rendered-page acceptance.
- The gateway receipt records both binary SHA-256 values and successful process
  cleanup under `target/rz/daily-summary-gateway/run-*/result.json`.
- Frontend page, frontend interactions, and full browser E2E remain **blocked** in
  this environment: cloud Chrome reported `net::ERR_BLOCKED_BY_CLIENT` opening the
  local loopback URL. The server logged a successful loopback bind, but a separate
  executor request could not reach that process. No supported preview mapping was
  available in the inspected tool surface. This does not establish the exact cause
  of the browser refusal; no alternate route or security setting was used to bypass it.
- Other user roles, elapsed hourly scheduling, Agent transport, systemd, production
  and the other two modules are not certified by this owner-only API journey.

Next Ready independent acceptance: exercise permitted and denied non-owner roles
through the Admin gateway with fresh owned accounts and observe revoked access.
UI remains pending a supported reachable preview; this next step does not replace it.

## Non-owner role and revocation closure: 2026-10-04

This slice was planned before execution using the candidate `tests` guidance at
`idaibin/skills@23cc6b0a30abf15dd86cbb6cd148d13c731cca97` (no global installation or
stable-promotion claim). The project-native plan is retained under ignored
`target/rz/daily-summary-rbac/plan.json`; requirement authority remains this repository's
product specification and `docs/guides/permission.md`, not the testing skill.
The original plan is preserved with its earlier `4af9c7e` source base. Execution
was explicitly rebound to `13bd091`: the intervening commit adds only the reviewed
gateway harness/command/documentation; application binaries, product/permission
authorities, initialization schema and role expectations did not change. The final
role receipt binds the subsequently extended verifier/helper by exact hashes.

`just verify-monitor-daily-summary-roles` reuses the existing gateway fixture and adds
`--verify-roles`. The helper creates only two synthetic users and two custom roles in
the newly initialized local Admin database, using real owner-authorized HTTP endpoints.
It obtains capability IDs from the actual catalog rather than hardcoding database IDs.
It does not mutate a user's existing database or any external service. Local development
fixture credentials and session tokens are never included in the JSON receipts.

| Case | Contract / boundary | Oracle | Current state |
| --- | --- | --- | --- |
| RZA-DS-R1 | Product granted-capability rule; real Admin JWT → Monitor delegation | Custom `monitor:node:view` user receives 200 and 23 summaries | Passed |
| RZA-DS-R2 | Same route, independently authenticated custom overview-only user | 403, with exact capability list checked through `/api/auth/me` | Passed |
| RZA-DS-R3 | Least-privilege Admin native API | Summary reader cannot list roles (403) | Passed |
| RZA-DS-R4 | Current authorization after role grant mutation | Same existing JWT: remove summary capability → 403; restore → 200, without restarting services | Passed |
| RZA-DS-R5 | User-enabled/session rule and user isolation | Disable reader → existing JWT rejected 401; independent owner still receives 200 | Passed |
| UI page / frontend function / full browser E2E | Existing daily-summary UI contract | Real rendered and interactive states | Blocked by the recorded browser prerequisite |
| Concurrent actors | Distinct from the serial cases above | No concurrent schedule was exercised | Not run |
| Performance | No agreed workload/budget for this slice | No throughput/latency acceptance claim | Not run |

Run basis: `13bd0918606da54acbe21900bf889b112449f90d` plus the role helper, gateway
flag/receipt extension, command and this documentation. Production application code,
PRD, wire contract and UI remain unchanged. Results under
`target/rz/daily-summary-rbac/run-*/result.json` include exact helper/verifier/fixture
SHA-256, both native binary hashes, start/end timestamps, each observed role status,
the underlying 20/3/20 paging receipt and successful process cleanup. Four existing
fixture/oracle regressions also pass. The earlier 76 Monitor tests are unchanged-code
regression evidence, not a new 76-test execution in this role slice.

This is a real API-entry-to-storage journey with serial permission changes, not a
browser-entry E2E or a general certification of all roles. Source review and repository
delivery remain separate gates. Next independent acceptance candidate: bounded
Monitor outage/restart recovery through the Admin gateway; UI stays pending a supported
reachable preview and is not replaced by that candidate.

## Owned Monitor outage/recovery closure: 2026-10-04

This existing product requirement was selected before implementation: module failure
must not prevent Admin login; an enabled, authorized destination remains discoverable
when its service is unavailable. The persisted navigation assertion below is an API
contract check only, not proof of the browser sidebar or search rendering.

Pre-execution plan: `target/rz/daily-summary-recovery/plan.json`, SHA-256
`f9416f7810d886ff1ee39c74ed782dd8957656e96d2a2fd46aeeb48cd2598b64`,
created before the first run and tied to `ba566a2f77d46c516d32e5cdbf4ee6f8db8f10cd`.
The `tests` candidate at `23cc6b0a30abf15dd86cbb6cd148d13c731cca97` directed the
contract/case/oracle/evidence separation. Final receipts embed the original plan,
its pre-run digest, verifier and helper hashes, binary identities, timestamps and
cleanup; a changed plan during execution fails the run. The production application
and accepted contracts are unchanged; this slice only extends the harness and docs.

Run `just verify-monitor-daily-summary-recovery <plan-path>` after preparing a current,
timestamped acceptance plan. This builds/runs the ordinary gateway gate, then performs
one owned Monitor termination and restart. It does not stop another process or change
network/security settings. A 30-second discovery/recovery deadline accommodates the
existing ten-second module sync loop; it is a harness resource bound, not an agreed
production SLO or a performance acceptance threshold.

| Case | Real boundary / oracle | Result |
| --- | --- | --- |
| RZA-DS-F1 | After the owned Monitor stops, gateway returns 503 / code 40001 / null data and an unavailable message, never a successful empty page | Passed |
| RZA-DS-F2 | Admin remains alive, `/health` returns 200, fresh owner login and that fresh session's `/api/auth/me` both return 200 during outage | Passed |
| RZA-DS-F3 | Wait until real discovery reports Monitor enabled but unavailable; persisted navigation API still returns the identical Monitor paths including summaries | Passed |
| RZA-DS-F4 | Restart same binary/database; original owner JWT regains 200; page-two payload and total 23 equal the pre-outage result; module becomes available | Passed |

First receipt: `target/rz/daily-summary-recovery/run-f7des5fi/result.json`.
The main process and replacement Monitor were stopped in the common cleanup path.
Four fixture/oracle tests passed again. Browser page/function/E2E remain blocked;
other-module availability, concurrent users, production load and performance remain
unverified. A future Ready candidate is a small fixed local concurrent-read workload
that records correctness and latency observations without inventing a product SLO.

## Fixed rate-limited read observation: 2026-10-04

Pre-run plan SHA-256:
`c96f6ddd1da735ff7945c83eba9e0cfb21d8050a3caa8485e531f5b536d598a4`,
`target/rz/daily-summary-load/plan.json`, source base
`38959f326252ac1e7f1cee59a8030e69ff45db5e`. The plan was frozen before implementation
and execution, then embedded and verified unchanged by the receipt. Only test code and
documentation changed; the native binary and 23-row raw-input fixture identities are
retained in the result.

Workload: one owned owner JWT, four client worker threads, 100 measured GETs per run,
page 1/2 alternating, global dispatch spacing of at least 100 ms (not 10 RPS per
client), at most four in flight, 30-second dispatch budget, and a 1 GiB combined owned
service RSS abort ceiling. Any payload/status mismatch stops further dispatch. No
production/third-party target, write workload or automatic load escalation is involved.
The ordinary gateway gate's readiness and 1/2/1 checks precede the measured requests;
there is no separate load warmup or sequential comparator.

Two real measured runs were executed, **200 load GETs in aggregate**:

1. `run-epg9_mz1`: first 100-GET observation. The client timer included resource-sample
   overhead. Preserved as an initial observation, not silently overwritten.
2. `run-ac9jv4fn`: second 100-GET regression after fail-closed diagnostic preservation
   and moving the latency timer immediately around HTTP request/response. This final
   receipt is bound to its exact verifier/helper hashes. No further real load replay
   was performed for this slice; independent review uses these receipts and mocked
   deterministic harness checks.

Both have 100/100 responses equal to the fixed expected page payloads and zero errors.
The final single-point observation records p50 5.07 ms, p95 7.26 ms (nearest-rank,
100 samples), min 4.16 ms, max 9.82 ms, and observed combined service RSS peak
81,674,240 bytes. The recorded 10.08 completed responses/second divides by time from
first dispatch to last completion; finite-window endpoint effects explain why that
number is slightly above the 10-per-second dispatch spacing. It is not a capacity claim.

Crucially, **maximum observed in-flight requests was one** despite four worker threads.
This closes bounded rate-limited read correctness only. Actual overlapping requests,
multi-user concurrency, performance-budget/SLO acceptance, improvement over a baseline,
production capacity and tail-latency certification remain **not run**. No p99 claim is
made. Shared-executor scheduling and client-generator overhead are not isolated.

Three mocked harness checks prove the exact request cap, stopping on a wrong payload,
and RSS-ceiling refusal before a request; the four fixture/oracle tests also pass.
These seven tests are not additional real HTTP load. Both service processes were stopped
and the final receipt records cleanup. Reproduction is explicit:
`just observe-monitor-daily-summary-reads <current-plan-path>`; it is not a recurring run.

Next Ready: a separately preplanned, at-most-20-request synchronized cohort correctness
check with measured client request overlap. It must not be presented as this strict-rate
observation, and it does not replace the still-blocked browser UI/E2E acceptance.

## One overlapping-client cohort: 2026-10-04

A separate plan was frozen before this new experiment:
`target/rz/daily-summary-overlap/plan.json`, SHA-256
`ffc3c02a083eed17f72de68595d980e404653063197319de3f2d52684b3c1f15`,
source base `972bedd6fab22d54111223acf1563a2b24ecb5e0`. This plan explicitly replaces
the previous strict dispatch-spacing workload for this one experiment only; it does
not increase or repeat that 100-request workload.

Exactly one cohort of four real GET requests ran after a four-client barrier, using
the same owned owner JWT and pages 1/2/1/2. The barrier ends before each HTTP timer
starts; no sleep or artificial hold extends measured request intervals. Each request
uses the existing two-second HTTP timeout; the barrier has a five-second timeout.
There is no automatic repeat if overlap is not observed.

Receipt: `target/rz/daily-summary-overlap/run-2hkprz3p/result.json`. All four responses
were 200 and exactly matched the corresponding previously validated payloads. The
recorded client HTTP intervals overlap with a computed maximum of four. Three pure
interval-oracle tests separately cover overlapping intervals, touching intervals,
zero-duration intervals and invalid ordering. The existing three mocked load-budget
checks and four fixture/oracle checks also pass; none creates additional real GETs.

This proves one bounded overlapping-client read-correctness scenario. It does **not**
prove simultaneous server critical-section execution, multi-user/concurrent-write
correctness, performance/SLO acceptance or production capacity. The browser UI and
browser-entry E2E remain blocked. The four cohort GETs are additional to the previous
200 measured load GETs; normal fixture readiness/paging calls remain separately scoped.
Both owned services were stopped. Review must not replay this one-cohort experiment
without a separately approved budget.

After the one authorized run, only two CLI help strings and generic receipt-limit
wording were clarified in the gateway runner; no scheduling, HTTP, assertion or
cleanup behavior changed. The exact at-run runner was preserved as
`run-2hkprz3p/verifier-at-run.py` and verified against the receipt's original SHA-256.
The overlap helper and its oracle are unchanged. Evidence reuse is limited to this
metadata-only difference; no second cohort was silently executed.

The current backend/API branch has closed generated-summary, gateway paging, serial
role/revocation, owned outage/recovery and one overlapping-client read scenarios.
Broader Monitoring/Agent workflows and browser acceptance remain distinct open work.
