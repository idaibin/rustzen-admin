# Reports future-schedule module contract acceptance

## Contract and pre-run identity

Authority: `docs/product/product.md`,
`docs/product/features/scheduled-report-automation/spec.md`, the current Reports
schedule DTO/service/error mapping and shared module JSON extractor. This slice checks
existing daily/weekly schedule metadata and CRUD only. It adds no new cadence, secret
storage, provider, browser behavior or product capability.

Baseline: `fb0edee90900bdde4995acab88b0ea4991ab6517`. The one run was planned before
execution using the candidate `tests` guidance. Plan:
`target/rz/reports-schedule-contract/plan.json`, SHA-256
`0b73fc7b3f1cdec3a0463fa3a08f9f5fd6562f8f18745b98dec28b4ca1e0055e`.
It binds the current authority files, runner and actual Rust executable before launch.
The runner verifies its own and the binary's hashes before launching and again after
shutdown; the plan digest must also remain unchanged.

The default-feature Reports binary was built with Rust 1.95.0, Linux x86_64, one build
job, development debug information disabled, 512 codegen units and incremental builds
disabled. This controls the large Chromium protocol dependency's build resource use;
it is not a signed release build. Build completed in 4m 16s. Binary SHA-256:
`a22a4ac2829dbc9c0a59dabde5a3b348e5af4172129773b7fe20596a559514fc`.

## Finite case budget and safeguards

`just verify-reports-schedule-contract <current-plan-path>` runs at most 24 signed or
unsigned module HTTP requests against a newly initialized owned Reports database.
Requests directly exercise the Reports HMAC module boundary. There is no Admin login,
JWT, role assignment or gateway in this slice, so those layers are not certified.

The system target and notification ingress both point to an owned loopback sink.
Any target/notification request fails the experiment; no external URL is contacted.
Before and after each API action, the runner checks no worker child process exists and
queries actual SQLite for zero runs, schedule occurrences, artifacts and notification
outbox rows. Source inspection established that browser launch requires claiming a
queued run. The fixture never creates a run and deletes both test schedules before
shutdown. The created target/template stay only in the retained owned database.

Enabled schedules have next-due metadata at least two hours in the future. The runtime
phase lasts about half a second, then stops both the owned service and sink. It does
not wait for the subsequent 15-second scheduler poll or for a schedule to become due.
The zero-execution claim is limited to this observed fixture window; real due-time
admission/execution, skipped occurrences, browser rendering and delivery remain separate.

## Observed cases

| Case / actual boundary | Expected and observed result | State |
| --- | --- | --- |
| Installation settings | UTC from the real module response | Passed |
| Daily/weekly metadata and CRUD | Disabled schedules have null nextDue; enable yields valid future UTC nextDue; reads and list preserve IDs/cadence | Passed |
| Module capability isolation | Correctly signed schedule-view context cannot write, 403 | Passed; not a user-role/JWT test |
| Schema validation | Unsupported cadence returns 422 through shared JSON rejection | Passed |
| Business validation | Daily weekday, missing weekly weekday, invalid time and fake sensitive input return 400 | Passed |
| Rejected-write persistence | Schedule list remains exactly two after rejected creates | Passed |
| Deletion and missing resource | Delete succeeds, deleted read returns 404, final list empty | Passed |
| Unsigned module access | 401 | Passed |
| No unintended execution | 46 guard checks; zero runs/occurrences/artifacts/outbox and zero loopback sink hits | Passed during this short run |
| Due execution, Admin gateway/user roles, real browser UI/E2E, notifications | Not exercised | Not run; existing UI environment blocker remains |

The 400 versus 422 oracles were derived from current typed extractor/service mappings
before execution. A rejected fake field named `password` is not a real user secret;
neither private credentials nor actual provider configuration are used or published.

## Receipts and focused regressions

Exactly one native runtime execution completed with **22 HTTP requests**:
`target/rz/reports-schedule-contract/run-j63jl1_4/result.json`.
The receipt retains the original plan, pre/post runner/binary digests, timestamps,
responses, guard count, final zero-row counts and cleanup. Do not silently repeat the
one-run HTTP budget during review.

