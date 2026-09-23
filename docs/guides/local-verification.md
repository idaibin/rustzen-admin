# Local verification status

This document records only the current delivery boundary as checked on 2026-09-23.
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
| RZA-008 | Integrate the GitHub mainline | **In progress.** Local `main` was returned to GitHub's `fa18de2` and the 304 local-only commits were preserved on `release/0.5.1-mainline-pr`. | Review the final branch, open one PR against GitHub `main`, select its merge method, merge, then read back local and remote refs. |

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
