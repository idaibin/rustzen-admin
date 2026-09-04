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
- Analytics: 22 Rust tests passed, including collection/query/delegation,
  admission bounds, origin policy and JSON error-envelope behavior. The browser
  overview/details pass confirmed that the policy-status display is absent.
- Reports: 26 tests passed; daily/weekly schedules exercise create, disable,
  next occurrence, permissions, and removal. Real headless Chrome executes six
  form steps, returns screenshot/live-frame artifacts, cancels a waiting run,
  runs two overlapping executions, and cleans up private profiles after success,
  cancellation, and overall timeout.
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

## Remaining acceptance

The complete visual/error/loading/focus matrix, final human visual acceptance,
Linux/Windows Agent collection, real deployment timers, production host tracker,
and release package/deployment remain unverified. The Colima x86_64 Reports
verifier passed delegated API, Chromium screenshot, non-root execution, user
namespaces, WAL/recovery/log ownership and cleanup; real systemd and a confined
native-Linux browser seccomp profile remain unverified. Admin module-log backup/cleanup
has source/test evidence but no new live browser acceptance in this pass.
The candidate DESIGN remains subject to its documented approval requirement.
Full Rust test runs use one harness thread because Admin fixtures share a global
permission cache; internal race tests still create parallel tasks.

## Nodes onboarding and configuration verification

- Nodes now owns Add node and Global settings drawers; the independent settings route and menu are removed. The live Monitor manifest exposes only overview, nodes, incidents and summaries.
- Built-in Browser verified required-node-ID validation, generated
  `rz-monitor-agent` setup command, waiting before an Agent report, and connected
  after a real local Agent reported as `local-dev-node`. Task-owned verification
  services were stopped after the checks.
- Global settings loaded and saved all four defaults together; the saved update time was read back. Values remain 90%, 90%, 90% and 90 seconds.
- At 390px viewport width, the drawer was corrected from 736px overflow to exactly 390px, then visually inspected. Desktop viewport restored afterward.
- Eight focused frontend checks, TypeScript and scoped lint passed. Remote-host installation, Windows setup, and complete role/browser error matrices were not exercised.
