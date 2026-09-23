# Admin maintenance task console

## Scope and authority

This specification covers the existing Admin task console at `/manage/task` and
its Admin-owned SQLite records. Rust route registration, task service and the
generated Web client remain the implementation authority. It defines a bounded
maintenance surface, not a generic automation product.

The catalog contains exactly these built-ins:

| Task key | Operator meaning | Schedule |
| --- | --- | --- |
| `cleanup-operation-logs-retention` | Remove operation logs older than the configured retention period. | Daily 01:20 UTC cron |
| `cleanup-task-runs-retention` | Remove old maintenance-task run records. | Daily 01:30 UTC cron |
| `sqlite-storage-maintenance` | Run WAL checkpoint, query-planner optimization, and reusable-page reclamation. | Daily 02:00 UTC cron |

Cron editing, task enable/disable controls, external task registration, generic
workflow orchestration, and Reports schedule management are excluded.

## Access and state contract

`manage:task:list` is assignable read-only and may list the three task summaries and each task's run records.
`manage:task:run` may submit a manual run. Owner has both capabilities; a
list-only custom role can read the same task and record data but a run `POST`
returns the standard 403 envelope. The Web action is hidden for list-only users.

Each task exposes its fixed schedule, current running state, last completed
result, next run, and error when present. A submitted manual run first becomes
`running`, then reaches `success` or `failed`; the records dialog refreshes
while a displayed run is active. A manual submission while the same task is
running is rejected as a business conflict. A scheduled overlap creates a
`skipped` record and does not execute concurrently. On Admin startup, stale
`running` records left by a stopped process are marked `failed` with a recovery
reason before the three built-ins are synchronized.

## Acceptance matrix

| ID | Scenario | Expected result |
| --- | --- | --- |
| ATC-01 | Fresh Admin SQLite startup | Exactly the three fixed built-ins are present. |
| ATC-02 | Owner list and records requests | Three task rows and the selected task's paged records are readable. |
| ATC-03 | Owner runs operation-log cleanup | API reports `running`, then its record reaches `success`. |
| ATC-04 | Same-task overlap | Manual request is rejected; scheduled request is recorded as `skipped`. |
| ATC-05 | Startup after interrupted run | Existing `running` record and task summary become `failed` with recovery detail. |
| ATC-06 | List-only role | GET remains visible; run POST returns 403 and no run record is added. |
| ATC-07 | Web query states | Loading, empty, and error states remain distinct; retry is available for errors. |
| ATC-08 | Narrow viewport | The table owns horizontal overflow and controls remain reachable without document overflow. |

The disposable Linux Chromium gate is the local runtime evidence owner. It
uses a fresh database and preserves only redacted, task-scoped failed-run
diagnostics; its `current` link changes only after receipt validation and owned
container cleanup. Native systemd, deployment, and production evidence remain
`Not verified`.
