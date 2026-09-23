# Full distribution and notifications

Status: first-release contract and implementation are being closed to one complete
signed distribution. A signed 0.5.1 fresh installation and its release worker have
passed in a disposable systemd container. Transition from the existing 0.5.0 worker
and production deployment remain unverified.

## Outcome

A developer builds one complete, signed product distribution with explicit host
prerequisites. It contains Admin, Monitor, Insights, Reports, the Web application,
notifications and the existing administration surfaces under one version and rollback
boundary. All product modules are installed and enabled by default. The owner may
disable an installed module without changing the signed installed inventory.

The operator journey is one path:

1. build the complete signed bundle;
2. copy the bundle and packaged installer to the server;
3. run the installer with the bundle path;
4. optionally edit ordinary runtime settings;
5. run `rz start` and observe all four services healthy. The installer activates the
   enabled update-request watcher without starting the server services, so an owner
   deployment request can trigger the root-only update worker before the next reboot.

The installer accepts no password or key arguments. The build supplies the public
release verification key, and the signing private key never enters the bundle.
Internal JWT, IPC, Agent and notification secrets are generated locally during a
fresh installation. Monitor creates, migrates and validates its own fresh database
when the controller starts; its internal database lifecycle is not an operator task.

## Scope and ownership

| Capability | Owner | Included behavior |
| --- | --- | --- |
| Access foundation | Admin entry host | Login, account/session lifecycle, local roles and grants with essential access-settings UI, minimal security audit, current module navigation, Web hosting and gateway |
| Server monitoring | Monitor | Current Agent reports, nodes, resource samples, alert policies, incidents and daily summaries |
| Product analytics | Insights | Current bounded collection, overview, details and retention |
| Browser reporting | Reports | Current targets, flows, runs, artifacts, frames and bounded scheduling |
| Message center | Admin notifications feature | Durable personal inbox, unread state, incident/run links and live invalidation |
| Administration console | Admin optional features | Module presentation controls, system diagnostics and general administration dashboard; essential account/role settings remain in access |
| Release management UI | Admin optional feature | Existing signed-bundle management, restricted to the installed distribution identity |
| Generic task administration | Admin optional feature | Existing Admin task administration; distinct from Reports schedules |
| Operation-log console | Admin optional feature | Existing operation-log browsing and module-service diagnostics |

No new analytics families, Reports credentials, Report Center, email, SMS,
mobile push, arbitrary webhook execution, plugin marketplace, WASM runtime,
multi-tenancy, operating-system abstraction, cluster scheduler, or general
workflow engine is implied by "full". Target full means the enumerated existing
capabilities plus the notification slice defined here.

Notifications are part of the complete distribution. Their focused source, runtime
and browser evidence remains supporting input, but it does not certify the current
dirty full-release candidate. External production deployment remains Not verified.

The installed release configuration contains every production secret required by
the enabled notification paths. Admin and Monitor consume one shared notification
event key. Admin and Reports consume one separate shared Reports notification event
key. The installer generates and writes these values atomically; operators do not
fill secret placeholders or discover them through successive service failures.

P5 is split into bounded delivery slices. P5a owns the Admin notification schema
and authenticated personal-inbox read state: list, unread count, detail,
idempotent single read, and sequence-bounded read-all.
Every operation rechecks the enabled user, current database grants, and enabled
producer module in one Admin database snapshot. Admin applies and verifies the
base and notification migration ledgers, while its formal contract exports base
and notification route owners separately from their registered Rust routes. P5a
added no notification configuration fields. P5b adds the `notifications`
configuration owner for the Admin-owned logical budgets, 128 MiB filesystem
reserve and sustained WAL-pressure threshold. P5b implements retention and
admission behind an internal service. The corresponding UI
states and deferred shell work are fixed in
[Message Center UI](../../../ui/features/message-center.md).

