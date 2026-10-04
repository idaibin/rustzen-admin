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
