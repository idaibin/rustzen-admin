# Monitoring Architecture

The product behavior is defined by
[`docs/product/features/monitoring/spec.md`](../product/features/monitoring/spec.md).
This document fixes the implementation architecture for that behavior.

## Runtime topology

```text
managed server
  rz-monitor-agent
      |
      | POST /api/monitor/agent-reports every 30 seconds
      v
rz-admin public module gateway :9801
      |
      | signed module delegation
      v
rz-monitor controller :9802 (loopback)
      |
      v
data/db/monitor.db
```

Admin remains the public gateway and RBAC owner. Monitor remains one independent service inside
the existing four-service release. `ModuleRouter` is the only method, path, access, handler, and
permission registry. `module.toml` owns menu metadata only.

## Ownership

### Agent

Agent owns only:

- a stable configured node ID;
- one boot ID and increasing sequence for the running process;
- fixed CPU, memory, and eligible real per-mount disk collection, excluding pseudo filesystems,
  ephemeral container mounts, and duplicate local bind mounts;
- one report attempt every 30 seconds;
- request logging.

The installed service also owns one system-service readiness transition. It emits `READY=1` only
after its first `accepted` or `duplicate` Controller response. A duplicate is delivery-confirmed by
the Controller's fencing result; `stale`, authorization failures, malformed responses, TLS failures,
and network failures do not transition readiness. The signal is sent at most once per process and
does not add local report state.

Agent has no SQLite database, policy engine, Incident state, report state, Outbox, inbound port, or
Controller-managed configuration.

### Controller

Controller owns:

- Agent authentication and report validation;
- current node/boot fencing and latest status;
- 30-day CPU, memory, and disk samples;
- four global default alert settings and optional complete per-node overrides;
- persistent consecutive-sample counters;
- active and resolved Incidents;
- offline-node scanning;
- daily node summaries;
- 30-day cleanup.

### Web

Web reads and mutates Monitor only through the Admin gateway. It does not calculate online state,
resource percentages, alert counters, retention, or Incident transitions.

## Report lifecycle

Each Agent process generates a boot ID and starts its sequence at one. A report from the current
boot is new only when its sequence is greater than the last accepted sequence and its collection
time is strictly later than the latest accepted collection time. A new boot may take over only with
sequence one and the same collection-time ordering. Collection time must stay within an inclusive
five-minute skew of Controller receive time; liveness uses receive time only. Once superseded, a
boot ID is retired and cannot refresh the node.

One accepted report uses one SQLite transaction:

1. validate the report and decide `accepted`, `duplicate`, or `stale`;
2. register or update the node and current boot;
3. update latest node state;
4. insert CPU and memory samples;
5. insert one disk sample per reported eligible mount point;
6. resolve an active offline Incident;
7. resolve the node's effective policy from its override or the global defaults, then update alert
   counters and resource Incidents;
8. commit.

Any storage or alert-state failure rolls back the complete report. Duplicate and stale reports do
not enter the transaction that mutates monitoring state.

## Alert lifecycle

CPU, memory, and every disk mount are independent targets. Each target stores an abnormal counter
and a normal counter. Three consecutive samples at or above the configured threshold create one
active Incident. Three consecutive samples below it resolve the Incident. A sample that changes
side resets the opposite counter.

Controller scans node liveness every 30 seconds. When the elapsed time since the last accepted
report is greater than the configured offline duration, it creates one active offline Incident.
The next accepted report resolves that Incident immediately.

Changing a threshold clears unfinished counters for that metric. Disabling a setting clears its
counters and resolves its active Incidents with `setting disabled`. Resource evaluation happens in
the report transaction; it is not a separate database polling loop.

## Persistence

The Monitor database initializes directly with the central Agent-report schema. The persistent
owners are:

- `monitor_nodes`: registration, current boot, sequence, latest report and resource state;
- `monitor_boots`: current and retired boot IDs used for fencing;
- `resource_samples`: CPU and memory history;
- `disk_samples`: per-mount disk history;
- `alert_settings`: the four global settings;
- `node_alert_settings`: optional complete per-node overrides; absence means dynamic global
  inheritance;
- `alert_counters`: persistent consecutive-sample state per node, kind, and target;
- `monitor_incidents`: active and resolved Incident records;
- `node_daily_summaries`: per-node daily summaries.

An active-Incident partial unique index enforces one active row per `(node, kind, target)`. Foreign
keys cascade node-owned history when the owning node is eventually removed. This initialization
baseline does not include an upgrade path from the previous TCP-check database; an existing old
Monitor database must be reset and initialized again. No compatibility query or dual-write path
remains.

## Background work

Controller runs only three Monitor background jobs:

- every 30 seconds: offline-node evaluation;
- after a statistics day closes: idempotent daily summary generation;
- periodically: delete raw samples, resolved Incidents, and daily summaries older than 30 days,
  then report the row-deletion result separately from repository SQLite reclaim maintenance. The
  deletion transaction is the only operation that makes cleanup fail; after deletion commits, a
  maintenance failure is returned as a pending maintenance result and retried by the next run
  when a freelist remains, even if that run deletes no additional rows.

Current node state, active Incidents, and alert counters are never removed by retention.

## Failure boundaries

- Controller or gateway unavailable: Agent logs the attempt and waits for the next interval; there
  is no old-sample retry or backfill, and the service remains unready.
- Invalid Agent credentials or payload: no monitoring mutation.
- Only accepted or duplicate report responses make the installed Agent ready; `401`, `stale`, other
  HTTP failures, malformed responses, TLS failures, and network failures cannot make it ready.
- Fencing classifies duplicate, lower-sequence, retired-boot, and invalid takeover reports before
  clock-skew validation; only a report that could be accepted is rejected for an excessive skew.
- Duplicate, stale, or retired boot: no online-time, sample, counter, or Incident mutation.
- Controller restart: SQLite restores settings, counters, nodes, samples, and Incidents.
- One malformed disk target rejects the whole report instead of storing a partial node state.

## Removed architecture

The terminal implementation has no TCP checks, check scheduler, check result retention, aggregate
disk field, 10-second resource evaluator, acknowledged Incident state, Agent database, control
protocol, configuration operation, Outbox, Inbox, or remote query operation.
