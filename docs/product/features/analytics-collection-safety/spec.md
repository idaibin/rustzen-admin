# Analytics Collection Safety

## Goal and implementation slice

Make product-event collection an explicit, bounded, and privacy-conscious
choice. A host must opt in before the tracker sends events; the Insights
service must verify the public project routing identifier and browser origin
before accepting them;
the payload must describe a pathname-based event rather than copied page text,
query strings, or arbitrary element properties.

The existing Analytics overview and detail journey remains single-project and
instance-wide. This slice hardens collection and keeps query/read behavior
under the existing Insights owner; it is not a multi-tenant analytics
platform. A host may load the public tracker asset, but the tracker remains
lazy and inert until the host explicitly enables it with consent.

## Users and scenarios

- An installation owner enables tracking only for an explicitly approved
  application origin and project key.
- A visitor who has not opted in produces no tracker request and receives no
  persistent tracking identifier from this product.
- A permitted operator can distinguish accepted event data from a collection
  rejection when inspecting Analytics health or diagnostics.
- A viewer can inspect retained overview/detail data but cannot change the
  collection policy or credentials.
- A malformed, oversized, unknown, expired, or origin-mismatched event is
  rejected without being silently stored or transformed into an accepted
  event.
- A project routing identifier is safe to expose in a host page. It selects a
  registered project and origin policy; it is not a credential, does not grant
  query or management access, and is not treated as proof that a visitor
  consented.

## Confirmed decisions and rationale

| Decision | Rationale | Acceptance consequence |
| --- | --- | --- |
| Tracking is opt-in and disabled until the host page explicitly enables it. | Collection must be a deliberate installation decision. | The tracker may be loaded as an inert asset, but it does not initialize, patch requests, send a request, or create visitor/session IDs before the enable entry. |
| Installation policy and visitor opt-in are separate boundaries. | The installation owner controls whether this installation accepts collection; the host controls whether a visitor opted in. | Insights accepts HTTP ingestion only when `collection_enabled`, `project`, and normalized `origin` match the installation policy; it never claims to verify visitor consent. |
| The project key is a public routing identifier, not a credential. | It must be present in a browser integration without pretending to protect a secret. | The server uses identifier + normalized source origin to select policy; it never grants authenticated read/manage access from the identifier. |
| The server verifies the identifier and configured origin. | Browser code alone cannot enforce an ingestion boundary. | Missing, invalid, or origin-mismatched requests are rejected. |
| Page and API paths use `pathname` only. | Query values can contain identifiers or secrets. | Stored paths never include a query string. |
| Built-in events and custom properties use a strict allowlist. | Button text and arbitrary DOM attributes are unstable and privacy risky. | Unknown event names, fields, and property keys are rejected; no free-form DOM scrape is accepted. |
| Visitor and session identifiers are pseudonymous and short-lived by default. | Useful retention does not require a person's name or account value. | IDs are generated only after opt-in, roll on a fixed schedule, and are removed immediately on opt-out. |
| Batch validation and capacity protection are all-or-none. | A partial accept count or an unbounded queue is easy to misread as complete collection. | 413, 429, and storage-protection responses persist zero events from the rejected batch. |

## Public identifier, opt-in, and safety limits

The project key is a **public routing identifier**. It may appear in the host
page and in a browser request. It is matched to a registered project and an
exact normalized source origin; it is not a password, bearer token, consent
receipt, or permission grant. The server never claims that receiving the key
proves visitor consent.

The installation policy is owned by Insights and has three first-slice fields:

- `collection_enabled`: whether this installation accepts public collection;
- `project`: the public routing identifier registered for the host integration;
- `origin`: the exact normalized browser origin allowed for that project.

These fields are the server boundary. A request is accepted only when all three
match the installation policy; the public `project` value is not a credential.
Origin normalization lowercases the scheme/host, removes a trailing slash, and
omits default `http:80`/`https:443` ports on both policy and ingestion paths.
The original 0001 development seed is cleared by migration and does not count
as `projectConfigured`; the first enable operation must provide a new project
key through the authenticated collection-policy API. Disabling collection may
retain the registered hash, but enabling an unconfigured installation cannot.

