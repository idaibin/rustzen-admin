# Monitoring Testing

This matrix is the executable acceptance basis for Monitoring. Tests use an in-memory SQLite pool
unless restart durability requires a temporary file database.

## Landed behavior tests

The current `cargo test -p rustzen-monitor` suite verifies the following boundaries through
real functions, HTTP handlers, and the SQLite persistence layer:

- `protocol::tests::validates_each_wire_field_at_its_declared_boundary` covers inclusive
  boundaries and out-of-range values for identity, hostname, version, CPU, byte counts,
  sequence numbers, and mount points; malformed RFC3339 and an empty disk list are verified
  separately.
- `agent::tests::agent_collection_maps_fixed_values_and_skips_invalid_devices` verifies fixed
  CPU, memory, and multi-mount mapping, and confirms that empty paths, zero capacity, invalid
  available capacity, pseudo filesystems, ephemeral container mounts, loop devices, and
  duplicate local bind mounts do not pollute reports; real `/`, `/data`, and `/var/lib/docker`
  mounts are retained, and same-source network mounts are not merged incorrectly.
- `agent::tests::agent_send_report_observes_request_timeout` and
  `agent_response_statuses_and_failures_are_distinct` cover timeout, 401, 503, accepted,
  duplicate, stale, and bad responses; `next_sequence` verifies that failed or expired
  submissions do not advance the sequence. The standalone Agent test entry is
  `just verify-monitor-agent`; the Controller's default test command does not include Agent
  tests.
- `agent_loop_keeps_sequence_and_skips_missed_ticks_for_all_send_outcomes` invokes the
  production `run_agent_loop` directly and verifies through the injectable
  collector/sender/readiness seam that readiness is sent exactly once only after `accepted`
  or `duplicate`, that 401, 503, timeout, stale, and missed ticks send no readiness, and that
  the existing sequence and scheduling behavior is preserved.
- `app::tests::agent_report_route_returns_accepted_duplicate_and_stale_envelopes` verifies
  the Controller report envelopes; the existing route tests verify token-before-body, 401,
  and 422.
- `historical_duplicate_stale_and_retired_reports_bypass_clock_skew` and
  `historical_fenced_reports_return_200_statuses_but_new_sequence_is_422` verify that fencing
  precedes clock skew: historical duplicate/stale/retired reports still return 200 without
  side effects, and only a new sequence returns 422.
- `older_collected_at_is_stale_even_with_an_increasing_sequence` and
  `report_clock_skew_has_an_inclusive_five_minute_boundary` verify strictly increasing
  collection time, the five-minute inclusive skew allowance, and far-future/past rejection;
  server-side liveness still uses receive time; `concurrent_replay_has_one_accept_and_one_duplicate`
  verifies the SQLite fencing race cannot produce duplicate samples or a 500.
- `threshold_boundary_applies_to_cpu_memory_and_each_disk`,
  `disabled_offline_alert_does_not_create_incident`,
  `invalid_alert_settings_are_rejected_without_partial_update`,
  `metric_buckets_align_by_epoch_and_retain_independent_mount_series`,
  `file_cleanup_reports_reclaim_maintenance_and_is_idempotent`,
  `cleanup_retries_maintenance_after_committed_deletion_failure`, and the retention tests
  cover inclusive threshold boundaries, disabled settings, invalid settings, five-minute
  buckets, independent per-mount series, file-database freelist maintenance, maintenance
  failure results and retries, and retention/cleanup.
- `sqlite_foreign_key_and_active_incident_uniqueness_are_enforced`, the unique
  initialization baseline, and the file-reopen tests cover foreign keys, the active-incident
  uniqueness constraint, creating only the final schema, and restart persistence.

The current acceptance baseline: Controller 47 and Agent 12 tests pass; `just check` and the
Admin OpenAPI/client contract checks pass. The underlying service verification of
`just verify-modules-mvp` validates 24 startup orders, a native macOS Agent report through
the Admin gateway, service isolation, and database recovery against fresh four-service
databases. The added `scripts/verify-monitoring-scenarios.mjs` verifies real owner/viewer
permissions, the four navigation entries, global and node policies, duplicate/stale
isolation, three-sample CPU/disk alert triggering and three-sample recovery, incident
paging/details, invalid input, and the 30-day query boundary. That script is invoked by the
unified worker verification entry.

The browser pass has covered the four pages, node multi-mount details, custom policy save
and reset, incident details, settings save, and the daily-summary empty state. Complete
paging/filter combinations, non-30-second daily-summary sampling coverage, real
Linux/Windows collection, deployed timers, and the complete visual/error state matrix still
require separate acceptance. Overview/Nodes currently have no paging contract; paging
acceptance applies to Incidents/Summaries; the evidence above does not replace production
deployment verification.

## P0 report and persistence

