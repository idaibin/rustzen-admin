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