Cross-origin browser collection uses the same installation policy for CORS. A
preflight `OPTIONS /api/insights/track` succeeds only for an enabled,
configured installation whose normalized `Origin` is allowlisted; it returns
`204` with the exact origin, `POST`, and the two collection request headers
(`content-type`, `x-rustzen-project-key`) plus `Vary: Origin`. There is no
wildcard origin. A `POST` echoes the exact verified origin only after project
key and origin validation, including validation or other business errors after
that boundary; denied origins receive no allow headers. Same-origin requests
keep the existing ingestion behavior and no CORS policy is inferred client
side.

Visitor opt-in is a separate host boundary. The host application shell or
deployment integration owns a bootstrap loader and its explicit entry. Its
order is fixed:

1. The host invokes the bootstrap entry and reads the host's visitor-consent
   result.
2. If consent is absent or denied, the bootstrap does not call the tracker's
   enable entry. A previously loaded asset stays inert: no queue, request
   patch, network request, or visitor/session identifier is created.
3. If consent is granted, the bootstrap loads the tracker if needed and calls
   its enable entry exactly once with the public `project` value; the tracker
   then emits only the approved payload.
4. Insights validates `collection_enabled`, `project`, and normalized `origin`
   on every HTTP ingestion request. It does not receive or verify visitor
   consent, and a successful HTTP response is not consent evidence.

An explicit opt-out immediately stops queued sends and removes the local
visitor and session identifiers. Browser evidence verifies the consent and
injection order; HTTP evidence verifies only the installation policy boundary.

The first release uses conservative limits. Body, batch, storage, and
free-disk values are code-owned hard fail-closed maxima; the current settings
API does not expose them as operator configuration. The per-project/source
request and event counters are process-local Insights runtime admission
guards: they reset when the Insights process restarts and are not cross-restart
hard maxima or durable quotas. A future lower-cap setting would require an
explicit contract and migration; no client or operator UI may imply that these
values are configurable today:

| Boundary | Initial value | Configuration rule | Rejection and persistence |
| --- | ---: | --- | --- |
| JSON request body | 64 KiB | Fixed code hard cap; no current setting | HTTP 413; the whole batch persists zero events. |
| Events per batch | 50 | Fixed code hard cap; no current setting | HTTP 413; the whole batch persists zero events. |
| Requests per project/source per minute | 30 | Process-local runtime guard; resets on Insights restart; no current setting | HTTP 429 while the process-local window is exhausted; the whole batch persists zero events. |
| Events per project/source per minute | 300 | Process-local runtime guard; resets on Insights restart; no current setting | HTTP 429 while the process-local window is exhausted; the whole batch persists zero events. |
| Insights event storage budget | 256 MiB | Fixed code hard cap; no current setting | HTTP 507 when the projected write exceeds the budget or disk reserve; the whole batch persists zero events. |
| Free-disk reserve for the Insights runtime path | 512 MiB | Fixed code reserve; no current setting | HTTP 507; the whole batch persists zero events. |

`source` is the normalized `Origin` tuple (scheme, host, and explicit port when
present). Rate accounting is keyed by `{public project identifier, source}`;
there is no global bucket that would let one origin starve another. The storage
budget measures the Insights database plus WAL/SHM sidecars. A preflight check
must fail closed before opening a write transaction when a limit is exceeded.
Malformed or unknown fields remain a validation error and are never counted as
accepted data.

## Scope and non-goals

In scope:

- host bootstrap enable entry, visitor opt-in/opt-out behavior, and lazy
  initialization/injection ordering;
- server-side `collection_enabled`, public project-routing-identifier, and
  allowed-origin verification;
- pathname-only page and API path normalization;
- a fixed event vocabulary for page views, API requests, and approved custom
  events;
- allowlisted property names and bounded payload size, batch size, timestamp,
  status, and duration values;
- default pseudonymous visitor/session identifiers with a rolling local
  lifetime and an explicit boundary for optional user identification;
