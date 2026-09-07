# Message Center UI

Status: P5a backend contract implemented and verified locally; bell, inbox page
and realtime shell are not implemented in this slice.

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
  current user revision and the configured 30-day retention copy. P5a exposes
  this policy metadata; P5b owns enforcement and capacity admission.
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

## Deferred UI and realtime work

The bell, unread badge, inbox list/detail presentation, retention explanation,
incident/run navigation, responsive states, fetch-SSE parser and reconnect
behavior remain P5b/P7. Until those slices land, backend success is not visual
acceptance. Monitor and Reports producer delivery remains P6 and does not block
their existing incident or run behavior.
