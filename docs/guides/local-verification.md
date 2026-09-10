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
| 1 | Analytics Overview/Details state matrix — [Analytics Product](../product/features/analytics-collection-safety/spec.md) and [UI contract](../ui/features/analytics-collection-safety.md) | **Closed locally.** Commit `960c506` contains the reviewed implementation and ten-case gate. The final Colima run at coordination commit `ac96306` passed against source tree `0de50e073de4be41914d27035e2f6a930d629813adb474143ef0eaa36ec4179f`; the only dirty path was the runtime-irrelevant root guidance file `AGENTS.md`. | Primary agent owned the single final Colima run and acceptance; implementation and independent review are complete. | `target/rz/analytics-ui-state/current/manifest.json` records `linux/arm64`, ten successful runs and step receipts, 36 fixture requests, exact binary provenance, and the two named screenshot hashes. Focused API/UI tests passed 22 cases with 155 assertions; TypeScript and Web build passed; independent review found no remaining P1/P2. | Closed on 2026-09-07. The published manifest identifies the executed commit, source-tree digest, binary hashes, receipts, screenshots, cleanup, and atomic publication result. This documentation-only status update does not change those runtime inputs. |
| 2 | Monitoring console acceptance — [Monitoring Product](../product/features/monitoring/spec.md), [UI contract](../ui/features/monitoring.md), and [test matrix](./monitoring-testing.md) | **Closed locally.** The four routes now distinguish loading, initial failure, permission loss, background failure and populated paging/filter states. At 390px, Daily Summaries exposes the readable Date, Node and Coverage columns while detail columns remain hidden. | A `gpt-5.6-terra high` implementation owner completed the UI slice; a `gpt-5.6-sol high` owner hardened the cross-layer gate; the primary agent ran final Colima acceptance; an independent `gpt-5.6-sol high` reviewer found no remaining P1/P2. | `target/rz/monitoring-ui-state/current/manifest.json` records `linux/arm64`, source tree `812dc059e07a027b04044c4015d53ba3220609e8647e8a2cc2696273e405cdae`, 23 successful runs and step groups, 36 route-exact fixture requests, exact binary provenance, and 1440×900 plus 390×844 screenshots. Focused Reports layout tests passed 4 cases, Web tests passed 7 cases with 40 assertions, and gate tests passed 10 cases with 235 assertions. | Closed on 2026-09-07. All six mobile geometry assertions, overflow checks and screenshots passed; visual review confirmed no vertical text or right-edge clipping. This documentation-only status update does not change the runtime inputs bound by the manifest. |
| 3 | Automation and Reports acceptance — [Scheduled Report Product](../product/features/scheduled-report-automation/spec.md) and [UI contract](../ui/features/scheduled-report-automation.md) | **Closed locally.** The current `linux/amd64` Schedule schema-2 and Reports schema-3 Chromium manifests bind head `d3f5229e9aa2923def3052963a4d3bd763bef701`, source tree `5573f00aabb99172dc91dbd2d3b6d3a3a19fe4274ed7016a13be28a9ab190706`, and Chromium `120.0.6099.224-1~deb11u1`. | One implementation owner completed the Reports/Web slice; final Colima gates and visual review passed. | Schedule: `target/rz/schedule-form-browser/runs/20260910T165701Z-49652/manifest.json`, SHA-256 `96ecece268bb90e1d2bf90f17f5e884c214d129e7ffb1ca44467fdf8927c72eb`, binds seven run-step receipts and reviewed 1440x900/390x844 captures. Reports: `target/rz/reports-ui-state/runs/20260910T170537Z-54826/manifest.json`, SHA-256 `ba401c32839c079634773a5205a2aaf2d24ea4c14c32c5143281847bec38a444`, binds four browser receipts, cancelled processing, preserved source hashes, delivery gap 15, direct-mutation 403 receipts, and four visually reviewed captures. | Current manifests prove the bounded manager/view-only Templates/Runs journeys, processing, partial, failure, retry, permission, and retained-evidence behavior. Native systemd, confined native-host seccomp, deployed scheduling, production deployment, external delivery, and real production due-time execution remain **Not verified**. |
| 4 | Four-service module-log runtime — [Module Log Product](../product/features/module-log-diagnostics/spec.md) and [UI contract](../ui/features/module-log-diagnostics.md) | **Closed locally.** The final fresh Colima run published `target/rz/module-log-runtime/current/manifest.json` for head `21ed7a8`, source tree `f0f4ede624e96600e894bf9b5a097c6df138a7dd0e2c7ce6386fe667fd4e91d4`, and `aarch64`; independent review found no remaining P1/P2. | One `gpt-5.6-sol high` cross-layer implementation owner; the primary agent ran final Colima acceptance; one `gpt-5.6-sol high` reviewer completed final evidence review. | The final manifest binds 25 receipts; five exact non-owner 403 envelopes; an archive of 4 files and 9216 bytes with SHA-256 `de9330db80df01c6f4c30ce66921b61ae19295ac1653a51e4c939462dd1cf96a`; Reports process/directory/file UID/GID `999:999`; `/opt/rz/logs` mode `0711`; `/opt/rz/logs/reports` mode `0750`; four removed old files, zero failures, and unchanged current files. The first jq-verifier failure is retained under `target/rz/module-log-runtime/failed-runs/20260907T070512Z-28312/`; the published `current` manifest is the final result. | Closed on 2026-09-07. All four service-created prefixes passed owner list/tail/backup/cleanup, archive bytes matched their source snapshots and manifest, and cleanup preserved the current UTC-day files. Native systemd and production deployment remain **Not verified**. |
| 5 | Message center and SSE — [Composable Distribution Product](../product/features/composable-distribution/spec.md), [architecture](../product/features/composable-distribution/architecture.md), and [implementation plan](../product/features/composable-distribution/implementation.md) P5-P7 | **Closed locally.** P5 durable inbox, P6 Monitor/Reports delivery, P7 Access authority, backend SSE and Web message center have passed source, build, disposable Linux runtime and Chromium acceptance. | One cross-layer implementation owner; the primary agent owns each final Colima acceptance; one independent reviewer must accept the final P7c basis. | The P7c manifest `target/rz/message-center-browser/runs/20260908T034234Z-91415/manifest.json` has SHA-256 `abf2d85978f080c339bce8498205cdb650303009ea104da7dc68051426afdd77`. It binds source tree `c407429cf9cc88ddf340540f3f656566417521adc22fdb075bf4ab3de21eb9d6`, six `linux/arm64` binaries, selected/pure Web inventories and 18 receipts. Eleven real Chromium journeys cover desktop/mobile, durable loading/empty/detail/paging/read states, one browser-origin stream per journey, event-driven count/list reconciliation, stream-401 login recovery, 403 clearing and in-app Monitor deep linking; direct and same-origin proxy SSE use Bearer headers without URL credentials. Selected state contains 23 messages, 23 recipients, 22 reads and one incident; pure Monitor has no notification-owned module, schema object, API route, ingress listener or service owner. | P5-P7 are locally closed. Global-1,000 sustained load, native systemd, production reverse-proxy and production deployment remain **Not verified** under P8 or external target acceptance. |
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
- Add node is the current six-step offline installation explanation: it names the
  signed archive, release manifest, envelope, trusted public key, key ID,
  Controller pinning, root-only configuration source, and activation phase. The
  informational Drawer contains no token, URL, input, copy action, or executable
  command; its primary action remains disabled until those target-host boundaries
  are integrated.
- Global settings loaded and saved all four defaults together; the saved update time was read back. Values remain 90%, 90%, 90% and 90 seconds.
- At 390px viewport width, the drawer was corrected from 736px overflow to exactly 390px, then visually inspected. Desktop viewport restored afterward.
- Eight focused frontend checks, TypeScript and scoped lint passed. Remote-host installation, Windows setup, and complete role/browser error matrices were not exercised.