- fixed body, batch, storage-budget, and free-disk protection with fail-closed
  rejection, plus process-local per-project/source rate admission guards;
- clear ingestion rejection semantics and retained Insights overview/detail
  compatibility;
- configuration and query access through existing Insights ownership and
  capabilities.

Non-goals:

- multi-project tenancy, project lifecycle, or a project selector;
- marketing automation, user profiling, advertising identifiers, or a general
  warehouse/export product;
- capturing button text, form values, arbitrary DOM attributes, URL queries,
  hashes, request bodies, headers, or cookies;
- adding a new analytics service, database, SDK dependency, or Admin shell;
- making a collection rejection look like a successful zero-event overview;
- legal/privacy policy authoring beyond the product-level opt-in boundary.

## Main and failure flows

1. The installation owner sets `collection_enabled`, registers a public
   project identifier, and approves the normalized source origin. Separately,
   the host bootstrap reads visitor opt-in; before consent the tracker may be
   loaded as an inert asset, but no initialization, request patch, request, or
   IDs exist.
2. The tracker emits only approved event names, pathname-only location fields,
   pseudonymous IDs, and allowlisted bounded properties. Visitor IDs roll after
   24 hours. Session state stores `createdAt` and `lastSeenAt` separately and
   rotates after 30 minutes idle or 24 hours absolute, whichever comes first.
   Server retention is 30 days in the first release.
3. Insights verifies the installation policy (`collection_enabled`, project,
   and origin), serves only a policy-approved CORS preflight, then validates
   shape, body/batch limits, process-local per-project/source rate admission,
   storage budget, disk reserve, timestamp, and retention before persisting an
   entire batch. Rate windows belong to the running Insights process and reset
   on restart; body, batch, storage, and disk caps remain fail-closed. It does
   not assert that the visitor consented.
4. A valid batch is stored and becomes visible through the existing overview or
   detail query. A successful zero-event query is a genuine empty state.
5. A rejected oversized/capacity batch returns 413, a rate-limited batch
   returns 429, and a storage/disk-protected batch returns 507; each response
   persists zero events from that batch. The query surfaces keep prior
   successful data and do not turn rejection into empty data.
6. A rate or storage response with an explicit all-or-nothing contract (429 or
   507) retries only within the tracker transport boundary with a bounded
   attempt count and backoff. A 5xx response or network error is ambiguous for
   a non-idempotent batch and is dropped once without retry. Every drop (and
   every exhausted 429/507 retry budget) is reported by the transport hook and
   is not requeued. Opt-out clears pending data and local IDs immediately.

## Business rules and permissions

- Public collection has no authenticated Admin capability. The host bootstrap
  enforces visitor opt-in before tracker initialization; the server enforces
  `collection_enabled`, public project identifier, origin, schema, body/batch
  and rate bounds, storage/disk protection, and retention before persistence.
- `insights:overview:view` and `insights:event:view` gate Analytics reads.
  Collection settings and allowed-origin management use `insights:manage` and
  remain unavailable to viewer-only roles.
- The default visitor and session IDs are pseudonymous and short-lived. Visitor
  IDs use a 24-hour creation lifetime. Session state stores separate
  `createdAt` and `lastSeenAt` timestamps and rotates at the first of a 30-minute
  idle timeout or 24-hour absolute lifetime. An
  optional user ID is
  accepted only when the host has separately enabled that field and the value
  is a stable internal identifier, not a display name or free-form secret.
- The server never interprets a project identifier or event field as consent
  proof. Consent evidence remains the host/deployment owner's responsibility.
- Event names, property keys, path fields, status codes, and durations are
  bounded. The service remains authoritative for normalization, rate limits,
  storage budget, disk reserve, and retention. The 30-request/300-event
  per-project/source windows are process-local runtime guards and reset on an
  Insights restart; they are not durable cross-restart quotas.
- Opting out stops new collection and removes local visitor/session IDs;
  existing retained events follow the normal retention policy and are not
  silently rewritten as if they never existed.
