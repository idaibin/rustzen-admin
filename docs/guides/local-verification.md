# Local verification — 2026-09-04

Basis: current local checkout on `main`, workspace/Web version **0.5.0**. All
changes remain under `Unreleased`, independently of ops-suite versioning.
Verification used new task-owned databases; existing application data was not reset.

## Verified behavior

- Admin: full suite 126 passed, one explicitly ignored benchmark. The monitor-distribution composition separately passed 37 tests with the same ignored benchmark. Navigation now
  persists each Manifest page separately from its required capability, preserving
  four Monitoring pages with Nodes-owned Global settings. Inventory edits use
  `PUT /api/system/menus/inventory/{id}`; the generated Admin contract and Web
  client agree. Hiding a page does not revoke or grant API permissions.
- Monitoring: Controller 47 and Agent 12 tests passed. Real gateway scenarios cover owner/viewer access,
  accepted/duplicate/stale reports, CPU and independent disk alerts, three-sample
  recovery, node/global policy inheritance and reset, incident paging/details,
  invalid requests, and 30-day query limits. A native macOS Agent report traversed
  Admin and persisted in Monitor.
- Analytics: 26 Rust tests passed for collection/query/delegation,
  admission-bound, origin-policy and JSON error-envelope behavior; 12 tracker
  contract tests and 6 independent Linux host-gate contract tests also passed. The browser
  overview/details pass confirmed that the policy-status display is absent.
- Reports: historical local evidence recorded 26 tests for daily/weekly create,
  disable, next occurrence, permissions, and removal. The current pinned Linux
  Chromium gate additionally proves target-backed schedule lifecycle and a
  next-minute healthy-flow occurrence: it bounded-waits for the real scheduler
  to persist `enqueued` plus `runId`, follows the Templates link into that exact
  Runs audit, and expands the verified matrix to 15 success cases and 10 fault cases. Its separate `missed`
  row is a controlled SQLite fixture in the disposable verifier, used only to
  render due/reason with no run link; it is not scheduler API evidence.
- Web: 60 tests passed. Functional browser checks cover all four Monitoring
  routes, separate node mount series, node override/reset, event details, global
  save, empty summaries, and Reports template onboarding. A viewer has disabled
  settings controls and no save action. Representative geometry checks at
  1920x1080 light Chinese and 1440x900 dark English show no document overflow.
- Services: 24 startup orders, Admin-alone login, independent process termination,
  disabled/unavailable module responses, four database restores, CLI status,
  Manifest contracts, and gateway latency passed. Debug p95 gateway overhead was
  0.694 ms at concurrency 32 with 320 samples per path; this is local debug evidence.

## Reproduction and evidence

Use root `justfile`: `just check`, `just contract-verify`,
`just verify-modules-mvp`, `just verify-reports-linux`, and
`just verify-automation-browser <installed-browser-executable>`.
For the public tracker host matrix, build the pinned Linux artifacts with
`just build-admin-browser-linux`, then run
`just verify-analytics-tracker-linux`. The 2026-09-05 arm64 Colima run passed
pre-opt-in (0 to 0 rows), opt-in (0 to 2 rows), opt-out (2 to 2 rows), and real
413/429 public-route responses with zero row delta. Its source-bound manifest is
`target/rz/analytics-tracker/current/manifest.json`; Linux 507 is explicitly
`not-verified` because the gate does not exhaust storage, while the controlled
Rust route seam remains the 507 row-preservation evidence.
The September 3 multi-service run directly invoked the same verification script
after `cargo build --workspace`, with `RUSTZEN_VERIFY_BUILD_PROFILE=debug`.
Focused Web build/type checks, the full current Web Bun suite (60 tests),
Admin full and monitor-distribution suites, OpenAPI contract verification, and
Reports build/clippy/tests were repeated after the final component/browser cleanup changes.

Machine-local logs and screenshots are retained in
`/Users/daibin/Codex/outputs/rustzen-admin-20260903/`, including `check-final.log`,
`services-final.log`, `reports-browser-final.log`, and
`settings-viewer-dark-1440.png`. The initial failing navigation regression and
Reports profile-lock evidence are retained alongside the passing runs.

## Current console refinement coverage

The subsequent local in-app-browser pass covers the user-requested console refinement:

- 1705×1039 light Chinese: right-aligned filters on user, role, menu, operation-log,
  incident and analytics-detail pages; automatic input, no-match, clear/recovery and
  retained focus exercised with real local responses. User list returned three rows,
  username filtering one row, and disabled-state filtering zero rows. Role/menu filters
  each narrowed to one expected row; log no-match cleared back to six rows.
- Analytics type/path behavior includes clearing/disabling path on Other reports.
  Daily Summaries has no search input. Module-log filtering narrowed four files to one.
- Dashboard tone cards, neutral module status cards and resource panels; unified
  four-control alert settings with visible numeric-input boundaries; representative
  light/dark theme and empty-state checks.
