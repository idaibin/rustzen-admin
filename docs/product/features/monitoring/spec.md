# Monitoring

Status: Implemented; local gateway/service scenarios verified on 2026-09-03.
Representative browser journeys verified. A disposable native-architecture Linux
gate covers two independently-run Agents across an unavailable Controller, first
delivery, and Controller restart; it is not a deployed multi-host or systemd
acceptance. The complete visual/state matrix and deployed multi-node behavior remain
Not verified. See
[Monitoring Testing](../../../guides/monitoring-testing.md).

## Product outcome

Monitoring provides one central place to view managed-node health, inspect the most recent 30 days
of resource data, configure resource thresholds, and handle current or recently resolved alerts.

A lightweight `rz-monitor-agent` reports fixed node information every 30 seconds. The Monitor
Controller stores accepted reports, calculates status, evaluates alerts, and produces daily
summaries. Admin continues to own identity, permissions, the gateway, releases, and the unified Web
application.

## Scope

Agent reports:

- stable node identity and build information;
- collection time;
- CPU usage percentage;
- used and total memory;
- used and total space for each eligible real mounted disk partition; pseudo filesystems,
  ephemeral container mounts, and duplicate local bind mounts are excluded before reporting.

Agent does not receive configuration, execute configurable checks, evaluate alerts, store monitoring
history, produce reports, or answer historical queries. Monitoring has no TCP, port, HTTP, process,
log, or custom-command check capability.

Controller owns:

- node registration and latest state;
- 30 days of accepted resource samples;
- global default alert settings and optional per-node overrides;
- active and recently resolved incidents;
- daily node summaries within the same 30-day window.

## Monitoring flow

For each Agent report, Controller performs one ordered business flow:

1. Authenticate the Agent and validate node identity, collection time, and resource values.
2. Reject duplicate or older reports without changing node online time, samples, or alert state.
3. Update the node's latest state and append one resource sample.
4. Evaluate CPU, memory, and every reported disk partition independently against the current global
   settings.
5. Create, retain, or resolve the corresponding incident.

Controller checks missing reports on its own schedule. A node does not need to send a request for an
offline incident to be created.

Fencing is evaluated before clock-skew validation. Duplicate, lower-sequence, retired-boot, and
invalid-takeover reports return their `duplicate`/`stale` result without mutation. `collectedAt` is
ordered per node: for the current boot, an accepted report must have a strictly
later collection time than the latest accepted report. It must also be within five minutes before or
after the Controller receive time. The five-minute bound is inclusive; a report outside it is
invalid and returns `422`. This simple bound matches the 30-second reporting interval and prevents
bad Agent clocks from distorting the 30-day window. Liveness always uses the Controller receive
time, never the Agent collection time.

If Controller is unavailable, Agent attempts the next scheduled report. Agent does not promise local
history, offline alerting, backfill, or exactly-once delivery.

When installed as a service, the Agent is not ready merely because its process started. It reports
readiness only after its current process has received an `accepted` or `duplicate` response from the
paired Controller. `duplicate` proves that the Controller already durably saw the same report, so it
is a successful delivery outcome for readiness. `stale`, `401`, other HTTP failures, malformed
responses, TLS failures, and network failures leave the Agent unready; later scheduled reports may
still establish readiness. The Agent sends the ready signal once and keeps no local report history.

## Alert settings

Monitoring has four global default settings:

- CPU usage threshold, default 90%;
- memory usage threshold, default 90%;
- disk usage threshold, default 90%;
- node offline duration, default 90 seconds.

Operators edit all four defaults in one configuration task and save them together.
Resource thresholds accept 1–100%; offline duration accepts 30–3600 seconds.
Invalid or missing values prevent submission, with user-readable validation messages.
Only permitted managers can save; read-only users retain disabled controls.

CPU, memory, and disk incidents use the same rule:

- three consecutive accepted samples at or above the threshold create one active incident;
- three consecutive accepted samples below the threshold resolve the active incident;
- a sample that breaks the sequence resets the corresponding counter;
- CPU, memory, and each disk mount are independent alert targets;
- one target can have at most one active incident.

Every node displays its effective alert settings and their source:

- a node without its own settings inherits the current global defaults;
- saving settings for a node creates one complete node override with higher priority than the
  global defaults;
- a node override is visibly marked as custom;
- resetting a node removes its override and immediately restores dynamic inheritance from the
  current global defaults;