- Reject missing or wrong Agent token.
- Reject empty/invalid node ID, invalid timestamp, non-finite/out-of-range CPU, zero totals,
  `used > total`, empty mount point, and duplicate mount points.
- Prove every rejected report leaves node time, samples, counters, and Incidents unchanged.
- Accept the first `bootId` report only at sequence one.
- Accept strictly increasing sequence for the current boot.
- Return `duplicate` for an exact replay without side effects.
- Return `stale` for a lower sequence, retired boot, or non-one new-boot takeover.
- Prove a new boot at sequence one takes ownership and retires the prior boot.
- Inject a failure after sample insertion and prove the complete report transaction rolls back.
- Prove CPU, memory, and every disk mount are stored independently.

## P0 resource alerts

Run the same behavior table for CPU, memory, `/`, and `/data`:

| Samples | Expected result |
| --- | --- |
| high | abnormal count 1, no Incident |
| high, high | abnormal count 2, no Incident |
| high, high, high | one active Incident |
| fourth high | still one active Incident, latest evidence updated |
| high, normal, high | abnormal count reset; no false trigger |
| active + normal | active, normal count 1 |
| active + normal, normal | active, normal count 2 |
| active + normal, normal, normal | resolved |

Also prove that `/` and `/data` do not share counters or Incidents and that one target cannot have
two active Incident rows.

## P0 offline alerts

- At exactly the configured boundary the node remains online.
- After the boundary one offline scan creates one active Incident.
- Repeated scans retain one active Incident.
- The next accepted report resolves it immediately.
- A duplicate, stale, or retired-boot report cannot resolve it or refresh liveness.
- Disabling offline alerts resolves active offline Incidents and prevents new ones.

## P0 restart durability

Using a file-backed SQLite database:

- accept reports until an abnormal or normal counter reaches two;
- close and reopen Controller storage;
- prove the next matching report reaches three and performs the expected transition;
- prove nodes, latest state, settings, samples, and active/resolved Incidents survive restart.

## P1 settings

- Defaults are CPU 90%, memory 90%, disk 90%, and offline 90 seconds.
- Reject thresholds outside 1-100 and offline duration outside 30-3600 seconds.
- A node without an override always reads the current global defaults.
- Saving node settings creates one complete custom override and marks its source as custom.
- A global edit resets only affected unfinished counters for inheriting nodes; custom nodes keep
  their effective settings and counters.
- Resetting a custom node removes its override, returns the current global defaults, and resets only
  counters whose effective setting changed.
- A threshold edit resets only affected unfinished counters.
- An active Incident is evaluated against the new threshold from the next accepted report.
- Disabling CPU, memory, disk, or offline resolves only that type with `setting disabled`.
- Disabled settings do not create new Incidents.

## P1 retention and summaries

- Delete samples, resolved Incidents, and daily summaries strictly older than the 30-day cutoff.
- Keep rows exactly at the cutoff.
- Keep latest node state, active Incidents, and unfinished counters regardless of age.
- Run cleanup twice and prove idempotence.
- Generate one daily summary per node/date and prove rerunning with unchanged input is idempotent.
- Prove CPU/memory and every mount summary use only accepted samples for that day.
- Prove the API rejects query windows longer than 30 days.

## P1 Agent behavior

- One collection contains CPU, memory, and each eligible real disk mount once per local block
  device; pseudo filesystems and ephemeral container mounts are absent.
- Successful, rejected, and network-failed submissions all wait for the next scheduled interval.
- The installed Agent sends its system-service readiness signal exactly once only after `accepted`
  or `duplicate`; `401`, `stale`, malformed/other HTTP responses, TLS failures, and network failures
  leave it unready.
- No path persists, retries, or backfills an old sample.
- Test scheduling with an injected interval/clock; do not sleep for real 30-second periods.

## P1 API and Manifest

- Runtime Manifest is derived from registered Rust routes.
- `agent-reports` is public but still enforces Agent token.
- Every protected route has the documented capability.
- There is no `heartbeat`, checks, check-result, or acknowledged-Incident route.
- Backend response fields match `apps/web/src/api/monitor` consumers.
- Error tests cover `401`, `404`, and `422` plus `accepted`, `duplicate`, and `stale` report results.

## P2 Web

- Navigation contains Overview, Nodes, Incidents, and Daily summaries only; Global settings opens from Nodes.
- Overview distinguishes online/offline nodes and active Incidents.
- Node details display independent disk-mount series and a maximum 30-day query.
- Incidents display only active/resolved states.
- Users without `monitor:manage` cannot mutate settings.
- Loading, empty, error, disabled-alert, stale-node, and partial-day states are distinguishable.

## Implementation order and gates

1. Migration and storage constraints.
2. Agent report DTO, validation, fencing, and atomic persistence.
3. Resource alert counters and Incident transitions.
4. Offline scan.
5. Settings mutation.
6. Daily summaries and retention.
7. ModuleRouter, capability, and Web contract.
8. UI after its separate specification is Ready.

