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

The current source candidate is commit
`eb9fd494c0e6e05824293887d227096846f08793`. Service wiring, Web formatting,
lint, type checking, 152 Web tests, 12 Insights tracker tests, the production Web
build, Rust formatting, workspace check and workspace clippy pass for that commit.
The deployment request and recovery suites also pass, including concurrent request
publication, durable transaction acceptance and failed-update propagation after a
successful rollback.

`just check` is the final source gate. Its Rust test stage is not fully executable in
the current restricted macOS runner: 15 tests that bind loopback TCP sockets fail at
`bind(2)` with `Operation not permitted`; the remaining exercised tests pass. Those
environment failures are not accepted as a passing source gate and must be rerun in
the release Linux environment. A passing source gate proves neither a signed artifact
nor systemd behavior.

## Current local candidate

There is no signed x86_64 candidate for the current source commit. The files currently
named `target/rz/rz-0.5.1-x86_64.tar` and `target/rz/rz-install` predate
`eb9fd494c0e6e05824293887d227096846f08793` and are invalidated for release use.
Rebuild, signature verification, exact source binding and all downstream artifact gates
must use one newly committed source identity.

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
| RZA-006 | Accept the complete signed server release | **In progress.** Source fixes are committed at `eb9fd494c0e6e05824293887d227096846f08793`; all non-network source gates pass, while the restricted runner cannot execute TCP-binding tests. No signed artifact is current for this source identity. | Rerun the complete source gate in the release Linux environment; build and verify the signed bundle and installer from the same committed SHA; inspect signature, members and static ELF identity; complete dual-Agent and fresh PID1/systemd install; `rz start/status/restart/stop`; injected child failure; four exact-version health checks; service UID/GID and filesystem-boundary checks; restart recovery; update and rollback tests. |
| RZA-007 | Accept an external production deployment | **Blocked / Not verified.** The last recorded production check reported `0.5.0` and an Nginx HTTP 413 before Admin received the upload. Current production state cannot be refreshed from the restricted runner. | Verify and raise the effective reverse-proxy request-body limit to at least 257 MiB, upload the new source-bound candidate, deploy from the verified current production version, verify all four service versions and health gates, execute the scheduled task and physical Agent journeys, and capture all 19 routes from production. |

## Known release blockers

1. The complete source gate, signed x86_64 artifact and all Linux runtime evidence must
   be regenerated from `eb9fd494c0e6e05824293887d227096846f08793` or a later reviewed
   commit; the current runner cannot bind TCP or access its Colima Docker socket.
2. Fresh PID1/systemd installation, lifecycle, failure and rollback evidence must be
   regenerated from that exact candidate before it is release-ready.
3. Production Nginx must accept the signed bundle plus multipart framing; the last
   observed HTTP 413 prevented upload, deployment and production screenshots.

## Evidence rules

- Source, build, artifact, installation, systemd, browser and external deployment are
  separate evidence layers.
- A build from another tree or an earlier dirty-tree digest is not current evidence.
- Colima is target-like local evidence, not production deployment.
- External deployment, publication, commit and push remain separate actions.
- Full Rust tests use one harness thread because Admin fixtures share a global
  permission cache; concurrency-sensitive tests create their own parallel tasks.