- System Status module-log diagnostics are now browser-accepted in the current
  pinned Linux Chromium gate: the owner reads the appended current Admin marker,
  creates a one-file backup summary, previews and confirms removal of only the
  expired Monitor fixture, and preserves the service-created current-day files.
  Desktop `1440x900` dark/zh-CN and narrow `390x844` light/en-US screenshots
  are hash- and dimension-bound in the 15-success manifest.
- 390×844: user and analytics filters, settings reflow, module-log controls, bounded
  table overflow, visible empty feedback and pagination. 1024×768 role layout was
  corrected so title and filters do not overlap. At 1440×900, the Dashboard module
  panel kept all three Chinese and English names and statuses readable in a 342px
  single-column panel. The temporary viewport override was reset afterward.
- Focused source format/type checks and lint completed; lint had no errors, with
  pre-existing generated-contract/test warnings. Independent source review covered
  timer cleanup, composition suppression, page-one query state, filter preservation
  through loading/error branches and applied-filter export semantics.

This is representative local UI evidence, not a full API or release acceptance pass.
Exact timer/IME event sequences, populated page-two API traces, forced slow/error
responses for every filter, all permission combinations and every locale/theme matrix
were not exhaustively browser-tested. Shared and feature requirements are recorded in
[DESIGN](../../DESIGN.md), the [UI index](../ui/index.md), and
[product requirements](../product/product.md#console-interaction-requirements).

## Finite remaining delivery checklist — 2026-09-07

This is the execution list for the current UI, code, Monitoring, Automation, and
Reports completion goal. A closed row requires its named evidence and an
independent P1/P2 review. New observations that do not block these acceptance
conditions are recorded as later recommendations rather than added to this list.

| Order | Required closure and source | Current state and remaining work | Execution owner | Required evidence | Completion condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Analytics Overview/Details state matrix — [Analytics Product](../product/features/analytics-collection-safety/spec.md) and [UI contract](../ui/features/analytics-collection-safety.md) | Commit `960c506` contains the reviewed implementation and ten-case gate. The final immutable runtime evidence still needs binding to the committed gate/build inputs. | Primary agent owns the single final Colima run and acceptance; implementation and independent review are complete. | Focused API/UI tests, TypeScript and Web build; source-bound `linux/arm64` manifest with ten successful run receipts, exact fixture request sequence, and the two named screenshots; clean independent review. | Published evidence identifies the committed runtime inputs and passes exact manifest, receipt, screenshot, process-cleanup, and publication checks. Documentation-only status changes do not invalidate those runtime inputs. |
| 2 | Monitoring console acceptance — [Monitoring Product](../product/features/monitoring/spec.md), [UI contract](../ui/features/monitoring.md), and [test matrix](./monitoring-testing.md) | Existing Controller/Agent/runtime scenarios pass. Close only the named gaps: Incidents/Summaries paging and filtering, populated daily summary, and loading/permission/error/background-refresh behavior on the four Monitoring routes. | One `gpt-5.6-terra high` implementation owner; primary agent owns expensive runs and acceptance; one `gpt-5.6-sol high` reviewer. | Route-exact API receipts and a Linux Chromium manifest for the required states and representative desktop/mobile theme-locale pairs; focused Monitor/Web tests; independent review. | Every named state has a successful receipt, no stale protected data remains after 403, and no horizontal overflow or broken recovery action is present. |
| 3 | Automation and Reports acceptance — [Scheduled Report Product](../product/features/scheduled-report-automation/spec.md) and [UI contract](../ui/features/scheduled-report-automation.md) | Scheduler lifecycle, SR-UI-002 form behavior, real `enqueued` linkage, and controlled `missed` rendering pass. Close the declared processing, partial, and runtime-failure presentation paths; do not expand Reports into a workflow system. | One `gpt-5.6-terra high` implementation owner; primary agent owns expensive runs and acceptance; one `gpt-5.6-sol high` reviewer. | Real Reports run/step/occurrence receipts, Linux Chromium captures for the missing states, service tests, Web build/typecheck, and independent review. | Templates/Runs show correct processing, partial, failure, retry, permission, and retained-evidence behavior for manager and view-only roles. |
| 4 | Four-service module-log runtime — [Module Log Product](../product/features/module-log-diagnostics/spec.md) and [UI contract](../ui/features/module-log-diagnostics.md) | Admin browser lifecycle and service/client archive checks pass with fixtures. Exercise actual Admin, Monitor, Insights, and Reports log prefixes and service-account permissions together. | One `gpt-5.6-sol high` cross-layer implementation owner; primary agent owns the Colima run and acceptance; one `gpt-5.6-sol high` reviewer. | Fresh Colima four-service run using service-created logs; Admin list/tail/backup/cleanup receipts; archive hash verification; permission/ownership checks; independent review. | All four real service prefixes are readable only through the allowed Admin boundary, backup bytes match the manifest, and cleanup preserves current-day logs. |
| 5 | Message center and SSE — [Composable Distribution Product](../product/features/composable-distribution/spec.md), [architecture](../product/features/composable-distribution/architecture.md), and [implementation plan](../product/features/composable-distribution/implementation.md) P5-P7 | This is an authorized target and remains queued: optional Admin inbox, Monitor/Reports outbox relay, and direct Admin fetch-SSE are not implemented. Keep SSE as advisory invalidation over the durable inbox and add no fifth resident service. | One `gpt-5.6-sol high` cross-layer implementation owner; primary agent owns acceptance; one `gpt-5.6-sol high` reviewer. | Fresh-schema transaction/concurrency/retention tests, outbox crash/dedupe/recipient tests, stream/parser/cancellation tests, real browser reconnect/reconciliation, alert/run links, and absence proof when notifications are unselected. | Selected notifications provide durable authorized inbox state and reconnect reconciliation; `monitor` has no notification table, relay, queue, endpoint, timer, or Web chunk; no email, SMS, webhook, or generic workflow is added. |
| 6 | Full and pruned distribution integration — [Composable Distribution Product](../product/features/composable-distribution/spec.md) and [implementation plan](../product/features/composable-distribution/implementation.md) P8 | Monitor server P1-P4 is verified. Complete and certify the finite catalog selections: `full`, `monitor`, `monitor-notify`, `analytics`, `reports`, validated `custom`, and the separate `node-agent` artifact. | One `gpt-5.6-sol high` cross-layer implementation owner; primary agent owns expensive runs and acceptance; one `gpt-5.6-sol high` reviewer. | For each shipped selection, bind resolver/build/package inventories, signed exact-member archives, forbidden-route/schema/file absence checks, fresh-root install/activation or Agent pairing/runtime evidence, current browser journeys, and measured artifact/load results to one immutable build; run full-workspace and Web regression plus independent review. | `full` contains every enumerated target capability including notifications; every pruned selection contains only its declared capabilities; node Agent and server artifacts remain separate; artifacts, routes, schemas, services, Web chunks, installation, activation, and runtime match their signed plan. If a required physical target is unavailable, this row remains open with that environment blocker recorded. |
| 7 | External target-environment acceptance — [Analytics Product](../product/features/analytics-collection-safety/spec.md), [Monitoring test matrix](./monitoring-testing.md), [Scheduled Report Product](../product/features/scheduled-report-automation/spec.md), and [deployment guide](./deployment.md) | Local/Colima behavior is evidence only. Production-host tracker consent/request/persistence, deployed Monitoring and Reports timers, native systemd/browser seccomp, and physical Linux/Windows Agent collection remain unverified. | Primary agent prepares exact procedures and evidence mapping; the environment owner must authorize and provide each external target before execution. | On the real target release, record tracker opt-in/opt-out and persisted pathname events, Monitoring offline/summary/retention timer outcomes, Reports due occurrence/run linkage, native service ownership/readiness, confined browser execution, and physical Agent reports; bind all receipts to the deployed build/composition. | Every named external path passes on its target. If an environment or deployment authorization is unavailable, this row stays open with the exact blocker; Colima cannot close it. |
| 8 | Shared visual approval — root [DESIGN](../../DESIGN.md) authority and [UI index](../ui/index.md) | `DESIGN.md` remains a candidate. Its current SHA-256 is `fe5ffa524b2cb7a7fd618a2d7b583626119bd035c383354bc88cc9f6dfccee89`; no matching named approval record exists. | A named non-implementer owns design approval; the primary agent prepares the final route/viewport evidence but cannot self-approve it. | Final human visual review of the required UI routes and representative viewports from rows 1-6, including the `full` and each shipped pruned Web shell, plus a named approval record bound to the exact `DESIGN.md` SHA-256 and corresponding build/selection browser manifests. | The reviewer accepts the final visual result and exact hash. Any later byte change to `DESIGN.md` reopens this row; lint, build, automated screenshots, browser runtime, and ordinary code review do not close it. |

Production deployment, a native systemd boot, native-host browser seccomp, and
physical Windows/Linux Agent hosts require their respective external
environments. They remain explicitly `Not verified` and do not become locally
verified through Colima evidence. Full Rust test runs use one harness thread
because Admin fixtures share a global permission cache; internal race tests
still create parallel tasks.

## Nodes onboarding and configuration verification

- Nodes now owns Add node and Global settings drawers; the independent settings route and menu are removed. The live Monitor manifest exposes only overview, nodes, incidents and summaries.
- Built-in Browser verified required-node-ID validation, generated
  `rz-monitor-agent` setup command, waiting before an Agent report, and connected
  after a real local Agent reported as `local-dev-node`. Task-owned verification
  services were stopped after the checks.
- Global settings loaded and saved all four defaults together; the saved update time was read back. Values remain 90%, 90%, 90% and 90 seconds.
- At 390px viewport width, the drawer was corrected from 736px overflow to exactly 390px, then visually inspected. Desktop viewport restored afterward.
- Eight focused frontend checks, TypeScript and scoped lint passed. Remote-host installation, Windows setup, and complete role/browser error matrices were not exercised.
