# Message Center UI

Status: P5a and P5b durable-inbox backend contracts are implemented and verified
locally. The P7 message-center and realtime contract below is frozen. P7a adds
only the Access session authority required by that contract; SSE and Web remain
pending P7b and P7c and have no implemented UI acceptance yet.

## Product boundary

The message center is an optional Admin capability. Its durable personal inbox
is the source of truth. A future SSE stream may only invalidate cached inbox
state; it never carries message bodies or replaces reconciliation. No fifth
resident service, email, SMS, mobile push, webhook or generic workflow is part
of this feature.

When notifications are omitted from a composition, its Admin schema, API
routes, Web roots, timers and service startup contain no message-center owner.
P5a adds only the selected backend schema and authenticated read APIs. It does
not add a hidden bell, empty placeholder, polling task or disconnected Web
route before the Web slice is implemented.

## P5a API-owned states

- Loading, empty, populated and error states will be driven by the durable list
  response. The response carries an authenticated-encrypted opaque snapshot
  boundary, next cursor,
  current user revision and the configured 30-day retention copy. P5b enforces
  retention by acceptance time, including unread messages; history that has
  expired is no longer available during the interval before physical cleanup.
- Unread count and list/detail/read operations infer the current user from the
  authenticated request. There is no client-provided user ID.
- Revoked grants, a disabled producer module or an inactive user take effect on
  the next database snapshot. Detail and single-read return not found when the
  message is no longer accessible, without exposing its ID or count.
- Single read preserves the first read time. Read-all applies only to accessible
  unread rows at or below the preceding signed snapshot boundary; a concurrent
  later arrival remains unread. User revision changes once only when visible
  inbox state changes.
- Cursors are opaque and bind user, unread filter, snapshot maximum sequence and
  page position. Their external bytes expose none of those fields. Invalid,
  out-of-range or cross-user query state returns the JSON 400 envelope without
  inbox data. Notification response items never expose the internal sequence.

## P7 composition and shell ownership

The message-center Web root belongs to `notifications`. A selected build mounts
its shell contribution beside the common layout; the common `BaseLayout` must
not statically import the bell, inbox client, SSE parser or notification route.
A composition without notifications therefore has no bell placeholder, hidden
route, client chunk, API owner, connection task or configuration lookup.

The selected desktop header places one icon-only bell action with an accessible
name before the existing theme/account actions. The unread badge displays
`1..99` and `99+`; zero hides it. Activating the bell opens an Ant Design Drawer
that preserves keyboard focus, returns focus to the bell on close, and uses the
shared 64 px desktop / 56 px narrow shell offsets. The Drawer contains the
durable list, unread filter, paging, detail and read actions. It does not render
HTML from a producer and does not accept a producer-supplied URL.

## P7 state matrix

| State | Presentation | Allowed action / transition |
| --- | --- | --- |
| Closed | Bell and current unread badge only | Open registers one client lifecycle and reconciles the durable count/list. |
| Initial loading | Drawer skeleton with title retained | Close remains available; no stale row action is enabled. |
| Empty | “No messages” with the active filter named | Switch all/unread or close. |
| Populated | Newest-first rows with producer, safe title/summary, occurred time and read state | Open detail; request the next opaque page when present. |
| Unread filter | Only server-authorized unread rows from its own snapshot | Filter change discards cursor/pages and starts one fresh query. |
| Page loading | Existing rows remain visible with one bounded loading affordance | Repeated load-more is disabled until the request settles. |
| Detail loading | Drawer context stays visible; detail body is skeleton-only | 404/403 closes the detail without retaining protected content. |
| Detail ready | Plain text fields and fixed subject action, if supported | Mark this message read; navigate only through the fixed mapping below. |
| Marking one read | Row action and duplicate requests disabled | Success applies the returned first-read state, then reconciles count. |
| Marking all read | One pending control scoped to the preceding server snapshot | Later arrivals remain unread; success reconciles list and count. |
| Retention gap | 30-day retention explanation beside empty/end state | No recovery or “load expired” action is offered. |
| Recoverable HTTP/network error | Inline retry without clearing already reconciled rows | Retry is single-flight with the current auth generation. |
| 401 / expired session | Drawer content is cleared | Stop SSE/retries and enter the existing login flow. |
| 403 / module unavailable | No protected row or existence detail remains | Show the existing unavailable/forbidden state; do not retry rapidly. |
| SSE connected | No extra success chrome | Advisory revision invalidates cached count/list only. |
| SSE reconnecting | Subtle connection status; durable rows remain | Exponential retry from 1 to 30 seconds with jitter; one connection attempt. |
| Lag / reconcile required | Connection closes after the advisory | Fetch durable count/list before reconnecting. |
| Hidden / frozen | Timers and stream may stop | On visible/pageshow, discard stale generation, reconcile, then subscribe once. |
| Logout / auth-store change / pagehide | No message-center work remains | Abort fetch, parser, retry timer and stream; BFCache restore starts fresh. |

Safe subject navigation is a closed mapping: Monitor incident messages may open
`/monitoring/incidents?incidentId=<encoded-id>` and Reports run messages may open
`/reports/runs?runId=<encoded-id>`. The client ignores every URL-like payload
field. The Monitor route must resolve that exact incident through its authorized
API and treat 403/404 as inaccessible without falling back to another incident.
Unknown producer/topic/subject combinations have no navigation action.

## P7 realtime contract (pending P7b/P7c)

Admin will expose one direct authenticated fetch-SSE endpoint. It emits an
initial revision hint, advisory revision invalidations, 15-second heartbeats and
`reconcile.required`; it never emits message bodies and never replays missed
business events. `Last-Event-ID` is only a client continuity hint, so reconnect
always reconciles the durable inbox. Server limits are 4 connections per user,
1,000 overall and a queue of 16 per connection. A full queue sends
`reconcile.required` when possible and closes while releasing both quotas.

The server rechecks JWT expiry before every frame and revalidates the current
session, enabled user and policy at least every five seconds. Logout, revocation,
permission/module change, expiry or database failure closes the stream. The Web
parser accepts split UTF-8, CRLF, comments and multiline data while bounding a
line to 8 KiB and one event to 16 KiB. No JWT appears in a URL. Visible tabs also
reconcile every 60 seconds. The 1,000-connection and 24-hour behavior is a P8
runtime gate and is not claimed by this frozen UI contract.

## Existing durable behavior

P5b capacity or storage-pressure refusal is an operator diagnostic and does not
create a user-facing partial message. Existing inbox rows remain readable while
new notification admission is paused. Duplicate producer retries remain
idempotent, and the UI never infers delivery from an event that was rejected.

Until P7b/P7c land, backend success is not realtime or visual acceptance.
Monitor and Reports producer delivery remains P6 and does not depend on a
browser connection.
