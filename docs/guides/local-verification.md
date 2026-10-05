# Local verification status

The release-level snapshot below was recorded on 2026-09-23. The subsequent scoped
2026-10-04 local checks are consolidated in
[`local-runtime-coverage.md`](../plans/local-runtime-coverage.md). Executed-check
success is separate from overall completeness. The 2026-10-05 local continuation
below restores scoped browser acceptance; neither batch certifies a signed or deployed release.
Historical runs remain in Git and in their source-bound machine-local manifests;
they do not certify a changed working tree or a different release artifact.

## Current scope

The server release has one path:

```text
build complete bundle -> copy bundle and installer -> install -> rz start -> four services healthy
```

The bundle contains Admin, Monitor, Insights, Reports, Web, notifications and `rz`.
Runtime module disablement does not create another release inventory. The managed-node
Agent remains a separate non-Web artifact.

## Current-source checks

The working tree is an integrated change set covering:

- Web shared components and their page consumers;
- Admin, Monitor, Insights and Reports behavior and contracts;
- the full signed package, installer, systemd layout and `rz` lifecycle commands;
- current product, architecture, deployment and UI documentation;
- 1920x1080 full-runtime screenshots for all 19 authenticated business routes.

The mainline PR starts at `0acc38d2a2736eb5ce7f063e125e010530786dd9`.
`just check` passed on 2026-09-23 with service wiring, Web formatting, lint, type
checking, tests and production build, plus Rust formatting, workspace check, clippy
and workspace tests. The check ran before this status-only documentation update;
the final PR commit requires a clean-tree readback. A passing source gate proves
neither artifact provenance nor systemd behavior.

## Current local candidate

The machine-local `target/rz/rz-0.5.1-x86_64.tar` exists and passed
`bun scripts/deploy-sign.mjs verify-bundle --file target/rz/rz-0.5.1-x86_64.tar
--version 0.5.1 --arch x86_64` on 2026-09-23. Its SHA-256 is
`01acde7fa359794d933bb5e6ae37400555651b99da5675b66bbee900ff6942ad`.
The signature check does not establish that the bundled binaries were built from
the final PR commit or that this local file is identical to the installed artifact.
Those provenance and publication checks remain **Not verified**.

## Recorded browser evidence

The README screenshots were captured from the complete service set in Colima with a
1920x1080 browser viewport. The fixture included two online Monitor nodes, at least one
Insights access record and one enabled Reports automation. The four representative
images are:

- `docs/assets/screenshots/dashboard.png`;
- `docs/assets/screenshots/monitoring-nodes.png`;
- `docs/assets/screenshots/analytics-details.png`;
- `docs/assets/screenshots/management-scheduled-tasks.png`.

These images prove only the recorded routes and data at their captured source identity.
They are not a substitute for the current-source build, artifact, fresh-install or
service-lifecycle gates.

## Active delivery ledger

| ID | Outcome | Current status | Completion evidence |
| --- | --- | --- | --- |
| RZA-006 | Accept the complete signed server release | **Partially verified.** `just check` passed and the local 0.5.1 bundle signature verifies. Final-commit-to-binary provenance and complete fresh PID1/systemd, upgrade and rollback acceptance are **Not verified** by these checks. | Verify the final committed source identity against the signed bundle and installer, then complete the target-like install, lifecycle, update and rollback gates on that exact artifact. |
| RZA-007 | Accept an external production deployment | **Partially verified.** On 2026-09-23 the ECS Workbench check observed all four services and the public `/health` reporting 0.5.1, and `rz doctor` passed. Nginx was reloaded with a 50m request-body override for the Admin HTTPS host. This does not prove installed-file parity with the local bundle or the full production browser and Agent journeys. | Verify installed artifact identity, the required production journeys and the effective upload boundary for future packages. |
| RZA-008 | Integrate the GitHub mainline | **Complete.** Fresh 2026-10-05 remote readback is `edd06daf34bf106064fee7241ff70600a8bf236b`, the merge of PR #6, with parents `fa18de2` and `b0a416b`. | The release branch is integrated. The later theme/interaction feature branch remains separate; this does not close RZA-006 or RZA-007. |

## Known release blockers

1. The final PR commit, local signed bundle and installed production files do not yet
   have one verified source-and-artifact identity chain.