- changing global defaults immediately affects every inheriting node, while custom nodes keep their
  own settings.

An offline incident becomes active when no newer accepted report has arrived within the configured
offline duration. The next valid newer report resolves it immediately.

Changing a threshold resets its unfinished consecutive-sample counters. Existing active incidents
are evaluated against the new threshold from the next accepted sample. Disabling an alert setting
resolves its active incidents with the reason `setting disabled`.

The user-visible incident states are only:

- `active`: the alert condition has been confirmed;
- `resolved`: the recovery condition has been confirmed, or the setting was disabled.

## Data retention

- Raw resource samples are retained for 30 days.
- Resolved incidents are retained for 30 days after resolution.
- Daily node summaries are retained for 30 days.
- Active incidents remain until they resolve.
- The latest state and registration of a managed node remain until that node is removed.

Cleanup must not delete the latest node state, an active incident, or unfinished alert counters.
Monitoring does not provide weekly, monthly, or yearly reports.

## User-visible areas

- Overview: online and offline node totals, active incidents, and latest resource summary.
- Nodes: latest node state, its current boot ID, and up to 30 days of CPU, memory, and per-mount disk history. Opening a node shows the same current boot ID and a 5-minute-bucket history chart. When points exist, the chart reserves a fixed-height plot area so the responsive chart is visible inside the Drawer. A single bucket renders visible CPU and memory markers; multiple buckets render their series lines.
- Alert incidents (告警事件, `/monitoring/incidents`): active incidents and resolved incidents still within retention; status and type
  selections apply immediately and return to page one.
- Nodes / Global settings: the four global default thresholds and whether each alert is enabled.
- Node details: the effective policy, its global/custom source, editing, and reset-to-default action.
- Daily summaries: per-node daily resource and incident summary within retention,
  browsed with pagination and without a node-ID search.

The four Monitoring read routes keep their current data visible while a normal
background refresh is in progress or fails, and offer a retry for the failed
refresh. An initial read failure has no retained data and is retryable. A `403`
is different from a temporary failure: the route must replace any previously
shown protected data with its permission state before offering a retry. Incidents
and Daily summaries use the fixed-size page contract; changing an Incident
status or type filter immediately requests page one. These route reads do not
retry automatically: the visible Reload or Retry action owns the next request,
so an initial or background `403` reaches its permission state after one call.
At the supported 390px viewport, Daily summaries must expose exactly the Date,
Node, and Coverage columns as readable table columns. Samples, CPU, Memory,
Disk mounts, Offline, and Incidents remain hidden; the visible header and first
data row must keep normal bounded row heights instead of stacking vertically,
and the table and pagination right edges must remain inside the viewport.

The finite local console closure is a source-bound Linux Chromium run with 23
browser cases against route-exact Monitoring read fixtures: four routes times
initial loading, `403`, initial `500`, after-success `403`, and after-success
`500`, plus Incident page two, Incident status/kind filtering at page one, and
populated Daily Summaries page two. After-success `403` must remove retained
protected data, while after-success `500` keeps it with Retry. The run also
records one desktop and one mobile theme/locale screenshot with no document
overflow. Its immutable
manifest binds fixture receipts, browser steps, staged binary provenance and
artifact hashes. This is local acceptance evidence only.

There is no checks page. Page layout, interaction details, responsive behavior, and visual states
belong to the UI specification rather than this product document.

## Non-goals

- Agent configuration synchronization or a bidirectional control channel.
- Agent-side database, alert engine, incident store, report engine, query handler, or Outbox.
- Custom or per-node alert rules, cooldown, hysteresis, notification delivery configuration, and alert escalation.
- Configurable network, service, process, log, or command checks.
- Weekly, monthly, yearly, APM, tracing, log-warehouse, or cloud-orchestration capabilities.
- Compatibility fallbacks for replaced Monitor or `rustzen-inspect` protocols and databases.

## Acceptance

The backend behavior is accepted when:

1. Multiple reports from one Agent update one node and append independent CPU, memory, and disk data.
2. Duplicate and older reports cannot refresh online time, add samples, or alter incidents.
3. Three consecutive violating samples create exactly one active incident per target.
4. Three consecutive normal samples resolve a resource incident; one valid report resolves an
   offline incident.
5. Eligible real disk partitions are reported once per local block device and evaluated
   independently.
