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
