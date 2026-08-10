# Monitoring Incident Diagnostics

## Goal and implementation slice

Give an operator a trustworthy read-only path from a Monitor signal to the
incident evidence behind it. The slice exposes the incidents that the existing
Monitor evaluator already stores, without adding acknowledgement, alert
policy editing, notifications, or a second monitoring engine.

The current source evaluates node and resource conditions and persists
`open` and `resolved` rows from the evaluator; `acknowledged` remains
schema-compatible for historical rows but is not written or transitioned by
the current evaluator. The overview currently uses only the active count. This
slice makes the stored lifecycle inspectable; it does not change how incidents
are detected.

## Users and scenarios

- An owner or permitted operator opens Monitoring and can see whether the
  active count is backed by concrete incident rows.
- An operator filters incidents by status, source, and time window, then opens
  one record to understand the node/check/resource context and last observed
  time.
- A viewer can inspect the same read-only evidence but cannot acknowledge,
  resolve, suppress, or edit an incident.
- When Monitor is unavailable, Admin remains usable and the page explains that
  incident data is unavailable rather than presenting a healthy empty list.

## Confirmed decisions and rationale

| Decision | Rationale | Acceptance consequence |
| --- | --- | --- |
| Read-only list and detail are the first slice. | The evaluator and lifecycle are already present; visibility is the smallest useful gap. | List/detail can be verified without changing detection or state transitions. |
| Current evaluator lifecycle is `open` to `resolved`; `acknowledged` is read-only schema compatibility. | The evaluator opens or updates active rows and resolves them; historical/imported acknowledged rows may remain. | The UI labels observed statuses, never writes acknowledgement, and never invents a new lifecycle. |
| `monitor:incident:view` is the target read boundary. | Incident evidence is more specific than the overview count. | Route registration, Manifest, gateway, and UI guard must agree on this boundary. |
| A missing detail record is a business error, not an empty result. | An incident selected from a list must remain diagnosable. | The detail view shows an error and retry or return action. |
| No notifications or policy editing. | Those behaviors change ownership, delivery, and failure semantics. | They remain separate future specifications. |

## Scope and non-goals

In scope:

- a paginated incident list reachable from Monitoring overview;
- filtering by status, source type, source identifier, and bounded time range;
- a detail view or drawer containing title, status, source, opened/last-observed
  times, resolved time when present, and structured evidence rendered as
  readable data;
- links from a node or check context to the relevant incident evidence;
- localized empty, error, permission, and retry states;
- read-only rendering of a historical `acknowledged` row when present, without
  an acknowledge action or transition;
- read-only audit visibility of the incident query, if the existing operation
  log boundary records the request.

Non-goals:

- acknowledgement, resolution, suppression, deletion, or manual incident
  creation;
- alert policy editing, threshold editing, notifications, escalation, or
  incident comments;
- a log warehouse, tracing system, APM, or cross-database query;
- changing evaluator cadence, thresholds, deduplication, or persistence shape;
- adding a new server process or copying the former standalone Monitor shell.

## Main and failure flows

1. The operator enters a permitted Monitoring surface. The list query enters
   `loading` and keeps the current page shell stable.
2. A successful response renders `populated` rows, or `empty` only when the
   query succeeded with no matching incidents.
3. Selecting a row loads detail without losing the list filter. A successful
   detail response shows the stored evidence and lifecycle timestamps.
4. A list or detail failure shows an explicit error and retry. A permission
   denial is labeled separately and never becomes an empty table.
5. If a bounded response contains rows whose detail cannot be read, the list
   remains visible and the affected row is marked `partial`; the detail reason
   is actionable and the page does not claim a complete evidence set.

## Business rules and permissions

- `monitor:incident:view` gates list and detail reads. The backend remains the
  authorization boundary; hiding a button is not authorization.
- Owner and admin roles receive the capability through the normal concrete
  module policy; viewer receives it only as a read capability. Custom-role
  assignment follows the existing capability catalog rules.
- Status values come from Monitor persistence. The current evaluator emits only
  `open` and `resolved`; `acknowledged` is accepted only as read-only schema
  compatibility for historical rows. The UI may localize labels but must
  retain the stable status code in accessible text or data attributes when
  needed for diagnostics.
- A detail request must not mutate incident status. Any future mutation gets a
  separate permission and feature specification.
- Structured evidence is presented as data, not executable markup. Raw service
  errors are logged for operators but are not the only user-facing copy.

## UI states and evidence