- Collection settings are owned by Insights. Admin may authorize and display
  them but does not read or join Insights tables directly.

## UI states and evidence

The UI contract is [Analytics Collection Safety UI](../../../ui/features/analytics-collection-safety.md).
It covers the existing Analytics overview/detail surfaces. They do not render a
collection-policy status panel or fetch policy solely to display enabled state, project
identifier, or allowed-origin count. Ingestion safety and the existing settings/permission
boundary remain unchanged; this does not create a settings UI.

Details filters by event type and page/API path. Text applies after the shared pause;
selection changes apply immediately and reset pagination. Other reports clear and
disable the path filter, so an obsolete path cannot exclude unrelated events.

| State | User-visible meaning | Required behavior |
| --- | --- | --- |
| Loading | Analytics query is in progress. | Preserve shell and filters; never show zero as a placeholder. |
| Populated | Retained data is available. | Show stable metrics/details with their time/path meaning. |
| Empty | Query succeeded with no retained events in scope. | Distinguish no retained activity from no match for the applied filters. |
| Error | Activity query failed. | Show retry and keep last good data when available. |
| Permission | Viewer lacks the requested Analytics read/manage capability. | Hide mutation actions and explain the boundary. |
| Partial | A bounded diagnostic response has some unavailable summaries. | Identify missing categories; do not claim a complete report. |

## User-visible data effects

Valid events are persisted in Insights SQLite under existing retention. Invalid
or unauthorized batches are not persisted. Opt-out changes future collection
only; retention cleanup remains the existing Insights-owned lifecycle. Query
responses expose the same product-level activity concepts and do not expose
raw secrets or browser payloads.

## Affected product surfaces and dependencies

- Insights owns tracking validation, origin/key policy, event storage,
  retention, aggregation, and query semantics.
- The host application shell or deployment integration owns the bootstrap
  loader, visitor-consent entry, and tracker injection order; Admin and Insights
  do not claim visitor-consent verification.
- Admin owns module enablement, delegation, capability reconciliation, and the
  Web shell.
- `apps/web` owns Analytics overview/detail composition and the module API
  client; the tracker script remains a public Insights asset rather than a
  page-local client.
- Insights HTTP contract chain is `Rust ModuleRouter/Manifest -> handwritten
  apps/web/src/api/insights/contract.ts -> scripts/verify-worker-contracts.mjs`.
  It does not use the Admin OpenAPI/Orval chain.
- Existing `MetricCard`, `PageHeader`, `DataState`, route-local tables, and
  localized copy remain the UI owners.

## Acceptance criteria

- No tracker request, request patch, queue, or local visitor/session ID exists
  before the host bootstrap observes explicit visitor opt-in; an already loaded
  script remains inert until `consent === true`. Opt-out stops queued sends and
  removes local IDs without deleting retained events.
- Insights accepts HTTP ingestion only when server installation policy fields
  `collection_enabled`, `project`, and normalized `origin` match; HTTP success
  is not visitor-consent evidence. Browser verification covers the bootstrap
  entry, script-injection order, and no-request/no-ID behavior before opt-in.
- Cross-origin collection responds to `OPTIONS /api/insights/track` only for an
  enabled, configured, normalized allowed origin, returns exact non-wildcard
  CORS headers with `Vary: Origin`, and echoes that origin on policy-authorized
  `POST` success or business errors. Denied origins never receive an allow
  header.
- A missing, invalid, or origin-mismatched public project routing identifier is
  rejected by Insights and leaves no event row; it never authorizes query or
  management access.
- Stored page and API paths contain a pathname and no query string, hash,
  request body, or copied page text.
- Unknown event names and property keys are rejected by one documented
  allowlist; arbitrary button text and DOM attributes are never collected.
- Default visitor/session identifiers are pseudonymous; visitor IDs roll after
  24 hours, while session IDs rotate at the first of 30 minutes idle or 24 hours
  absolute lifetime, and are removed on opt-out; optional
  user identification requires explicit configuration and stable internal IDs.
