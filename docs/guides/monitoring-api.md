# Monitoring API

All paths are registered by the Monitor Rust `ModuleRouter`. Responses use the repository module
response envelope. Frontend JSON fields use `camelCase`.

## Agent report

### `POST /api/monitor/agent-reports`

Public module route. It still requires the exact `x-rustzen-monitor-agent-token` header. The Admin
gateway forwards it to the loopback Monitor Controller.

Request:

```json
{
  "nodeId": "edge-a",
  "bootId": "3f70ea4d-6e16-40ca-b78c-7cb54fc27d43",
  "sequence": 42,
  "hostname": "edge-a.example",
  "agentVersion": "0.5.0",
  "collectedAt": "2026-09-02T08:00:00Z",
  "cpuPercent": 21.5,
  "memory": { "usedBytes": 8589934592, "totalBytes": 17179869184 },
  "disks": [
    { "mountPoint": "/", "usedBytes": 53687091200, "totalBytes": 107374182400 },
    { "mountPoint": "/data", "usedBytes": 214748364800, "totalBytes": 536870912000 }
  ]
}
```

Rules:

- `nodeId` is a stable 1-128 character identifier containing letters, digits, `.`, `_`, or `-`.
- `bootId` is generated once per Agent process and `sequence` starts at one.
- CPU is finite and between 0 and 100.
- totals are positive and used bytes do not exceed totals.
- mount points are non-empty, unique inside one report, and limited to 256 characters.
- `collectedAt` must be strictly later than the node's latest accepted report for the same or a new
  boot; an equal or older report returns `stale` with no mutation.
- `collectedAt` must be within five minutes (inclusive) of Controller receive time. A report outside
  that allowed clock skew is invalid and returns `422`. Online state uses receive time only.

Success data:

```json
{ "status": "accepted" }
```

Fencing is evaluated before clock-skew validation. An exact replay returns `duplicate`; an older
sequence, retired boot, or invalid boot takeover
returns `stale`. These statuses produce no monitoring-state mutation. Missing or wrong Agent token
returns `401`; malformed or invalid input returns `422`.

## Overview

### `GET /api/monitor/overview`

Permission: `monitor:overview:view`.

Returns registered, online and offline node totals, active Incident total, and the most recent
resource summary. It contains no check health fields.

## Nodes

### `GET /api/monitor/nodes`

Permission: `monitor:node:view`. Returns latest state for every managed node.

### `GET /api/monitor/nodes/{nodeId}`

Permission: `monitor:node:view`. Returns one node or `404`.

### `GET /api/monitor/nodes/{nodeId}/metrics`

Permission: `monitor:node:view`.

Query fields:

- `from`, `to`: RFC3339 timestamps; default is the latest 24 hours and maximum range is 30 days;
- `bucket`: `raw`, `5m`, or `1h`.

Returns aligned CPU and memory points plus independent disk series identified by mount point. An
unknown node returns `404`; an invalid or over-30-day window returns `422`.

### `GET /api/monitor/nodes/{nodeId}/alert-settings`

Permission: `monitor:node:view`. Returns the node's effective settings with
`source=global|custom` and `isCustom`. A global source is resolved dynamically rather than copied
onto the node.

### `PUT /api/monitor/nodes/{nodeId}/alert-settings`

Permission: `monitor:manage`. Accepts the same four setting groups as the global update and saves
one complete custom node policy.

### `DELETE /api/monitor/nodes/{nodeId}/alert-settings`

Permission: `monitor:manage`. Deletes the custom policy and returns the current effective global
defaults. Repeating reset is idempotent.

## Incidents

### `GET /api/monitor/incidents`

Permission: `monitor:incident:view`.

Filters: pagination, `status=active|resolved`, `kind=cpuHigh|memoryHigh|diskHigh|nodeOffline`,
`nodeId`, `from`, and `to`. The maximum history window is 30 days.

### `GET /api/monitor/incidents/{incidentId}`

Permission: `monitor:incident:view`. Returns the Incident, node context, target, threshold,
observed values, opened time, last observed time, optional resolution time, and resolution reason.
There is no acknowledged state or mutation endpoint.

## Alert settings

### `GET /api/monitor/alert-settings`

Permission: `monitor:overview:view`.

### `PUT /api/monitor/alert-settings`

Permission: `monitor:manage`.

Request and response data:

```json
{
  "cpu": { "enabled": true, "thresholdPercent": 90 },
  "memory": { "enabled": true, "thresholdPercent": 90 },
  "disk": { "enabled": true, "thresholdPercent": 90 },
  "offline": { "enabled": true, "afterSeconds": 90 },
  "updatedAt": "2026-09-02T08:00:00Z",
  "source": "global",
  "isCustom": false
}
```

Thresholds are between 1 and 100. Offline duration is between 30 and 3600 seconds. The fixed three
sample trigger and recovery counts are not configurable. PUT requests omit `updatedAt`, `source`,
and `isCustom`. Global changes affect only nodes that do not have a custom policy.

## Daily summaries

### `GET /api/monitor/daily-summaries`

Permission: `monitor:node:view`.

Filters: pagination, optional `nodeId`, `from`, and `to`. The maximum range is 30 days. Each row
contains the date, node, sample count, coverage, CPU/memory minimum-average-maximum, per-mount disk
minimum-average-maximum, offline duration, and Incident counts.

## Removed APIs

`/heartbeat`, every `/checks` route, check results, check testing, and check enable/disable routes
are removed. No compatibility aliases are registered.
