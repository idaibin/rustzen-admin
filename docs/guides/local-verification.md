# Local verification status

This document records only the current delivery boundary. Historical runs remain in
Git and in their source-bound machine-local manifests; they do not certify a changed
working tree.

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

Applicable focused Rust, Bun and shell checks are rerun after each affected slice.
`just check` is the final source gate. A passing source gate proves neither a signed
artifact nor systemd behavior.

## Current local candidate

The current x86_64 candidate was rebuilt from the accepted code inputs in Colima after
the source gate passed:

- bundle: `target/rz/rz-0.5.1-x86_64.tar`;
- bundle SHA-256: `e49e7e24ae189ee50b45294d94aaeea6818d19c7f83b0be7d527795f8d53be17`;
- bundle size: `49,001,498` bytes;
- installer: `target/rz/rz-install`;
- installer SHA-256: `e0075d49d5c46ab4fafa15c528eaf91184d497cb19546f1fb40a50247fe4d9d5`.

The Ed25519 bundle verification, 22-member inventory, five static x86_64 ELF checks,
four-service process and database isolation gate, and dual-Agent Controller pairing all
passed. The current `0.5.1` candidate has not yet completed a fresh PID1/systemd lifecycle
and `0.5.0` to `0.5.1` rollback run.

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
| RZA-006 | Accept the complete signed server release | **In progress.** The final source gate, signed x86_64 full candidate, four-service verification and dual-Agent pairing are complete. Fresh PID1/systemd lifecycle, failure, recovery and rollback acceptance for this exact `0.5.1` candidate remain outstanding. | `just check`; `just verify-services`; signed bundle and packaged installer; signature, member and static-ELF inspection; dual-Agent Linux runtime evidence; then fresh PID1/systemd install; `rz start/status/restart/stop`; injected child failure; four exact-version health checks; service UID/GID and filesystem-boundary checks; restart recovery; update and rollback tests. |
| RZA-007 | Accept an external production deployment | **Blocked / Not verified.** The authorized target `https://admin.rustzen.dev` is reachable and currently reports `0.5.0`. Uploading the exact `0.5.1` bundle is rejected by Nginx with HTTP 413 before Admin receives it. Production remains unchanged. | Raise the reverse-proxy request-body limit above the signed bundle size, then upload the exact recorded digest, deploy from `0.5.0`, verify all four service versions and health gates, execute the scheduled task and physical Agent journeys, and capture all 19 routes from production. |

## Known release blockers

1. Fresh PID1/systemd installation, lifecycle, failure and rollback evidence must be
   regenerated from the exact `0.5.1` candidate before it is release-ready.
2. Production Nginx must accept the 49,001,498-byte bundle plus multipart framing;
   the observed HTTP 413 currently prevents upload, deployment and production screenshots.

## Evidence rules

- Source, build, artifact, installation, systemd, browser and external deployment are
  separate evidence layers.
- A build from another tree or an earlier dirty-tree digest is not current evidence.
- Colima is target-like local evidence, not production deployment.
- External deployment, publication, commit and push remain separate actions.
- Full Rust tests use one harness thread because Admin fixtures share a global
  permission cache; concurrency-sensitive tests create their own parallel tasks.
