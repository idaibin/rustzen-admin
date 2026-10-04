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