The following selected Rust unit filters also passed, without starting the service,
browser, notification relay or any external request:

- `features::automation::scheduler::calendar::tests`: 4 passed, 64 filtered out;
- `features::automation::validation::tests`: 2 passed, 66 filtered out.

Calendar functions cover the bounded enqueue window, missed slots, weekday calculation,
DST gap/fold handling and effective-at admission. These function results do not prove a
real scheduler timer fired or a browser run completed. Python compilation and
`git diff --check` passed. Overall browser/full-product acceptance remains incomplete.

## Separate finding: weekly DST next-due horizon

Independent review reproduced an existing P2 outside this UTC-only receipt:
`next_due_at` for a weekly Sunday 02:30 schedule, queried at
`2026-03-02T12:00:00Z` in `America/New_York`, returns null. The March 8 slot falls in
an actual DST gap; the next valid slot is March 15 at `06:30:00Z`, beyond the current
eight-day forward search. The accepted schedule spec requires skipping the gap and
advancing to the next valid slot. This finding was recorded as open at the UTC-only acceptance checkpoint. The
separate source correction and pure regression evidence below do not rewrite that run. No HTTP receipt is being rewritten or replayed,
and no all-timezone scheduling claim follows from the UTC acceptance above.


## Minimal source correction and pure regression

The next-due forward search now includes day 14. This covers a query just after a
weekly slot, a DST-gap slot seven days later, and the next valid slot fourteen days
later. The backward occurrence scan, earlier-offset fold rule, 60-second admission
window, run creation and HTTP status policies are unchanged. No version was bumped;
the behavior correction is recorded under `Unreleased`.

The pre-execution pure-fix plan is
`target/rz/reports-weekly-dst-fix/plan.json`, SHA-256
`fef29cf098631b1349dd181c8f005677d024313c4df3d352722e2e4eabfad8f8`.
It binds the original calendar, new regression source, planned corrected calendar and
Rust compiler hash before the red test. Its original `fb0edee` base is preserved;
rebinding to parent `d734d4e` is justified because that intervening commit contains only
the UTC acceptance harness/commands/docs and leaves the application source identical.

- Red: the new weekly test on unchanged calendar code fails, actual null versus
  March 15 `06:30:00Z`, when queried after the prior Sunday slot. Full diagnostic is
  preserved in the task-local `rustzen-reports-dst-red.log`.
- Green: the same weekly case also checks the March 2 query. A separate daily case
  checks the upcoming March 8 gap rather than only dates after it. All six calendar
  tests and both input-validation tests pass; existing UTC, fold, missed-window and
  effective-at cases remain in that focused suite.
- The default-feature Reports binary builds with the same bounded development profile.
  No HTTP, scheduler process, job, browser or notification was run for this correction.
- The earlier native HTTP executable is retained as
  `run-j63jl1_4/rz-reports-at-run`, verified against its original receipt hash before
  the new build replaced `target/debug/rz-reports`.

The P2 is corrected and covered at the pure calendar boundary. This is not an
all-timezone certification; at that pure-correction checkpoint, fixed-calendar HTTP and actual due execution had
not been replayed. The separate current-date smoke below adds only its stated boundary. The original UTC receipt remains earlier-source evidence, not a claim
that the newly built native process has been exercised.


## Fixed-binary current-date IANA smoke

A separate, explicitly bounded plan authorized exactly one smoke after the source fix:
`target/rz/reports-fixed-calendar-smoke/plan.json`, SHA-256
`bbd8c12201877f489684008576f8d79b38775e96b40b3eb149e7cd32d6cd2404`.
Source base: `d10adc8b89fa70c610616fb435fe7917fc717280` plus the smoke-mode harness
extension. The corrected Reports executable is pre-bound by SHA-256
`2decca24fc6419f1b973b2ca7c00d956ea937607cd4f208a196b671ff6581a98`;
runner, executable and plan hashes are verified before launch and after shutdown.
The earlier 22-request runner was preserved as `run-j63jl1_4/runner-at-run.py` with
its original hash, rather than pretending the extended runner was used in that run.

