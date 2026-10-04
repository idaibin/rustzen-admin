# Scoped local coverage, 2026-10-04

## Verdict and fixed basis

**Executed checks: passed within the rows below. Overall acceptance: incomplete.**
Real page rendering, browser interactions and browser-entry E2E remain blocked by the
cloud browser's `net::ERR_BLOCKED_BY_CLIENT` refusal of the owned loopback preview.
Source seams, VM tests, build success and real API journeys cannot close those rows.
No aggregate test target or successful exit implies complete product acceptance.

Latest execution basis: `d734d4e71518095006bc1fcdd469ef18c47657c0`, plus the
separately reviewed Reports weekly-DST calendar correction and documentation. The
Monitor, Insights and Web application sources remain unchanged from the restored
`b94909de11c4e09a50815eceec123dbe8c00a6f6` baseline. Reports now has one bounded
forward-calendar change and two regressions; its previous UTC HTTP receipt is retained
as earlier-source evidence and is not promoted to a native runtime claim for the fixed
binary. Other unaffected rows retain their own original source/harness receipts and
explicit boundary limits; original identities are never rewritten as new runs.

Product/UI authorities remain `docs/product/product.md`, the Monitoring and Analytics
collection-safety specifications, `docs/ui/index.md`, and root `DESIGN.md`. The candidate
`tests` skill at `idaibin/skills@23cc6b0a30abf15dd86cbb6cd148d13c731cca97` structured
case/evidence separation; it does not replace those product contracts or grant release
approval. This document is an evidence matrix, not a second task ledger.

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
| Reports schedule module API | One future-only 22-request signed-module run: daily/weekly CRUD, UTC/next-due metadata, capability/input rejection and deletion; zero runs/occurrences/artifacts/outbox | Earlier UTC-only pass; native fixed-calendar HTTP not rerun; not Admin JWT/RBAC or due execution |
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

- Reports weekly-DST P2 is corrected at source and pure-regression boundaries. Current
  fixed-calendar native HTTP and due-time execution have not been rerun; the earlier
  UTC-only receipt remains explicitly tied to its retained pre-fix executable.

- Browser-dependent work is genuinely blocked: current Monitor/Analytics page rendering,
  interaction, real consent/bootstrap and complete browser-entry journeys need an
  approved, reachable preview. Do not change network/security settings or switch
  browser-control mechanisms to evade the recorded refusal.
- Reports future-schedule metadata/CRUD and selected input rejection are now covered
  at the signed module API. Admin gateway/user-role integration and an actual due
  occurrence remain separate plans; due browser execution must not be used to evade
  the blocked preview or silently enable external delivery.
- Real Agent transport, other roles/modules, concurrent writes, production load and
  release provenance remain separate scenarios. Do not infer their completion from
  these receipts, and do not repeat passing loads just to accumulate test counts.

All owned services and the failed agent-created preview tab were closed. Local fixture
artifacts remain intentionally available for review; generated Web tooling hooks are
untracked local installation artifacts, not part of these commits.