6. A node without an override follows later global changes; a custom node remains unchanged.
7. Saving a node policy marks it custom; resetting it removes the override and restores the current
   global defaults.
8. The offline timer creates an incident after the effective configured duration without needing a
   report.
9. Cleanup enforces every 30-day retention rule without deleting current state or active incidents.
10. Restarting Controller preserves nodes, retained data, global defaults, node overrides, counters,
    and incidents.
11. No configurable-check, Agent-configuration, Agent-history, or legacy compatibility path remains.
12. An installed Agent reports system-service readiness only after an `accepted` or `duplicate`
    Controller response; credential and transport failures never fabricate readiness.

The Monitoring behavior is source-resolved. Representative browser coverage and its
limits are recorded in [local verification](../../../guides/local-verification.md);
that coverage does not certify every visual, permission, pagination, or deployment state.

The Controller and gateway share a database-pool default of one minimum and
eight maximum connections. Eight is the measured default for the certified
four-CPU, 512MiB pure-Monitor runtime profile; deployment-specific overrides
remain configuration, and must be justified by that deployment's own load
evidence.

The Linux dual-Agent gate starts two service identities with separate runtime roots,
logs, node IDs, and readiness sockets against one fresh central database. It requires
both Agents to remain unready while the Controller is unavailable, then verifies
accepted reports through Admin, two distinct current boot IDs, raw metric samples, and
continued reporting after the central processes restart without registering a third
node. The source-bound Agent cold build and the runtime scenario have independent,
bounded budgets: 900 seconds for the build (operator cap 1800) and 240 seconds for
the runtime (operator cap 600). A build timeout cannot consume or weaken the runtime
budget. The gate owns a named build container before compilation and removes it on
success, failure, timeout, or signal before accepting any Agent binary. It records only
source and binary identity plus non-secret runtime evidence.
The disposable container shares one kernel and does not boot systemd, provision a
remote host, or establish production TLS acceptance.

The separate dual-Agent Nodes browser gate reuses that topology with Admin, Monitor,
Reports, and two real Agent processes. Chromium opens the Nodes inventory and each
node detail, checks the node-ID-to-current-boot-ID mapping and the 5-minute history
surface, brings its CPU and memory markers into the viewport, and records ordered run
receipts plus one 1440x900 PNG for each detail with both markers visible. It is local container evidence for
the console projection of two real Agent reports; it does not prove systemd PID 1,
multi-host networking, production TLS, or the complete Monitoring visual matrix.

## Node onboarding

Nodes exposes Add node to monitor:manage users. Production onboarding is an offline,
signed installation flow: apply the signed node-agent archive with its manifest,
envelope, trusted public key and key ID; create the fixed service identity; prepare
access; pin the signed Controller tuple; provision a root-only four-key source file;
then activate the selected unit. The console does not receive the archive paths or
signature inputs, so it must show these actionable prerequisites and must not generate
a direct `rz-monitor-agent` command. A token is never rendered, copied, logged, or
placed in an argv value. Production Controller endpoints are HTTPS-only; HTTP is
limited to loopback development configurations.

The first accepted Agent report registers a node. Viewing prerequisites does not
register an empty node. Connection actions remain unavailable until the signed inputs
and local secret boundary are integrated. Agent startup, authentication and reporting
remain the existing contract.

Global settings is part of Nodes, with no independent menu/page. Reading requires
monitor:node:view; saving requires monitor:manage. The four defaults share one
Save, and successful saves invalidate Monitoring queries including node policies.
If a global or node-policy save/reset cannot reach the service, the open Drawer
shows an actionable failure while retaining the current form values. HTTP and
business rejections retain the request layer's existing single global error
message; the Drawer does not add a second toast for the same rejection.

Nodes refresh in the background every 30 seconds. Each inventory refresh reads node
state and the matching latest disk samples in one SQLite read transaction, using a
bounded number of indexed Controller queries; it does not issue one disk-sample query
per node or combine node state from one report with disks from another. Controller may
reuse one successful serialized immutable Nodes response snapshot for at most 250ms. Concurrent cache misses
share one refresh; accepted Agent reports and successful global/node policy writes or
resets invalidate it before returning. Failed reads are never cached. Admin still checks
the authoritative session and current permission on every gateway request. A failed
background refresh
retains the last successful inventory and exposes its update time, Retry, and an
explicit refresh action. An initial load failure continues to use the blocking
retry state because no inventory is available to retain.