`just verify-reports-fixed-calendar-smoke <current-plan-path>` invokes the explicit
`--smoke-fixed-calendar` mode, with a hard eight-request cap. It does not rebuild an
unbound executable or silently run the full 22-request contract. The operator first
builds and binds the desired binary in the plan, as with the original gate.

Exactly eight module requests passed: settings, owned target, owned template, future
daily create, future weekly create, list, and both schedule deletions. The installation
fixture uses `America/New_York` only in the child application's environment. Neither
the OS timezone nor the system clock changes. Both nextDue values are at least two
hours ahead and round-trip from UTC into the installation timezone to the expected
HH:MM and, for weekly cadence, weekday. List preserves both created IDs.

Receipt: `target/rz/reports-fixed-calendar-smoke/run-jaxbl6yr/result.json`.
Eighteen guard checks observed zero child processes, runs, occurrences, artifacts,
notification outbox rows and sink requests. Final schedule count is also zero. The
owned process and sink were stopped. No automatic repeat or additional 22-request
run occurred.

This now proves current-date metadata HTTP wiring on the corrected native binary.
It does not execute the spring DST transition: the historical gap is covered by the
six pure calendar regressions, while this smoke uses the actual current date. Actual
due jobs, real browser/PDF rendering, notification delivery, Admin JWT/user-role
integration and rendered UI/E2E remain unverified. The old full UTC rejection/capability
receipt retains its original executable/source identity; it is not rewritten as a full
post-fix replay.

## Production single-poll / SQLite lifecycle integration

Follow-on basis: `5c1ef0fd7e3ca786c4c61d0cef05e59842e08268`. The Reports scheduler
had pure calendar and repository transaction tests, but no direct integration of
`process_schedules_once` with service readback after reopening a file database.
Two new tests in `scheduler/tests.rs` cover this accepted contract. The only change
in production source is a `cfg(test)` module declaration; runtime behavior is unchanged.

The fixture uses a fresh SQLite database, actual migrations, the production poll,
service readback/cancellation and startup recovery entry, with an injected fixed time.
It never calls `spawn`, starts a server, launches a browser or sends a notification.
The target is disabled and points at loopback; the browser path is nonexistent.
Closing and reopening the pool proves persistent state rather than an in-memory cache;
it is not an actual operating-system process restart or timer-driven execution.
Schedules are inserted through the repository fixture helper with an empty stored flow;
this does not certify schedule-create validation or executable-flow acceptance.

- Daily and Monday-weekly schedules produce no decision before due, then exactly two
  queued runs and two linked decisions at the inclusive +60-second boundary.
- A queued run cancels through the real service. After pool reopen/startup recovery,
  both a same-window enqueue poll and a +61-second skip attempt retain exactly the
  original two decisions/runs and the cancelled run's link/input snapshot.
- A fresh +61-second poll produces one `missed` decision with no run. Disabled and
  post-due-effective schedules have no decision. Reopen and repoll preserve that state.
- All scheduled initiators remain null, artifacts/outbox stay empty, output directories
  are not created, SQLite `quick_check` succeeds and owned fixture directories are removed.

Pre-execution plan revision 4:
`target/rz/reports-due-integration/plan-v4.json`, SHA-256
`ea3cf9134efa706609748406f741ee9d6f19e34514af19026cd191d3cded2094`.
It preserves the original plan and records an independent review improvement before
execution: replay the Enqueue branch after reopen, then the late Skip branch. Executable
identity and final results are recorded separately in the execution receipt.

The first compile exposed a fixture-only SQLx 0.9 static-query requirement; the first
focused run then failed because the raw migrations omitted the optional notification
schema. The fixture now calls the production `run_migrations`. Both failures and plan
revisions are retained; neither required changing application behavior. Both failed
fixture directories were verified as owned and removed. The corrected focused run
passes 2/2; the complete default-feature Reports suite passes 72/72 (including local
loopback notification-transport tests, without real delivery).

