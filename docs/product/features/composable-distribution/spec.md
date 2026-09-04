# Composable distributions and notifications

Status: implementation underway; P1 selection and P2 minimal Admin backend are complete.
Decision date: 2026-09-03.

The user has authorized architecture design for a complete distribution and
physically pruned distributions, including monitoring alone, together with ten
ChatGPT review rounds. This changes the future distribution objective. It does
not claim that the current four-service implementation is already composable.

## Outcome

A developer can choose a supported set of product capabilities and build one
complete, signed product distribution with explicit host prerequisites. A full distribution includes all shipped
capabilities. A monitoring distribution includes server monitoring and its
essential access/security prerequisites. Unselected product capabilities do not
exist in its executable dependency closure, Web assets, HTTP routes, capability
catalog, schema, configuration, service units, or installed files.

The source template remains complete. Physical exclusion applies to the built
and installed product, not deletion of reusable source from this repository.

## Scope and ownership

| Capability | Owner | Included behavior |
| --- | --- | --- |
| Access foundation | Admin entry host | Login, account/session lifecycle, local roles and grants with essential access-settings UI, minimal security audit, selected-product navigation, Web hosting and gateway |
| Server monitoring | Monitor | Current Agent reports, nodes, resource samples, alert policies, incidents and daily summaries |
| Product analytics | Insights | Current bounded collection, overview, details and retention |
| Browser reporting | Reports | Current targets, flows, runs, artifacts, frames and bounded scheduling |
| Message center | Admin notifications feature | Durable personal inbox, unread state, incident/run links and live invalidation |
| Administration console | Admin optional features | Module presentation controls, system diagnostics and general administration dashboard; essential account/role settings remain in access |
| Release management UI | Admin optional feature | Existing signed-bundle management, restricted to the installed distribution identity |
| Generic task administration | Admin optional feature | Existing Admin task administration; distinct from Reports schedules |
| Operation-log console | Admin optional feature | Existing operation-log browsing and selected-service diagnostics |

No new analytics families, Reports credentials, Report Center, email, SMS,
mobile push, arbitrary webhook execution, plugin marketplace, WASM runtime,
multi-tenancy, operating-system abstraction, cluster scheduler, or general
workflow engine is implied by "full". Target full means the enumerated existing
capabilities plus the notification slice defined here.

Notifications are an authorized target capability, not an already released one:
the preceding user request explicitly asked to integrate SSE with the message
center and alerts, and the current request continues that design. The target
full distribution includes this slice after its own gates pass. Physical
distribution pruning can be implemented and accepted before notifications;
the existing full behavior baseline must remain independently testable.

## Distribution presets

| Preset | Product capabilities | Server processes | Databases |
| --- | --- | --- | --- |
| `full` | All rows above | Entry host, Monitor, Insights, Reports | Admin, Monitor, Insights, Reports |
| `monitor` | Access foundation + monitoring | Minimal entry host + Monitor | Minimal Admin schema + Monitor |
| `monitor-notify` | `monitor` + message center | Minimal entry host + Monitor | Minimal Admin schema with inbox + Monitor with outbox |
| `analytics` | Access foundation + analytics | Minimal entry host + Insights | Minimal Admin schema + Insights |
| `reports` | Access foundation + browser reporting | Minimal entry host + Reports | Minimal Admin schema + Reports |
| `custom` | Explicit validated selection | Entry host + selected services | Only selected owners and schema fragments |

These are presets over one finite capability catalog, not separately maintained
products. There is one source implementation per behavior and one release
version per installed distribution. No fifth resident notification service is
needed. The node Agent is a separate, non-Web deployment artifact.

The monitoring preset's entry host is intentionally retained for authenticated
access. Its binary name does not authorize retaining the full Admin product:
there must be no release-management API, task console, analytics/reporting
route, browser automation dependency, general dashboard, or unused schema.
Its installation default landing page is Monitoring; users without Monitor access
land on an authorized access setting or their own profile. Access settings are restricted to the minimum
needed to administer the installation securely. A one-process monitoring
appliance is an evaluated alternative, not an additional initial topology.

## Meaning of absence

1. An omitted service has no binary/image, service unit, process, port, database,
   background task, periodic health request, or required secret.
2. An omitted local feature has no handler registration, capability declaration,
   SQL tables/indexes/seeds, Web route, chunk, image, API client in the product
   build graph, or feature-owned runtime dependency.