P5b owns an internal Admin admission service only. It checks a durable
trigger-maintained accounting singleton and commits receipt, message, recipient
and user-revision changes in one immediate SQLite transaction. Expired history
is excluded by the same injected-clock cutoff at every read entry before its
bounded physical reclamation. Admin performs one bounded cleanup at startup and
one per hour; admission may independently commit up to
eight bounded cleanup rounds before reopening its final write transaction.
Existing receipts are rechecked at each transaction boundary before expiry,
storage or budget admission checks. The final transaction resolves the exact
audience and projected charge, then rechecks free space under its write lock.
P5b adds no process or public producer route; Monitor and Reports connect their
durable outboxes in P6.

P6a selects only the Monitor incident producer path. A first active transition
and the first legal resolution allocate their immutable event and outbox row in
the incident write transaction. Duplicate or stale Agent reports and unchanged
active incidents allocate no event. The Monitor process claims and relays its
own bounded outbox; Admin authenticates the producer on a notifications-only
loopback ingress and recomputes current recipients from its own identity,
permission and enabled-module state. Delivery leases are fenced. Response loss
and read/response timeouts retain the original event bytes and ID for
reconciliation; 429, 503 and explicit connect-before-send failures retain
retryable pending state. Both paths use deterministic jittered backoff. The exact
HTTP loopback origin/path is parsed rather than
prefix-matched. A previous signing key is accepted only through an explicit
cutoff no more than 120 seconds after ingress startup. Terminal results delete
the pending payload, and bounded status/quarantine counters expose delivery gaps.
This adds no fifth service. Reports producer delivery and trusted initiator
persistence remain outside P6a.

P6b adds the Reports producer without broadening the notification product.
The verified delegated user that creates a manual run, or creates a retry run,
is stored once as the run's immutable `initiator_user_id`; request JSON cannot
set or replace it. Scheduled runs store `NULL`. Only a conditional transition
that actually enters `succeeded`, `failed` or `cancelled` may write one terminal
event and one outbox row in the same Reports transaction. Repeated cancellation,
recovery of an already-terminal run and a completion/cancellation race therefore
allocate no second event. Admin treats the stored initiator as a candidate only
and rechecks the current enabled user, current `reports:run:view` grant and
enabled Reports module in the admission transaction. Reports execution never
waits synchronously for Admin delivery.

## Release composition

| Release | Product capabilities | Server processes | Databases |
| --- | --- | --- | --- |
| `full` | All rows above | Entry host, Monitor, Insights, Reports | Admin, Monitor, Insights, Reports |

There is one source implementation per behavior and one release version per installed
distribution. No fifth resident notification service is needed. The node Agent remains
a separate non-Web deployment artifact.

Installed inventory, runtime enablement, service availability and user authorization
remain independent states. Disabling a module removes its navigation and gateway
access. An enabled module whose service is unavailable remains visible to authorized
users and returns a useful unavailable state. User permissions continue to apply only
to enabled installed behavior.

## Notification journeys

### Monitoring message center

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
Retries are new manual runs whose verified retrying user becomes their immutable
initiator. A queued cancellation emits on the first winning terminal update;
running cancellation emits only when execution actually enters `cancelled`.

## Acceptance and release boundary

Initial acceptance uses a fresh target directory and fresh databases. A release is
accepted only when the package verifies without external key input, installation
requires no secret input, `rz-full.service` starts all four services, each service reports
the exact release version healthy, a host restart restores the same state, and a
failed Admin API update restores the previous release and databases. See
[implementation](implementation.md) for the current execution plan and
[validation](validation.md) for the executable acceptance matrix.

Quantitative reliability and load requirements are test targets, not measured
claims. See [validation](validation.md). The first release is implemented as one
complete signed distribution; its current source, artifact and target-like runtime
evidence is recorded in the local verification guide. External production deployment
remains unverified until an operator authorizes a real target host.

## Design package

- [Implementation plan and current progress](implementation.md)
- [Deployment architecture](../../../architecture.md#release-topology)
- [Bundle, installation and update guide](../../../guides/deployment.md)
- [Executable acceptance](validation.md)
