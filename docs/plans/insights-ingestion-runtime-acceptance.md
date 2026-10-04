# Insights ingestion runtime acceptance

## Accepted scope and pre-run plan

Authority: `docs/product/product.md` and
`docs/product/features/analytics-collection-safety/spec.md`. This bounded workflow
verifies the existing installation policy, origin/CORS, batch validation and retained
query contracts. It does not decide a new collection policy or change product behavior.
The public project routing key is a synthetic fixture identifier, not a credential.

Source base: `70238b67a0550f3c6288aa7bd0b9a1f348a42857`. The application Rust code,
initialization SQL, existing `scripts/verify-insights-scenarios.mjs` and Web code remain
unchanged. New code owns only runtime acceptance orchestration and negative assertions.

The candidate `tests` guidance at
`idaibin/skills@23cc6b0a30abf15dd86cbb6cd148d13c731cca97` directed the explicit
contract/case/oracle/evidence map. Before execution the project-local plan was saved at
`target/rz/insights-ingestion-runtime/plan.json`, SHA-256
`ec9e3413d2b7b7a97600ed99fe313e0b476ea3b0dda579b396ee37ae701255ec`.
It fixes a single execution, at most 20 real HTTP requests including login/readiness,
three intentionally accepted events, and no external-domain requests.

## Reproduction and ownership

Inspect and run `just verify-insights-ingestion-runtime <current-plan-path>`.
The target builds the real Web distribution and full Admin/Insights binaries, starts
only these two owned services with fresh databases, and reuses the existing scenario
function through a real Admin gateway adapter. Startup log observation consumes no HTTP
requests. Gateway discovery may be retried at most three times, within the same total
20-request budget. No event rows are inserted by SQL.

Only loopback addresses receive network calls. `https://app.example` and the denied
origin are literal synthetic `Origin` headers used to test the policy; neither site is
contacted. Login/session values stay in memory and are excluded from receipts. SQLite
inspection uses a read-only connection. Output databases/logs are retained under unique
`target/rz/insights-ingestion-runtime/run-*` directories; both owned processes are
stopped even when an assertion fails. The storage guard and its limits are unchanged.

## Acceptance-to-evidence map

| Requirement / boundary | Observed oracle | State |
| --- | --- | --- |
| Installation policy through real owner JWT and gateway | Disabled ingestion 403; enable operation succeeds | Passed |
| CORS HTTP contract | Normalized allowed preflight 204 and bounded allow headers; denied preflight/origin 403 without allow headers | Passed |
| Real public ingestion → Insights SQLite | Three known events accepted by HTTP; exactly three database rows | Passed |
| Retained overview/details | PV 1, UV 2, event count 3, API request count 1, error count 1, event-duration p95 42; detail count 3 and safe pathname fields | Passed |
| Validation | Unknown event 422; allowed-origin business error retains correct CORS response | Passed |
| All-or-none mixed batch | A valid event plus an invalid event returns 422; database count before and after remains 3 | Passed |
| Batch hard limit | 51-event batch returns 413; before/after count remains 3 | Passed |
| Body hard limit | Serialized body above 64 KiB returns 413; before/after count remains 3 | Passed |
| Query/delegation authorization | Unauthenticated Admin query and unsigned direct module query each return 401 | Passed |
| Tracker asset | Real gateway returns JavaScript content type | Passed, asset delivery only |
| Browser opt-in/bootstrap/consent | Actual browser initialization and consent behavior | Not run; HTTP origin/key validation cannot prove consent |
| Frontend page / function / browser-entry E2E | Current rendered Analytics pages and interactions | Blocked by the previously observed cloud-browser local-preview restriction |
| Other roles, concurrent users, rate exhaustion, performance or production | Outside this finite ingestion scenario | Not run |

The overview's 42 ms p95 is the expected aggregate of submitted synthetic event data,
not a measurement or claim about server request latency.

## Recorded execution and regressions

One runtime execution passed on 2026-10-04 with **18 total real HTTP requests**, including
login/readiness and the final two unauthorized reads. Receipt:
`target/rz/insights-ingestion-runtime/run-rq_hk908/result.json`.
It embeds the immutable plan/digest, runner and existing-scenario hashes, both native
binary hashes, Node version, timestamps, relevant response bodies/CORS headers,
per-rejected-batch SQLite before/after counts, and successful process cleanup.
The plan bytes/digest were captured before launch. Runner/scenario and binary digests
were recorded after the processes stopped, from the unchanged local files; they were
not separately cryptographically frozen before launch. This is local reproducible
source/build evidence, not tamper-resistant execution attestation or signed-release
provenance. The preceding build log is retained at the task-local evidence location.

- Rust 1.95.0, Linux x86_64, default-feature `rz-insights` build passed.
- The focused Insights binary test suite passed: 28 tests, zero failures, one thread.
- Existing scenario tests passed under Bun 1.3.14: two tests, six assertions. These are
  mocked scenario-oracle checks, separately identified from the real HTTP run.
- Python compilation, JavaScript syntax and `git diff --check` passed.

Independent review must inspect the fixed receipt and may run the focused mock/unit
checks, but must not silently repeat this single-execution HTTP budget. API-entry to
real gateway/module/storage is verified within the named cases; browser-entry E2E,
all modules, production deployment and source-to-signed-artifact provenance are not.

Next Ready possibilities are separate plans: authenticated viewer/manager isolation for
Insights, or tracker consent/bootstrap tests using an approved reachable browser. The
blocked browser row remains open and is not replaced by more HTTP acceptance.