- A request body over 64 KiB or batch over 50 events returns 413; a
  project/source process-local window over 30 requests or 300 events per minute
  returns 429; storage budget or free-disk protection returns 507. Every
  rejected batch persists zero events. The rate window resets when Insights
  restarts; the other caps remain hard fail-closed boundaries.
- Policy disable, key rotation, and origin removal are serialized with tracking
  admission; after each change, requests using the prior policy are rejected and
  create zero new rows.
- Oversized, overlong, future-dated, expired, or capacity-protected batches fail
  with a visible reason and no partial persistence.
- Analytics overview/detail keeps loading, populated, empty, error, permission,
  and partial semantics distinct and does not replace an ingestion failure with
  zero metrics.
- Read/manage capabilities are enforced by the backend and represented by the
  Insights handwritten contract verified through
  `scripts/verify-worker-contracts.mjs`; public collection never relies on UI
  visibility.
- Fixed copy is localized in Simplified Chinese and English while event names,
  paths, and runtime identifiers remain unchanged.
- The linked UI matrix covers light/dark, zh-CN/en-US, desktop/narrow,
  populated/empty/error/permission/partial, focus, contrast, and no overflow.

## Verification matrix

| Layer | Evidence | Acceptance |
| --- | --- | --- |
| Source/static | tracker, handler, validator, settings, route/capability review | No query-string capture, DOM text scrape, or unbounded custom payload remains. |
| Automated | Insights validation, retention, origin/identifier, opt-in, body/batch, process-local rate guards, storage/disk, batch-atomicity, policy barriers, and contract tests | 413/429/507 rejection paths persist zero events; 30/300 rate windows are scoped to one running Insights process and reset on restart; policy changes reject prior credentials without new rows; valid data retains existing query behavior. |
| Local HTTP fixture | Public ingestion and authenticated query requests in the Insights router test harness | A legal pathname remains queryable; `pagePath`, `apiPath`, and `referrer` carrying a query, fragment, absolute URL, free text, newline, or control character return 422 before persistence. A separate Web behavior test limits the target display projection to its fixed fields. This is local fixture and source behavior evidence, not a deployed-service or browser result. |
| HTTP | Real public ingestion, CORS preflight, and authenticated query requests | Only `collection_enabled`, project, normalized origin, exact preflight/POST CORS headers, payload, rate, storage, and role boundaries are observable; HTTP does not verify visitor consent. |
| Browser | Host bootstrap, tracker opt-in/opt-out, and Analytics UI matrix | `Not verified`: browser capture and visual rendering remain to be exercised. No initialization, request patch, request, queue, or ID before opt-in; loading the inert asset and consent-driven initialization remain distinct. |
| Runtime/deployment | configured origin and retention in the four-service bundle | `Not verified` until environment and browser policy are exercised. |

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- A single installation remains the supported data boundary.
- Pseudonymous visitor IDs remain locally for at most 24 hours and session IDs
  for at most 30 minutes idle/24 hours; server event retention is 30 days in
  the first release. Per-source request/event admission is process-local and
  resets on restart; the host owns consent copy outside this product spec.

### Open questions

- The exact operator-facing settings control and consent wording must be
  confirmed by the deployment owner before a settings UI is added. This does
  not block the server safety boundary or read-only Analytics surfaces. The
  numeric safety limits and status behavior above are not open.

### Rejected

- Auto-starting or initializing the tracker solely because a script tag or data
  attribute exists.
- Trusting a client-supplied project ID or origin without server verification.
- Collecting button text, query strings, form fields, or arbitrary DOM data.
- Treating rejected collection as a successful zero-event report.

### Deferred

- Multi-project analytics, segmentation, exports, warehouse delivery, and
  user-level identity joins.
- Advanced consent-management integrations and legal policy management.

## Ready for analytics collection safety implementation

The safety boundary, data ownership, permissions, failure semantics, non-goals,
and acceptance are fixed. The server implementation can proceed without a new
UI settings system; the linked Analytics UI contract is ready for the existing
overview/detail surfaces. External legal review, browser network evidence, and
deployment configuration remain `Not verified`.