2. The complete fresh PID1/systemd installation, lifecycle, update and rollback
   acceptance remains separate from the passing source and signature checks.
3. The Admin HTTPS Nginx limit is 50m. It covers the observed 49,005,594-byte bundle
   with limited multipart headroom, but not the full 256 MiB Admin upload contract.
   A real upload at the boundary has not been repeated after the Nginx change.

## Evidence rules

- Source, build, artifact, installation, systemd, browser and external deployment are
  separate evidence layers.
- A build from another tree or an earlier dirty-tree digest is not current evidence.
- Colima is target-like local evidence, not production deployment.
- External deployment, publication, commit and push remain separate actions.
- Full Rust tests use one harness thread because Admin fixtures share a global
  permission cache; concurrency-sensitive tests create their own parallel tasks.


## 2026-10-05 local continuation

Incoming source: `ac8ce7321fdfff73e5e534f390800c4a7475cda6`; new branch
`feat/theme-verification-20261005`. Production Web, API/account, Monitor, Insights
and Reports behavior remain unchanged. DESIGN SHA-256 remains
`c401f71cbb66206b4166e85a3339b027ff0183efd44e7e0ce74280b4425f93cd`.

The managed isolated Chromium 153.0.8010.0 operated the complete locally built
four-service composition with newly initialized owned SQLite databases. This is a new
local verification surface; it does not retroactively change the old cloud-browser
refusal or certify a public preview. The final runner refuses occupied service ports,
records executable hashes, and shuts down its browser/services with bounded cleanup.

- Web: 167 tests, 916 assertions, no failures; types, formatting, lint and production
  build pass. Existing lint warnings in API test files and the large-chunk build
  warning remain. Tracker: twelve VM tests, 154 assertions; service wiring check pass.
- Rust workspace: 475 tests pass, one explicitly ignored release-only performance
  benchmark. Reports configuration-only test first failed because it auto-discovered
  host Chrome; it now supplies a non-executed test path. Production launch is unchanged.
- CLI Clippy first failed on a same-type `u32` conversion on Linux. Comparing both
  permission values as `u64` preserves Unix width handling and the original check.
  All nine CLI tests pass after that change. Workspace fmt/check/Clippy pass.
- Actual browser filechooser event, unsupported MIME and over-1MiB client rejection
  with zero HTTP dispatch, real server rejection of corrupt PNG, one labeled network
  failure followed by a successful actual retry, and exactly one held pending request.
- Uploaded PNG bytes agree with the served resource. Avatar remains after browser
  reload, one owned Admin process restart, and a fresh login after clearing browser
  storage. The native operating-system picker interface itself is not automated.
- 19 authenticated routes × light/dark × 1920×1080, 1440×900 and 390×844: 114
  current views, all expected headings/URLs, no route alerts, page exceptions or
  document horizontal overflow. These views are rendering coverage, not 114 business
  write journeys. Fresh Monitor/Analytics/Reports datasets are empty.
- 19 final PNGs cover representative route/theme views, profile compatibility/mobile
  layouts, pending upload and the real backend rejection. The initial selector-only
  failed run and intermediate successful runner are retained separately.

Reproduction (after the four default-feature debug binaries and actual Web bundle are
built according to `justfile`):

```bash
RUSTZEN_BROWSER_PATH=/absolute/path/to/isolated/chromium \
RUSTZEN_PLAYWRIGHT_MODULE=/absolute/path/to/playwright \
node scripts/verify-profile-browser.mjs
```

Default evidence location is `target/rz/profile-browser/<timestamp>/result.json`.
The delivered screenshot/report ZIP contains the final JSON receipt, exact runner,
check logs and PNG hashes, excluding fixture databases and credentials. Final receipt
SHA-256: `abe31f91b9324e841b41ba9f35b9b81c073db1a4b420923497c5682f5d09a421`.

The current source checks were run via the underlying `just check` commands, not a
claim that the `just` executable/parser was run. This self-reviewed continuation is
not independently reviewed or merge-ready. Signed release provenance, complete
PID1/systemd install/update/recovery, production upload limits, selected-portal runtime,
real Agent transport, successful Reports rendering and all-role/concurrency journeys
remain separate acceptance work. The dashboard disk-discovery fixture reports 0 B/0 B
in this sandbox; it is not evidence of real disk capacity.