Final execution plan: `target/rz/reports-due-integration/attempt2/execution-plan.json`,
SHA-256 `ac60ff19dc61bbaaad870439982b1e8036fc3833fc5c2431c9cdcb5b808d2699`.
It binds the actual test executable before execution. The sibling `result.json` records
commands, timestamps, exit codes, log hashes and post-source/binary equality. Scoped
formatting passed with the existing stable-rustfmt nightly-option warning. Final
`cargo check --locked -p rustzen-reports --all-targets` and scoped Clippy with
`-D warnings` both passed. Source and test executable hashes remained unchanged.

The current `tests` 0.1.1 guidance drives focused checks during editing and the complete
applicable Reports gate before submission: `just verify-reports-backend`. Its commands
were run directly because this restored environment has no just executable; the new
recipe itself has not been parser-executed here. The existing full workspace
`just check` is broader; this test-only slice does not claim that gate passed. Monitor,
Insights and frontend application sources are unchanged, so their scoped prior evidence
remains reusable with its original identities and limitations. Real UI/E2E remains
blocked by the recorded preview refusal. No complete project/release verdict follows.

## Real due-worker failure and process-restart acceptance

Next basis: `7ad98ef2cb2a7bed700749e4aea40e4ac6b655af`, with a new bounded Python
runner and oracle tests only. The native default-feature Reports executable was rebuilt
from that source. This slice closes the negative timer/worker lifecycle gap without
executing a browser: the operator creates a valid target, goto flow and near-future
schedule, then disables the target before due. The production worker must retain a
failed run with the exact `target system is disabled` error, linked to one `enqueued`
occurrence. The same database is reopened by a real second service process.

Pre-run plan: `target/rz/reports-due-worker/plan.json`, SHA-256
`43f5fa0eea52c82c0a196e860679b6a4e513a4959542252f639113416a10216c`.
The runner, executable, Reports source/migration authorities and test oracle are hashed
before execution and verified afterward. `tests` 0.1.1 guides the case/evidence boundaries.

The finite budget is eight module HTTP attempts, one owned process restart, a 120-second
terminal-observation deadline, and a 180-second whole-run dispatch deadline. Startup,
request and shutdown calls have their own small timeouts; these are not OS resource
limits. The new run is scheduled 30–90 seconds ahead, with a minimum 25-second margin
after creation and confirmed disabled target at least 20 seconds before due. No system
clock/timezone changes occur. The child application uses UTC only through configuration.

Safety checks run before/after requests and during waiting: no child process, target or
notification sink request, step, artifact or notification outbox row. The browser path
also points to a nonexistent owned path. Child detection is sampled rather than a
continuous process trace; the source-resolved disabled-target branch precedes browser
execution. Any failure stops the experiment; there is no automatic native replay.

The two pure oracle checks cover UTC minute/day rollover safety margin and rejection of
incorrect terminal errors/status/timestamps. They do not simulate the complete worker.
The single real run passed: exactly eight HTTP attempts, one process restart and
919 guard checks, from 2026-10-04 13:07:52 to 13:09:23 UTC. The actual scheduler
persisted one `enqueued` occurrence and the run failed with the exact disabled-target
error before any step. HTTP schedule/run readback agreed with SQLite. After restarting
the native process and observing another 16 seconds, both complete run and occurrence
objects were unchanged; counts remained one each and the initiator remained null.
Artifacts, steps, notification outbox, sink requests and observed child processes stayed
zero. SQLite `quick_check` passed. Both owned service processes and the sink stopped.

- Receipt: `target/rz/reports-due-worker/run-q9rmtcgp/result.json`.
- Receipt SHA-256: `abdbdb28953f26d442f1ce0cf72dae9fb21c06739ad425cc01297ef8b7b33922`.
- Native executable SHA-256: `2bd19e654315e5cc43ebfd3a5c3f6cde70138c953341c851abeec6427b3221f7`.
- Final runner/binary/source hashes match their pre-run plan; no native replay occurred.

