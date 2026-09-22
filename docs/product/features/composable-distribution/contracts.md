# Full distribution, event and inbox contracts

Status: current.

## Release contract

The server release is one complete signed bundle containing exactly the Web
application, `rz`, Admin, Monitor, Insights, Reports, production configuration
templates and six systemd units. It has one version, one signature, one installation
root, one active `current` link and one rollback boundary.

Frontend and backend digests are inspection fields within that signed identity; they
are not independently installable signatures. Runtime module enablement and user
authorization do not alter the installed inventory.

The packaged installer accepts the bundle path only. Verification trust is packaged
with the installer, while the signing private key remains in the build environment.
The installer generates internal service secrets and never asks the operator for an
administrator password, signing key, verification key or database command.

## Runtime contract

The public service commands are:

```text
rz start
rz stop
rz restart
rz status
```

`rz-full.service` groups `rz-admin.service`, `rz-monitor.service`,
`rz-insights.service` and `rz-reports.service`. `status` succeeds only when all four
resident units are active. `start` and `restart` succeed only after all four become
active within the bounded readiness interval. `stop` stops the aggregate service set.

Each resident service owns its configuration, database, migrations, logs and health.
Against an empty runtime root, service startup creates and validates the owned SQLite
database. No caller invokes a separate database initialization, binding or validation
command during installation.

The node Agent is a separate managed-node artifact and protocol peer. It is not a
server-module release and does not contain Admin or Web assets.

## Update contract

Only an authorized owner may submit a release candidate to Admin. The deployment
transaction:

1. verifies the complete candidate and exact member inventory;
2. snapshots all four databases;
3. publishes the immutable release and switches `current`;
4. restarts all four resident services;
5. verifies their exact release identity and health;
6. commits success, or restores the previous link and database snapshots.

Interrupted transactions are handled by the recovery unit. Service-specific updates,
mixed versions and partial database rollback are invalid.

## Producer event contract

Monitor and Reports own their business events and outboxes. An event contains a stable
event ID, producer, topic, subject reference, bounded presentation fields and creation
time. It never contains credentials, arbitrary recipient grants or executable content.

The producer commits the business transition and outbox row in the same database
transaction. Retrying delivery retains the same event ID. A repeated ID with the same
payload is an idempotent success; a repeated ID with a different payload is a conflict.

Producer ingress is loopback-only and authenticated with a producer-specific secret.
The signature covers the producer identity, timestamp, nonce, method, path and body
digest. The public reverse proxy must not expose this ingress.

Admin owns recipient resolution. It checks the current enabled user, current module
state and current capability grants during admission. Producers cannot choose users or
grant permissions.

## Inbox persistence contract

Admin transactionally stores:

- the accepted producer receipt and payload digest;
- one durable message;
- the eligible recipient projections;
- read state per recipient.

List, unread count, detail and mark-read operations always enforce the current user
and authorization state. Reading a Monitoring notification does not acknowledge or
resolve the incident. Disabled users, revoked grants and disabled modules cannot use a
previously issued JWT to retain inbox access.

Reports may notify the immutable initiator of a terminal manual run when that user is
still active and authorized. Scheduled runs have no personal recipient by default.
Retries are new runs and use the verified retrying user as their initiator. Competing
terminal transitions may create at most one event.

## SSE contract

SSE carries invalidation hints after durable commit. Clients use the existing Bearer
header, never a query-string credential. Each connection has bounded buffering,
cancellation, retry backoff and periodic authorization revalidation. Session expiry,
revocation or permission loss closes the stream.

Clients reload the authoritative unread count and message list on initial connection,
reconnect and periodic reconciliation. Sequence numbers are not a durable replay log,
and a live event is not recorded as human receipt.

## Error and evidence boundaries

Invalid signatures, stale timestamps, reused nonces with different payloads, unknown
producers, unsupported topics, unauthorized users and malformed bodies fail closed.
Errors use the owning API's stable JSON envelope and do not reveal secrets.

Source checks, artifact verification, fresh installation, systemd operation, browser
journeys and external production deployment are independent evidence layers. Current
acceptance requirements are defined in [validation](validation.md).