The UI contract is [Monitoring Incident Diagnostics UI](../../../ui/features/monitor-incident-diagnostics.md).
It reuses the accepted root `DESIGN.md` and current Ant Design surfaces.

| State | User-visible meaning | Required behavior |
| --- | --- | --- |
| Loading | Query is in progress. | Keep filters and shell; do not show an empty result. |
| Populated | At least one matching incident is readable. | Keep status, source, and time visible; allow detail. |
| Empty | Query succeeded with zero matches. | Explain the active filters and offer a clear reset. |
| Error | Query failed. | Show retry; preserve filters and last successful rows when available. |
| Permission | The caller lacks incident-read access. | Explain authorization; no retry loop that can leak data. |
| Partial | Some rows or evidence are unavailable. | Identify affected records and preserve readable rows. |

## User-visible data effects

The slice is read-only and creates no new persisted data. It exposes existing
Monitor incident records and may add an ordinary access-log entry if the
existing Admin audit policy covers the request. No Admin database joins Monitor
tables, and no response includes credentials or executable content.

## Affected product surfaces and dependencies

- Monitor owns incident query behavior, persistence, status meanings, and
  response data.
- Admin owns delegation, capability reconciliation, and Web hosting.
- `apps/web` owns the Monitoring overview/node composition and module API client.
- Monitor HTTP contract chain is `Rust ModuleRouter/Manifest -> handwritten
  apps/web/src/api/monitor/contract.ts -> scripts/verify-worker-contracts.mjs`.
  It does not use the Admin OpenAPI/Orval chain.
- Existing `PageHeader`/`PageCard`, `DataState`, `DataTableShell`/route-local
  `ProTable`, `Drawer`, and `formatDateTime` are the visual/interaction owners.

## Acceptance criteria

- A permitted operator can load a paginated incident list and open a concrete
  detail record without leaving Monitoring.
- Status, source, title, observed time, and resolved time (when present) are
  distinguishable for every populated row/detail.
- A successful zero-row response is visibly different from request failure,
  permission denial, and unavailable Monitor service.
- Retry preserves filters and the selected row context where a retry is safe.
- No UI action or backend request acknowledges, resolves, suppresses, deletes,
  or creates an incident in this slice.
- Current evaluator tests show only `open`/`resolved` transitions; a historical
  `acknowledged` row, if present, renders read-only and cannot transition.
- The target capability is enforced by the backend route and is present in the
  Monitor handwritten contract verified through
  `scripts/verify-worker-contracts.mjs` after implementation.
- Node/check context can reach the corresponding incident evidence without
  duplicating the incident data model in the frontend.
- All fixed copy has Simplified Chinese and English variants; runtime incident
  titles and details remain unchanged.
- The UI acceptance matrix in the linked UI contract is exercised at the
  required desktop and narrow viewports, including loading, empty, error,
  permission, populated, and partial states.

## Verification matrix

| Layer | Evidence | Acceptance |
| --- | --- | --- |
| Source/static | route registration, capability, DTO/client, and state derivation review | No second route catalog; no mutation path; status codes map one-to-one. |
| Automated | Monitor unit/service tests plus contract verification | List/detail serialization, pagination, filters, and permission rejection pass. |
| HTTP | Real module request through Admin delegation | Owner/admin/viewer behavior and module-unavailable error are distinct. |
| Browser | Required UI viewport/state matrix | List/detail, retry, empty, permission, partial, focus, wrapping, and no overflow pass. |
| Runtime/deployment | installed four-service bundle | `Not verified` until the release topology is started and exercised. |

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- The existing Monitor incident rows remain the source of truth for the first
  read-only slice.
- Time filters use the installation display timezone already used by the Web
  shell; the stored timestamps remain unmodified UTC values.

### Open questions

- None block this read-only slice. Whether users need saved filters or incident
  comments is outside the current acceptance boundary.

### Rejected

- Treating the active count as sufficient incident diagnosis.
- Treating a failed Monitor query as a valid empty list.
- Adding acknowledgement or notifications to make the first read surface look
  complete.

### Deferred

- Alert policy management, notification delivery, escalation, comments, and
  incident retention administration.
- Cross-node correlation, trace links, and a searchable log warehouse.

## Ready for monitor incident diagnostics implementation

The product behavior, permission boundary, lifecycle interpretation, failure
semantics, non-goals, ownership, and acceptance are fixed. The linked UI
contract is ready for frontend implementation. Browser, HTTP, deployment, and
external-consumer evidence remain `Not verified` until the implementation and
validation owners exercise them.
