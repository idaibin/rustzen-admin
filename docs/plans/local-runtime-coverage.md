# Scoped local coverage, 2026-10-04

## Verdict and fixed basis

**2026-10-04 snapshot: executed checks passed within the rows below; overall acceptance incomplete.**

For the subsequent 2026-10-05 current-browser and source-gate update, see
[local verification](../guides/local-verification.md#2026-10-05-local-continuation).
The historical blocked rows below describe that earlier environment. Work later
reported a distinct local verification run; the later coordinating review and this
Cloud integrator's source-hash checks are qualified below. In the earlier snapshot, real page
rendering, browser interactions and browser-entry E2E were blocked by the cloud
browser's `net::ERR_BLOCKED_BY_CLIENT` refusal of the owned loopback preview.
Source seams, VM tests, build success and real API journeys cannot close those rows.
No aggregate test target or successful exit implies complete product acceptance.

Delivered execution basis for this historical snapshot: `8f80902417a39e2da19ca4deee498c25ddf28de3`.
This follow-on documentation update records dependency preflight only, without
changing or replaying the accepted Reports application/runtime evidence.
The previous fixed-calendar HTTP smoke retains its original executable receipt. The separately reviewed Reports weekly-DST
calendar correction is included in that source basis. The
Monitor, Insights and Web application sources remain unchanged from the restored
`b94909de11c4e09a50815eceec123dbe8c00a6f6` baseline. Reports now has one bounded
forward-calendar change and two regressions. Its previous full UTC HTTP receipt retains
its earlier source identity; a separate eight-request current-date IANA metadata smoke
now binds the fixed native binary without promoting the old full gate. Other unaffected rows retain their own original source/harness receipts and
explicit boundary limits; original identities are never rewritten as new runs.

Product/UI authorities remain `docs/product/product.md`, the Monitoring and Analytics
collection-safety specifications, `docs/ui/index.md`, and root `DESIGN.md`. The candidate
`tests` skill at `idaibin/skills@23cc6b0a30abf15dd86cbb6cd148d13c731cca97` structured
case/evidence separation; it does not replace those product contracts or grant release
approval. This document is an evidence matrix, not a second task ledger.

## Completed-work delivery summary, 2026-10-05

The palette reconciliation recorded a remote readback of `feat/theme-palette-20261003` at
`ac8ce7321fdfff73e5e534f390800c4a7475cda6`, including all completed batches below.
This reconciliation fixes stale pre-publication ledger statuses; it does not repeat
implementation, tests, browser work or previously delivered commits.

- Theme/status semantics and keyboard diagnostics were delivered in
  [`42d6cb1`](https://github.com/idaibin/rustzen-admin/commit/42d6cb104a68a484685f6e6d9372b1d8e8af9b97).
  The [theme ledger](./theme-palette-tasks.md) retains scoped component-fixture evidence,
  approved design identity and the original 158-test Web checkpoint.
- Shared pointer/keyboard/focus, pending/disabled/error states and recoverable
  interactions were delivered in
  [`b94909d`](https://github.com/idaibin/rustzen-admin/commit/b94909de11c4e09a50815eceec123dbe8c00a6f6).
  The [interaction ledger](./interaction-state-tasks.md) records 167 Web tests,
  type/lint/build checks, historical 19-route/114-view rendering and separately scoped
  real API/SQLite actions. The matrix is historical resting-view evidence, not 114
  complete workflows or a fresh browser rerun in the current environment.
- Monitor completion covers real summary generation/upsert, delegated and gateway
  reads, serial role revocation, outage/recovery and bounded read observations;
  [the Monitor record](./daily-summary-runtime-acceptance.md) separates each boundary.
- Insights completion covers ingestion/CORS/rejected-batch atomicity plus tracker VM
  functions and Analytics static seams; see [the Insights record](./insights-ingestion-runtime-acceptance.md).
- Reports completion covers schedule metadata/CRUD, the weekly DST-gap fix, persistent
  due/cancel state, a real disabled-target worker failure/restart, and Admin reader
  authorization; see [the Reports record](./reports-schedule-contract-acceptance.md).

At that palette reconciliation, the original local checkout was absent in its
2026-10-05 execution workspace, so a fresh local clean-tree/untracked-artifact verdict
was unavailable. The last verified local
state on 2026-10-04 matched `ac8ce732` with only generated `apps/web/.vite-hooks/`
untracked. That reconciliation's completion/delivery claims were grounded in current immutable remote
contents and ancestry, not an assumed surviving checkout. No raw database,
authentication log or secret is published by this summary correction.

## Boundary-specific coverage

| Boundary | Concrete evidence | State / limitation |
| --- | --- | --- |
| Monitor backend domain/persistence | 76 Rust tests; actual startup worker generates summaries from raw inputs, empty-node handling, cross-day incidents, restart upsert | Passed for named cases |
| Monitor direct module API | Signed reads; unsigned 401; wrong capability 403 | Passed |
| Monitor Admin gateway/storage | Real owner login/JWT/HMAC, 23 generated summaries, 20/3/20 paging, SQLite agreement | Passed |
| Monitor serial non-owner authorization | Two owned custom roles, read allow/deny, least privilege, same-session revoke/restore, disabled-user rejection and owner isolation | Passed; not all roles or concurrent mutations |
| Monitor failure/recovery API | One owned worker outage; 503/null error, Admin login independence, persisted navigation retained, same-JWT/same-DB recovery | Passed; navigation API is not rendered navigation |
| Monitor limited read observation | Two separately recorded 100-GET runs, 200 measured GETs total; final sample correct with max in-flight 1 | Read correctness passed; no overlap, improvement, capacity or SLO claim from this row |
| Monitor overlapping-client reads | One separate four-GET cohort; all responses match, four actual client HTTP intervals overlap | Passed only for that bounded client-overlap scenario; server critical sections and multi-user writes unverified |
| Insights backend/API/storage | 28 Rust tests; one real 18-HTTP run via Admin, CORS policy, three accepted events, overview/details, auth rejection and all-or-none invalid/body/batch limits | Passed for named cases |
| Insights scenario harness | Two Bun tests of the existing scenario oracle | Passed with mocked transport; not additional real HTTP |
| Tracker client functions | Twelve VM tests of actual tracker JavaScript, including consent state, opt-out, identifiers, pathname sanitization, limits, queueing and bounded retry | Passed in fake DOM/storage/network/timers; no browser-host consent proof |
| Analytics page source seams | Four source-string/order tests for locale, responsive class declarations, 403 precedence and route-local errors | Passed static contracts only; no React render or interaction proof |
| Reports schedule module API | One future-only 22-request signed-module run: daily/weekly CRUD, UTC/next-due metadata, capability/input rejection and deletion; zero runs/occurrences/artifacts/outbox | Earlier full UTC pass only; no full 22-case replay on fixed binary; not Admin JWT/RBAC or due execution |
| Reports fixed-calendar module metadata | One current-date, eight-request America/New_York fixture smoke on the corrected binary; future due values round-trip to local time/weekday; all execution/delivery guards stay zero | Passed for that native metadata boundary; not a real spring transition or due job |
| Reports Admin gateway and serial reader authorization | Seventeen real HTTP attempts: owner/custom run-only reader, exact prior application-generated failure readback, unsigned401, schedule/create403, same-JWT revoke403/restore200, owner unaffected | Passed for one custom reader and serial role changes; no all-roles/concurrency or rendered UI claim |
| Reports native due-worker failure lifecycle | One real timer-generated occurrence/run, disabled-target failure, eight signed module HTTP reads/writes, one OS process restart plus 16-second repeat observation, same persisted run/occurrence | Passed negative path only; no browser steps/artifacts/notification delivery or successful render |
| Reports scheduler/service/SQLite integration | Two new deterministic production-poll cases: due +60s, missed +61s, file-pool reopen/recovery, repeat enqueue/skip, cancelled linkage, disabled/effectiveAt boundaries | Passed with real fresh SQLite and injected clock; no process restart, timer worker or browser execution |
| Reports pure scheduler/input functions | Six calendar and two input-validation Rust tests, including red-to-green weekly DST horizon regression | Passed as unit functions; no real timer/browser proof |
| Current real frontend page rendering | No successful browser connection to current owned preview | Blocked |
| Current real frontend user interactions | No successful browser connection to current owned preview | Blocked |
| Full browser-entry E2E | No current browser → gateway → persistence run | Blocked |
| Signed bundle/install/systemd/production | Not part of these local acceptance runs | Not run; previous release snapshot is not promoted |

The [Reports future-schedule record](./reports-schedule-contract-acceptance.md) adds its
pre-bound executable/runner/plan and zero-execution guards.
The separate [Monitor record](./daily-summary-runtime-acceptance.md) and
[Insights record](./insights-ingestion-runtime-acceptance.md) retain exact scenario
boundaries, commands, plan hashes, receipt paths and publication checkpoints. The
historical 114-view matrix in `interaction-state-tasks.md` is not fresh evidence for
this restored workspace and is not counted as current browser acceptance.

## Pre-hashed tracker and static-seam execution

Command, with Bun 1.3.14:

```text
bun test apps/insights/src/features/tracking/tracker.test.mjs apps/web/tests/analytics-ui.seam.test.mjs
```

This one execution passed **16 tests / 178 assertions**, split into the twelve VM
function cases and four static page seams above. No HTTP calls or browser operations
were made. The suites were inspected to confirm their simulated boundaries first.

Before running, the plan bound the exact Bun executable, both test files, tracker
source, Analytics route/API sources and product/UI authorities. All those hashes were
checked again after execution and remained identical.

- Plan: `target/rz/insights-frontend-functions/plan.json`;
  SHA-256 `c70946ed5a4a9f9492d9a4b228aa6c39034ae445a852f2ae922a513e07d91819`.
- Bun executable SHA-256:
  `8e3eb1a8566d4dca90ee5b4a290349b6dae278c120a538bedeaf38d929f0c0b5`.
- Receipt and full log: `target/rz/insights-frontend-functions/result.json`, `test.log`.
- Execution: 2026-10-04 09:02:34 UTC, exit code 0. The receipt retains precise
  timestamps, pre/post source and executable hashes and output digest.

Reusable commands deliberately identify evidence layers:
`just verify-insights-tracker-vm` and `just verify-analytics-ui-source-seams`.
Neither is named or presented as complete frontend/E2E acceptance. An actual rendered
browser run must be planned separately once a supported reachable preview exists.

## Remaining meaningful work and prerequisites

- Reports weekly-DST P2 is corrected at source and pure-regression boundaries. A new
  eight-request smoke verifies current-date metadata on the fixed native binary; actual
  timer/worker failure now passes a separate single native run and process restart
  against an owned disabled target. Successful target/browser execution remains
  unverified. Two direct production-poll integration cases separately cover due
  decisions and service readback after reopening SQLite. The earlier full UTC receipt stays tied to its
  retained pre-fix executable.

- Browser-dependent work is genuinely blocked: current Monitor/Analytics page rendering,
  interaction, real consent/bootstrap and complete browser-entry journeys need an
  approved, reachable preview. Do not change network/security settings or switch
  browser-control mechanisms to evade the recorded refusal.
- Reports future-schedule metadata/CRUD and selected input rejection are now covered
  at the signed module API, followed by one real due failure and a separate real
  Admin gateway/custom-reader authorization read journey. Successful target/render
  execution remains unverified; browser work must not evade the recorded refusal.
- Real Agent transport, other roles/modules, concurrent writes, production load and
  release provenance remain separate scenarios. Do not infer their completion from
  these receipts, and do not repeat passing loads just to accumulate test counts.

All owned services and the failed agent-created preview tab were closed. Local fixture
artifacts remain intentionally available for review; generated Web tooling hooks are
untracked local installation artifacts, not part of these commits.


## Pure regression entries and official command parsing

Two additional entries keep low-side-effect checks separate from runtime gates:

- `just verify-reports-calendar-unit`: the six calendar and two input-validation Rust
  functions, using the same bounded build profile as the accepted source fix.
- `just verify-local-acceptance-harness-unit`: the ten existing Python fixture,
  fail-closed-budget and interval-oracle checks. No native service or real HTTP load.

Together with `verify-insights-tracker-vm` and `verify-analytics-ui-source-seams`, these
names state their evidence layer. There is deliberately no “everything accepted”
aggregate that would hide blocked browser rows. The underlying suites already ran as
recorded above; the parser checks below are not another execution of those suites.

Official `just` 1.58.0 was built from crates.io in an isolated workspace tool directory.
The locked install completed with an upstream yanked-dependency warning for
`chacha20 0.10.1`; a warning-free tool installation is not claimed. No repository package
or lockfile was changed for this tool install. The command still requires its documented
Rust/Bun/Python tools on PATH when recipes are actually executed.

Ten preplanned `--version`, `--list`, `--show` and `--dry-run` checks passed for the four
pure/VM/static recipe names. None executed a recipe. The official parser therefore
resolves the earlier “just unavailable” syntax-check gap, without changing UI/E2E status.

- Pre-run plan: `target/rz/local-pure-gates/plan.json`, SHA-256
  `8358019cbe408015df1e9210c30158c8bc2e074f387020290f0235d8d1fdf90e`.
- Receipt/logs: `target/rz/local-pure-gates/result.json` and `command-*.log`.
- The plan freezes the just binary and justfile hashes before parser execution and
  verifies both afterward. `recipesExecuted` is explicitly false in the receipt.
- The exact DST-fix commit's accessible GitHub status and PR-workflow queries returned
  no checks/runs. This is not a required-CI pass or a ready-to-merge claim. Main and
  production deployment were not modified.


## Reports integration follow-on and Ready frontier

The current test-only slice follows `tests` 0.1.1 at
`idaibin/skills@741f4161c863219528d2596ce03fdeb7ce7f1f1a`. Its pre-execution plan,
failed fixture attempt, correction and current results are retained in the
[Reports record](./reports-schedule-contract-acceptance.md). The complete applicable
Reports binary suite passes 72/72; this includes the two new direct-poll cases.
No production behavior, shared crate or frontend source changes in this slice.
The existing Monitor/Insights/VM/static receipts therefore remain unaffected and keep
their original source identities. The repository-wide `just check` and required remote
CI are not promoted from the scoped Reports result.

The bounded native due-worker failure journey now passes in its own source-bound
receipt: one actual due occurrence, exact failed run/error, eight HTTP requests and
same-database process-restart persistence, with zero browser/target/delivery effects.
See the Reports record for the 919 sampled guards and fixed executable identity.
Reports Admin gateway/custom-reader readback of that retained application-generated
run now passes its own 17-HTTP receipt, including same-session revocation/restoration.
The next independent backend candidate, real Monitor Agent transport, reached its
dependency preflight and is currently blocked as detailed below. Successful Reports
browser execution and real
rendered frontend/E2E remain blocked by the preview refusal; those dependencies do
not block independent gateway/API acceptance. This acceptance batch does not alter
Reports production sources, so the complete 72-test scoped Rust gate from `7ad98ef`
is reused with its original evidence; the new Python oracles and actual worker run
provide the affected-harness verification.


## Historical prerequisite frontier after the three Reports milestones

Delivered milestones on `feat/theme-palette-20261003`:

- `7ad98ef`: direct production-poll/persistent-SQLite lifecycle, focused 2 plus full
  Reports 72, scoped formatting/check/Clippy and independent review.
- `73ae953`: one real due-worker failure, eight module HTTP attempts, one real process
  restart and zero browser/target/notification effects, independently reviewed.
- `8f809024`: seventeen real Admin gateway/RBAC HTTP attempts using the previous
  application-generated run, independently reviewed. The setup-only failed attempt
  used zero HTTP and remains visible.

The next authorized single-Agent development-mode transport scenario was selected
with a budget of two genuine samples at the fixed 30-second cadence, at most 90 seconds
and six auxiliary HTTP requests. Only the build-preflight plan was frozen; no runtime
plan was finalized. No Agent execution or transport request occurred: the Agent
binary was absent, and its build stopped during dependency acquisition. Offline Cargo
resolution identifies the missing package as `sysinfo v0.39.6`; the download attempt
reported `CONNECT tunnel failed, response 403`. The configured registry download base
is `https://static.crates.io/crates`; the retained error does not expose the complete
blocked request URL. No alternate registry/mirror or network/security workaround was
attempted. Build plan and blocked receipt are in `target/rz/monitor-agent-transport/`.
The build session is no longer available to the status tool; its terminal exit status
was not returned after tool cancellation, so a successful build/clean exit is not claimed.

The official isolated Rust 1.95.0 toolchain, rustfmt and Clippy were restored and used
successfully for the Reports gate. The earlier shared tool directory is absent in this
execution environment; Bun and just are not currently on PATH. Existing historical
parser/VM receipts remain scoped evidence, not a claim that those tools are installed
now. Reports recipes were executed through their recorded underlying commands.

Remaining next actions and conditions:

- Single-Agent transport: restore the denied official dependency through an approved
  available route, then bind a built Agent executable and a new finite runtime plan.
  Do not turn the unexecuted plan into passed transport coverage.
- The repository's full dual-Agent gate also requires Docker and OS identity/ownership
  setup unavailable here. A development-mode single-Agent run would not replace
  dual identities, production TLS, installed activation or system-service readiness.
- Rustzen CLI is a separate prerequisite candidate: no compiled `rz` exists here and
  offline dependency resolution first reports missing `anstream v1.0.0`. No CLI
  download was attempted; this is an unverified prerequisite, not a proven network
  denial or a claim that the CLI can never run.
- Whole-Web pure/source tests could be assessed independently after verifying their
  current Bun/dependency prerequisites. They have not been newly executed in these
  Reports batches and would not resolve real browser rendering/E2E.
- Current real UI/E2E and successful Reports rendering remain blocked by the recorded
  browser preview refusal. No alternative browser route was used to evade it.

No repeat of passing API/load scenarios was started to fill those gaps. The latest
exact-head accessible GitHub combined-status and PR-workflow queries returned empty
results; required CI is unverified. Main remained `edd06daf34bf106064fee7241ff70600a8bf236b`.

## 2026-10-05 populated development continuation

Work reports the historical Agent dependency blocker resolved for its local run:
locked Agent build passed there. Its IN-11–IN-13 report in
`interaction-state-tasks.md` records a pass with the actual four-service
runtime, real Agent two-sample transport (30.013 seconds, five auxiliary requests),
real browser Tracker consent/opt-out and successful native Reports render plus Web
artifact download. Eight light/dark captures and the final receipt are documented
in `../guides/local-verification.md`. Earlier failed harness attempts are preserved.
That report supersedes the old blocked status only within the Work-owned
single-Agent development slice. It neither grants this Cloud task a new execution
route nor supersedes release, PID1, multi-user or production evidence requirements.

## Cloud integration boundary, 2026-10-05

Fresh fetch for branch consolidation found palette tip
`ff0a162a08029f91b5e694416dcd333e9429af4d` and Work verification tip
`39ac997d206ccbee9d00d6c2dbb52f047bb91217`, diverged by one and five commits from
`ac8ce7321fdfff73e5e534f390800c4a7475cda6`. The unified development branch is
`feat/theme-verification-20261005`; the original palette branch and its delivery
history are retained. Main is unchanged at
`edd06daf34bf106064fee7241ff70600a8bf236b`.

The source and Work's committed execution report in
[local verification](../guides/local-verification.md) are available. The separate
profile and populated-run raw result JSON, logs and screenshot bundles have not
been obtained by this Cloud integrator. The report names these receipt hashes:

- Profile: `abe31f91b9324e841b41ba9f35b9b81c073db1a4b420923497c5682f5d09a421`.
- Populated: `3bdc7e9cbd203c7ed62d8f8a4d14059b928a688a233c16153deb4eeebb41fab7`,
  at `2026-10-05T02-48-14.834Z/result.json`.

The coordinating reviewer reports independent checks of the populated ZIP, its
result hash and all eight screenshot byte hashes. The reported ZIP SHA-256 is
`ac220a926d50cb15c6cfc6eecb1ce0f1b9e00c3a93e6082a5b38442bfd7f43be`.
This is attributed handoff evidence: this Cloud task's Library transfer returned
`download failed` without a more specific cause, so archive/result/screenshot hashes
were not independently recomputed in this environment. The separate profile bundle
has not been verified here.

The populated delivery record reports `runtimeWithUncommittedUIAndRunnerHashes=true`
and `sourceBase=2de058ab190875fe4e1f2d4df3e75da93f445902`. Its `remoteCommit=39ac997`
alone does not establish execution on all final-commit bytes. The coordinating
reviewer compared the receipt's runner/UI hashes against that commit, and this
Cloud integrator independently recomputed the same two Git-blob SHA-256 values:

| File at `39ac997d206ccbee9d00d6c2dbb52f047bb91217` | SHA-256 matching the reviewer's receipt readback |
| --- | --- |
| `scripts/verify-populated-browser.mjs` | `34a6f3b9c8105f44333b642b670676ae94d4c70ae715fe76afec5f0998239d90` |
| `apps/web/src/routes/monitoring/-node-details.tsx` | `efb34d57096ca46ff2d7f3af1eefddff6f5588d07065ddb3586fd94ee84753d9` |

The `2de058a..39ac997` delta contains only these two source/runner files and three
documents. This binds the reported populated runtime slice to those final files;
it is not a new run, whole-release provenance or acceptance of every final-commit
behavior. This documentation integration leaves all Work source/runner bytes intact.

The coordinating reviewer's pixel inspection of eight historical PNGs confirms
light/dark node history, CPU/memory and no-disk messaging, Analytics values
1 PV / 1 UV / 3 events / 1 request, and Reports four-step success with fixture preview.
The Reports artifact area is outside the screenshot viewport; images alone do not
prove every artifact row or the download action. Download consistency is supported
by the reviewer's receipt comparison of three identical PNG hashes, not by those
screenshots. These are attributed read-only checks, not a claim this Cloud integrator
opened the images or independently replayed the journey.

The earlier cloud refusal does not prove Work did not execute these checks;
conversely, neither Work's report nor subsequent evidence review authorizes this
Cloud task to bypass `ERR_BLOCKED_BY_CLIENT` or Cargo
`CONNECT tunnel failed, response 403`. No alternate browser, network route or
dependency download was attempted here.

This integration validates Git ancestry, scoped documentation, the two source hashes
and preservation of the Work source tree; the raw-artifact review is attributed above. It runs no application, build, test, migration or service,
and does not promote Work's self-review into independent runtime acceptance.
Receiving the original evidence permits read-only identity checks; it does not
implicitly permit a rerun. Work owner activity and unpushed changes remain unknown,
so new execution must also avoid overlapping that owner's work.


## Single-branch owned runtime continuation, 2026-10-05

This workspace now has direct execution evidence for IN-14 and IN-15, recorded in
[local verification](../guides/local-verification.md#populated-responsive-continuation-2026-10-05).
It supplements, rather than rewrites, the historical Cloud restrictions above.
The final owned run passed the populated 1920/1440/390 light/dark matrix, Reports
artifact containment, read-only actual downloads, denied writes and permission
revocation. Thirty screenshots and a hash-bound receipt support this finite scope.

Main/release, selected-build coverage, disk-capacity measurements, external portals,
all-role/concurrency and production remain separate. The next finite UI goal is
keyboard focus and visible loading/error recovery across Reports audit/download
interactions in both themes. Continue on the same branch with an owned fixture.