3. Requests to omitted API namespaces return 404, including requests from the
   owner. They never fall through to the SPA HTML fallback.
4. Navigation, search, role selectors and command help contain only installed
   features. No disabled placeholders or "not installed" advertising panels.
5. An installed-but-unavailable service remains visible to authorized users and
   returns a useful unavailable state. Absence, disabled state, lack of
   permission and service outage are different conditions.
6. Optional settings are rejected when their owner is absent. An environment
   variable cannot load code omitted at build time.

Essential infrastructure shared with a selected feature is allowed. For
example, HTTP, TLS, SQLite and authorization remain necessary for monitoring.
The absence test uses feature ownership and dependency reachability, not a
ban on common library names.

## Notification journeys

### Monitoring without a message center

An operator reads active and resolved incidents in Monitoring. Detection and
history work even though no inbox, unread badge, SSE endpoint, delivery worker,
or notification outbox exists. Existing bounded polling remains valid.

### Monitoring with a message center

1. An accepted Agent report or offline scan changes an incident lifecycle.
2. Monitor commits the incident and its notification event in the same database
   transaction. Repeated abnormal samples do not create repeated open messages.
3. A bounded relay delivers the event to Admin; retries retain the same ID.
4. Admin validates the producer and installed topic, resolves eligible users,
   and atomically stores its receipt, message and recipient records.
5. Only after commit does Admin attempt live invalidation of online inboxes.
6. A user can read history after being offline, mark a message read, and navigate
   to the incident if still authorized. Reading does not acknowledge or resolve
   an incident.
7. If the SSE signal is lost, reconnect and periodic reconciliation retrieve
   the authoritative inbox. A push attempt is never recorded as human receipt.

### Reports with notifications

Terminal successful, failed and cancelled manual runs may create messages for
their persisted initiator if the user remains active and authorized. Scheduled
runs initially have no personal notification policy. Frames, screenshots, input
values and every progress tick do not become durable messages. Ordinary run
execution and cancellation do not depend synchronously on Admin availability.

## Acceptance and release boundary

The initial acceptance target is a fresh installation. The existing template's
no-legacy-migration rule remains: do not add a historical upgrade framework or
delete existing developer databases. Any install-time mismatch with an existing
schema fails before writes.

A different capability selection is a different distribution identity. Changing
an existing installation from full to monitor is not a runtime toggle or an
implicit destructive uninstall. Initial support is a fresh target directory;
the old installation is preserved and cannot share live database files with
the new one. The current AGENTS.md requires fresh baselines only: different
builds use fresh installation data. Historical upgrade and rollback compatibility
are excluded. Only restarting or recovering an interrupted installation of the
same exact build may reuse its own matching database. See [implementation](implementation.md)
for the current execution plan and the correction to the reviewed alternative.

Quantitative reliability and load requirements are test targets, not measured
claims. See [validation](validation.md). Ten actual, attributed ChatGPT responses
were captured and reconciled. Final external verdict: design PASS WITH CHANGES;
the last two local contract corrections are recorded in [review](review.md).
The coordinator considers the design ready for slice-by-slice implementation.
No compiled distribution or runtime acceptance is claimed.

## Selected native layout contract

The current Monitor native-layout contract is a canonical generated artifact.
The server selection declares only `rz.target`, `rz-admin.service`, and
`rz-monitor.service`, with `rz-admin.env` and `rz-monitor.env` scoped to their
respective consumers. The node-agent selection declares only
`rz-monitor-agent.service` and `rz-monitor-agent.env`. It is bound to the
resolver composition, artifact class and configuration owners, and rejects
foreign, duplicate, missing or stale members before package work. Its verified
byte digest is part of the release manifest and build identity. This is not yet
installer publication or a replacement for the existing full-layout files.
The generated Monitor units retain service start limits and the Agent retains
`network-online.target`; every service names its distinct non-root identity.
Recovery/`ExecCondition` wiring is omitted until the fresh-root
installer/recovery closure exists.

## Design package

- [Implementation plan and current progress](implementation.md)
- [Architecture and alternatives](architecture.md)
- [Interfaces and persistence](contracts.md)
- [Implementation slices and acceptance tests](validation.md)
- [Community evidence and applicability](sources.md)
- [Review record](review.md)