After each slice run its named focused tests, then `cargo fmt --all -- --check`,
`cargo check -p rustzen-monitor`, `cargo clippy -p rustzen-monitor --all-targets -- -D warnings`,
`cargo test -p rustzen-monitor`, and `git diff --check`. Cross-module completion additionally runs
`just verify-modules-mvp`; browser and deployed multi-node behavior remain separate gates.

## Monitoring UI state Linux gate

`just verify-monitoring-ui-state-linux` has one total container budget controlled by
`RUSTZEN_MONITORING_UI_STATE_TIMEOUT`: 2100 seconds by default and at most 2400.
It covers the 23 route-state browser runs, their eight 31-second background waits,
four 5-second loading fixtures, and delivery-card runs; Docker architecture discovery
and cleanup retain their separate bounded timeouts.

The local gate closes only when
`target/rz/monitoring-ui-state/current/manifest.json` matches the current checkout
and has `status: "passed"`. Its receipt set covers 23 canonical route runs, 18
owner and 26 viewer delivery steps, four screenshots, real Monitor SQLite and
authorized API receipts, permission behavior, and either zero retries or a
recorded single retry receipt.

## Linux dual-Agent runtime gate

`just verify-monitor-agent-multi-node-linux` is a bounded, native-architecture
Colima/Docker check. It creates a fresh central Admin and Monitor SQLite state plus
two independent unprivileged Agent service identities. Each Agent has its own node ID,
runtime root, log directory, and Unix readiness socket. Before central services start,
neither readiness socket may receive `READY=1`. Once Admin and Monitor are healthy,
the gate requires both Agent reports to be accepted, verifies the two nodes through the
Admin gateway, checks distinct current boot IDs and raw metric samples, restarts the
central processes, and requires both existing nodes to continue reporting without a
third registration.

The Agent cold build uses `RUSTZEN_MONITOR_MULTI_NODE_BUILD_TIMEOUT` (900 seconds by
default, capped at 1800). The actual topology uses the separate
`RUSTZEN_MONITOR_MULTI_NODE_TIMEOUT` budget (240 seconds by default, capped at 600),
so dependency compilation cannot exhaust or silently extend the runtime acceptance.
The evidence lock and cleanup trap are installed before that build starts. Both the
named build container and named runtime container must be absent before evidence can
be published. Failure-log capture is separately bounded at 15 seconds (operator cap
60) before TERM, followed by at most 10 seconds of hard-kill grace. That bounded
window may delay runtime-container removal, but it cannot prevent removal or cleanup;
a timeout or signal preserves the previous `current` evidence.

Evidence is atomically published below
`target/rz/monitor-agent-multi-node/current/manifest.json`. The manifest contains the
source digest, platform, staged binary digests, service UID evidence, readiness events,
Admin query outcomes, and recovery result; it never contains the Agent token. This is
real Linux process and service-account evidence, not a systemd PID 1, remote-host,
production TLS, or independent-kernel test.

## Linux dual-Agent Nodes Chromium gate

`just verify-monitor-agent-multi-node-ui-linux` reuses the two real Agent runtime
topology and adds the Admin Web shell plus Reports-driven Chromium. It binds the
current source identity and SHA-256 values for Admin, Monitor, Reports, and Agent
binaries. The gate obtains the two node ID/current boot ID pairs from Admin, opens the
Nodes list and each detail Drawer, checks the same boot ID and the 5-minute history
surface, scrolls its two single-bucket series markers into the viewport, verifies both
are visible, and publishes one ordered browser-step receipt plus two 1440x900 detail PNGs.
Each PNG has SHA-256 and byte-size evidence. The manifest also binds the exact submitted
step list and the Reports API's persisted flow definition before it accepts the
receipt's contiguous successful step sequence.

The pinned browser verifier intentionally contains no Node or Bun runtime. Its small
dual-node step generator therefore uses the image's already-pinned Python standard
library; the generated JSON is independently reconstructed and compared by the host
Bun evidence verifier after the container exits.

The named verifier container must be absent before a successful run is published.
On failure, cleanup keeps the previous `current` link, removes staged binaries and
the lock, and stores only bounded diagnostic output plus non-secret provenance under
`target/rz/monitor-agent-multi-node-ui/failed-runs/<run-id>/`. Browser definitions,
API receipts, screenshots, and fixture credentials are excluded from failed-run
evidence. If Docker removal or the following absence check fails, the gate reports
a cleanup failure and retains its ownership lock. A later run is rejected until the
operator removes the named residual container and the stale lock.

Its evidence is local, disposable Linux-container evidence only. It does not replace
the native dual-Agent gate and does not certify systemd PID 1, independent hosts,
production TLS, or the complete Monitoring UI state matrix.