`just verify-reports-due-worker <pre-bound-plan>` names the reproducible entry. The
Python oracle tests, AST parse and diff check pass. The recipe has not been parsed by
just in this restored environment; its underlying commands ran directly. The Reports
72-test/fmt/check/Clippy result at `7ad98ef` remains applicable because this batch changes
only test scripts, a command and documents, with all Reports/shared production source
unchanged. No runtime, HTTP or load is repeated solely to accumulate another pass.

This is a real negative scheduled-run lifecycle plus signed module HTTP and persisted
process-restart evidence. It does not prove successful target execution, rendering/PDF,
Admin JWT/gateway/RBAC or browser-entry E2E. The next independent seam is Admin gateway
readback of this retained application-generated failed run under bounded user roles.

## Admin gateway and serial reader authorization

Basis: `73ae95370b39ebb2431674594494247758540b13`, plus the scoped gateway runner,
its two pure oracle tests, command and documentation. This is the first current
Reports acceptance here that crosses real Admin login/JWT, current role grants,
Admin gateway/HMAC delegation, Reports service and persisted run readback.

The input is the immediately preceding application-generated failed run, not a
manually inserted run. The original closed database is opened read-only and copied
using SQLite backup into a new owned fixture. Schedules are disabled in the copy
before either process starts; the original database hash remains unchanged. A fresh
Admin database contains one synthetic custom reader/role. The retained target is
already disabled. No new run, browser or real notification is permitted.

Pre-run revised plan: `target/rz/reports-gateway-read/plan-v2.json`, SHA-256
`d2c02156745c7d0970d8ff6c354df3dc3c5ba0dd9d4a60375c8d34556926f1f7`.
It binds runner, both binaries, relevant source/contracts, original receipt and
original database before launch. The first attempt stopped at Reports configuration
validation before any HTTP: the fixture sink path was not the required notification
endpoint. Correcting only that local path resolved it. The initial plan and zero-HTTP
failure in `run-6ok0f9nj` remain retained; the total is 17 actual HTTP attempts across
both attempts, within the overall 20-attempt budget. No product/security rule changed.

The successful `run-nnch26nt` used 17 HTTP attempts and 46 guard checks, from
2026-10-04 13:19:01 to 13:19:03 UTC:

- Owner and a reader with exactly `reports:run:view` received the exact prior failed
  run, including ID, flow, error and timestamps.
- An unsigned read returned 401. The reader's schedule read and run creation both
  returned 403; the copied database still had exactly one run and one occurrence.
- Removing run-view from the reader's role made the same JWT return 403. Restoring it
  made that same session return 200 with the identical run. Owner access remained 200.
- Steps, artifacts, outbox, observed children and unexpected notification requests
  stayed zero. Both native processes and the owned sink stopped. The source DB digest
  and all pre-bound source/runner/binary hashes remained unchanged.

Receipt: `target/rz/reports-gateway-read/run-nnch26nt/result.json`, SHA-256
`0976dbdce5cfdead01e5845dee78dddf06866776b4c3f28d0c2263bce9610a4f`.
Login responses, passwords, Authorization headers and raw JWTs are never persisted in
this receipt. The Admin binary hash is
`b57fd5121a955ccaa56d3960db601a944fb99236204f9acec62d5a1c567127f6`, matching the earlier
real gateway receipt `daily-summary-gateway/run-_fu9dk8s/result.json`; its Admin/shared
sources are unchanged from that verified baseline and its dynamic libraries are
present. Reports reuses the exact preceding due-worker binary. Neither is a release
or full frontend-build attestation.

`just verify-reports-gateway-read <plan> <source-receipt>` is the reproducible entry.
The two pure input-evidence oracle tests, Python AST parse and diff check pass; the
underlying commands ran directly because just is absent in this restored environment.
The complete scoped Rust gate from `7ad98ef` remains unaffected by this scripts/docs
batch and is reused with its exact original result. Browser rendering, successful
Reports target execution, concurrency, every other role and full browser-entry E2E
remain unverified. No broader permission or UI verdict is inferred.
